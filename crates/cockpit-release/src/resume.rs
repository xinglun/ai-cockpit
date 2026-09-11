//! Identity-bound, append-only release phase receipts.
//!
//! The release shell harnesses call this module through the `cockpit-release`
//! binary.  Shell is still responsible for invoking the adopter/runtime
//! commands, but it does not decide whether a persisted phase is reusable.
//! A receipt is reusable only when the complete release identity and every
//! evidence digest bind to the current request.

use std::{fs, path::Path};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{
    acceptance::{
        AcceptanceScope, EvidenceReference, PhaseBoundaries, PhaseResult, PhaseStatus,
        ReleaseIdentity, ReleasePhase,
    },
    recovery::{PhaseAction, RecoveryPlan},
};

pub const RECEIPT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum ReceiptError {
    #[error("receipt file is not valid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("receipt file operation failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("receipt identity does not match the current release identity")]
    IdentityMismatch,
    #[error("receipt identity digest is corrupt")]
    CorruptIdentityDigest,
    #[error("receipt contains a result with a mismatched identity")]
    ResultIdentityMismatch,
    #[error("receipt contains a corrupt result identity digest")]
    ResultIdentityDigest,
    #[error("receipt contains duplicate phase: {0:?}")]
    DuplicatePhase(ReleasePhase),
    #[error("receipt result is not reusable: {0:?}")]
    ResultNotReusable(PhaseStatus),
    #[error("evidence is missing, symlinked, or not a regular file: {0}")]
    InvalidEvidence(String),
    #[error("receipt evidence digest does not match the current evidence file")]
    EvidenceDigestMismatch,
    #[error("cannot replace receipt atomically: {0}")]
    AtomicWrite(String),
    #[error("recovery plan is invalid: {0}")]
    RecoveryPlan(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PhaseReceiptStore {
    pub schema_version: u32,
    pub identity: ReleaseIdentity,
    pub identity_digest: String,
    pub results: Vec<PhaseResult>,
}

impl PhaseReceiptStore {
    pub fn empty(identity: ReleaseIdentity) -> Self {
        let identity_digest = identity.digest();
        Self {
            schema_version: RECEIPT_SCHEMA_VERSION,
            identity,
            identity_digest,
            results: Vec::new(),
        }
    }

    /// Load a receipt store or create the in-memory empty store for the first
    /// attempt. Existing stores are strict: changing any input, artifact,
    /// runtime, target, or acceptance scope is a fail-closed error.
    pub fn load(path: &Path, expected: &ReleaseIdentity) -> Result<Self, ReceiptError> {
        match fs::symlink_metadata(path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
                    return Err(ReceiptError::InvalidEvidence(path.display().to_string()));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::empty(expected.clone()));
            }
            Err(error) => return Err(ReceiptError::Io(error)),
        }
        let store: Self = serde_json::from_slice(&fs::read(path)?)?;
        store.validate_binding(expected)?;
        Ok(store)
    }

    pub fn validate_binding(&self, expected: &ReleaseIdentity) -> Result<(), ReceiptError> {
        if self.schema_version != RECEIPT_SCHEMA_VERSION {
            return Err(ReceiptError::CorruptIdentityDigest);
        }
        if self.identity_digest != self.identity.digest() {
            return Err(ReceiptError::CorruptIdentityDigest);
        }
        if self.identity != *expected {
            return Err(ReceiptError::IdentityMismatch);
        }

        let mut phases = std::collections::BTreeSet::new();
        for result in &self.results {
            if !phases.insert(result.phase) {
                return Err(ReceiptError::DuplicatePhase(result.phase));
            }
            if result.identity != self.identity {
                return Err(ReceiptError::ResultIdentityMismatch);
            }
            if result.identity_digest != result.identity.digest() {
                return Err(ReceiptError::ResultIdentityDigest);
            }
            if result.status == PhaseStatus::Succeeded {
                let Some(evidence) = result.evidence.as_ref() else {
                    return Err(ReceiptError::InvalidEvidence(format!(
                        "missing evidence for {}",
                        result.phase.as_str()
                    )));
                };
                validate_evidence(evidence)?;
            }
        }
        Ok(())
    }

    pub fn plan(&self) -> Result<RecoveryPlan, ReceiptError> {
        self.plan_for_scope(AcceptanceScope::Full)
    }

    pub fn plan_for_scope(&self, scope: AcceptanceScope) -> Result<RecoveryPlan, ReceiptError> {
        RecoveryPlan::from_results_for_scope(&self.identity, &self.results, scope)
            .map_err(|error| ReceiptError::RecoveryPlan(format!("{error:?}")))
    }

    /// Record one successful phase. The same phase can only be recorded again
    /// when its evidence digest is identical; a changed result is rejected.
    pub fn record_success(
        &mut self,
        phase: ReleasePhase,
        attempt: u32,
        evidence_id: impl Into<String>,
        evidence_path: &Path,
    ) -> Result<(), ReceiptError> {
        let metadata = fs::symlink_metadata(evidence_path)
            .map_err(|_| ReceiptError::InvalidEvidence(evidence_path.display().to_string()))?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            return Err(ReceiptError::InvalidEvidence(
                evidence_path.display().to_string(),
            ));
        }
        let evidence_digest = sha256_file(evidence_path)?;
        let evidence = EvidenceReference {
            id: evidence_id.into(),
            path: evidence_path.display().to_string(),
            digest: format!("sha256:{evidence_digest}"),
        };

        if let Some(existing) = self.results.iter().find(|result| result.phase == phase) {
            if existing.status == PhaseStatus::Succeeded
                && existing.evidence.as_ref() == Some(&evidence)
                && existing.binding_matches(&self.identity)
            {
                return Ok(());
            }
            return Err(ReceiptError::DuplicatePhase(phase));
        }

        self.results.push(PhaseResult::succeeded(
            phase,
            self.identity.clone(),
            attempt,
            PhaseBoundaries {
                started_at: Utc::now().to_rfc3339(),
                finished_at: Some(Utc::now().to_rfc3339()),
                elapsed_ms: Some(0),
            },
            evidence,
        ));
        self.results.sort_by_key(|result| result.phase);
        self.validate_binding(&self.identity.clone())
    }

    pub fn write_atomic(&self, path: &Path) -> Result<(), ReceiptError> {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let temp = path.with_extension(format!("json.tmp.{}", std::process::id()));
        let bytes = serde_json::to_vec_pretty(self)?;
        fs::write(&temp, bytes)?;
        fs::rename(&temp, path).map_err(|error| ReceiptError::AtomicWrite(error.to_string()))
    }
}

pub fn plan_for_phase(plan: &RecoveryPlan, phase: ReleasePhase) -> Option<&PhaseAction> {
    plan.actions.iter().find(|action| match action {
        PhaseAction::NotApplicable {
            phase: action_phase,
        }
        | PhaseAction::Reuse {
            phase: action_phase,
            ..
        }
        | PhaseAction::Run {
            phase: action_phase,
        }
        | PhaseAction::Retry {
            phase: action_phase,
            ..
        }
        | PhaseAction::Blocked {
            phase: action_phase,
            ..
        } => *action_phase == phase,
    })
}

fn sha256_file(path: &Path) -> Result<String, ReceiptError> {
    let mut file = fs::File::open(path)?;
    let mut digest = Sha256::new();
    std::io::copy(&mut file, &mut digest)?;
    Ok(hex::encode(digest.finalize()))
}

fn validate_evidence(evidence: &EvidenceReference) -> Result<(), ReceiptError> {
    let path = Path::new(&evidence.path);
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| ReceiptError::InvalidEvidence(evidence.path.clone()))?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(ReceiptError::InvalidEvidence(evidence.path.clone()));
    }
    let actual = format!("sha256:{}", sha256_file(path)?);
    if actual != evidence.digest {
        return Err(ReceiptError::EvidenceDigestMismatch);
    }
    Ok(())
}
