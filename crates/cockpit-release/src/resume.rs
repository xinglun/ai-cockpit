//! Identity-bound, append-only release phase receipts.
//!
//! The release shell harnesses call this module through the `cockpit-release`
//! binary.  Shell is still responsible for invoking the adopter/runtime
//! commands, but it does not decide whether a persisted phase is reusable.
//! A receipt is reusable only when the complete release identity and every
//! evidence digest bind to the current request.

use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{
    acceptance::{
        AcceptanceScope, EvidenceReference, PhaseBoundaries, PhaseResult, PhaseStatus,
        ReleaseIdentity, ReleasePhase,
    },
    recovery::{PhaseAction, PhaseFailure, RecoveryPlan},
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
    #[error("receipt phase result is missing a failure description")]
    MissingFailure,
    #[error("replacement phase attempt must be greater than the recorded attempt")]
    AttemptNotMonotonic,
    #[error("receipt phase result contains an inconsistent failure strategy")]
    InconsistentFailure,
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
    #[serde(default)]
    pub history: Vec<PhaseResult>,
    /// Runtime-only location of the restored receipt.  Evidence paths in a
    /// receipt are portable by basename, so validation after artifact
    /// relocation must resolve against this directory rather than an old
    /// absolute runner path.
    #[serde(skip)]
    evidence_base: Option<PathBuf>,
}

impl PhaseReceiptStore {
    pub fn empty(identity: ReleaseIdentity) -> Self {
        let identity_digest = identity.digest();
        Self {
            schema_version: RECEIPT_SCHEMA_VERSION,
            identity,
            identity_digest,
            results: Vec::new(),
            history: Vec::new(),
            evidence_base: None,
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
        let mut store: Self = serde_json::from_slice(&fs::read(path)?)?;
        store.evidence_base = path.parent().map(Path::to_path_buf);
        store.validate_binding_at(expected, store.evidence_base.as_deref())?;
        Ok(store)
    }

    pub fn validate_binding(&self, expected: &ReleaseIdentity) -> Result<(), ReceiptError> {
        self.validate_binding_at(expected, None)
    }

    fn validate_binding_at(
        &self,
        expected: &ReleaseIdentity,
        evidence_base: Option<&Path>,
    ) -> Result<(), ReceiptError> {
        if self.schema_version != RECEIPT_SCHEMA_VERSION {
            return Err(ReceiptError::CorruptIdentityDigest);
        }
        if !self.identity.digest_matches(&self.identity_digest) {
            return Err(ReceiptError::CorruptIdentityDigest);
        }
        if !self.identity.binding_matches(expected) {
            return Err(ReceiptError::IdentityMismatch);
        }

        let mut phases = std::collections::BTreeSet::new();
        for result in &self.results {
            if !phases.insert(result.phase) {
                return Err(ReceiptError::DuplicatePhase(result.phase));
            }
            self.validate_result(result, evidence_base)?;
        }
        for result in &self.history {
            self.validate_result(result, evidence_base)?;
        }
        Ok(())
    }

    fn validate_result(
        &self,
        result: &PhaseResult,
        evidence_base: Option<&Path>,
    ) -> Result<(), ReceiptError> {
        if result.identity != self.identity {
            return Err(ReceiptError::ResultIdentityMismatch);
        }
        if !result.identity.digest_matches(&result.identity_digest) {
            return Err(ReceiptError::ResultIdentityDigest);
        }
        if result.status == PhaseStatus::Succeeded {
            let Some(evidence) = result.evidence.as_ref() else {
                return Err(ReceiptError::InvalidEvidence(format!(
                    "missing evidence for {}",
                    result.phase.as_str()
                )));
            };
            validate_evidence(evidence, evidence_base)?;
        }
        if matches!(
            result.status,
            PhaseStatus::Failed | PhaseStatus::Interrupted | PhaseStatus::TimedOut
        ) {
            let Some(failure) = result.failure.as_ref() else {
                return Err(ReceiptError::MissingFailure);
            };
            if !failure.is_consistent() {
                return Err(ReceiptError::InconsistentFailure);
            }
            if failure.status() != result.status {
                return Err(ReceiptError::InconsistentFailure);
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

        let replacement = PhaseResult::succeeded(
            phase,
            self.identity.clone(),
            attempt,
            PhaseBoundaries {
                started_at: Utc::now().to_rfc3339(),
                finished_at: Some(Utc::now().to_rfc3339()),
                elapsed_ms: Some(0),
            },
            evidence,
        );
        if let Some(index) = self.results.iter().position(|result| result.phase == phase) {
            let existing = &self.results[index];
            if existing.status == PhaseStatus::Succeeded
                && existing
                    .evidence
                    .as_ref()
                    .zip(replacement.evidence.as_ref())
                    .is_some_and(|(left, right)| left.id == right.id && left.digest == right.digest)
                && existing.binding_matches(&self.identity)
            {
                return Ok(());
            }
            if existing.status == PhaseStatus::Succeeded {
                return Err(ReceiptError::DuplicatePhase(phase));
            }
            if attempt <= existing.attempt {
                return Err(ReceiptError::AttemptNotMonotonic);
            }
            let previous = self.results.remove(index);
            self.history.push(previous);
        }
        self.results.push(replacement);
        self.results.sort_by_key(|result| result.phase);
        self.validate_binding_at(&self.identity.clone(), self.evidence_base.as_deref())
    }

    /// Record a failed, interrupted, or timed-out phase without overwriting
    /// the prior attempt. The latest result drives recovery; older attempts
    /// remain in `history` for diagnosis and audit.
    pub fn record_failure(
        &mut self,
        phase: ReleasePhase,
        attempt: u32,
        failure: PhaseFailure,
    ) -> Result<(), ReceiptError> {
        let result = PhaseResult::failed_with_status(
            phase,
            self.identity.clone(),
            attempt,
            PhaseBoundaries {
                started_at: Utc::now().to_rfc3339(),
                finished_at: Some(Utc::now().to_rfc3339()),
                elapsed_ms: Some(0),
            },
            failure.status(),
            failure,
        );
        if let Some(index) = self.results.iter().position(|stored| stored.phase == phase) {
            if self.results[index].status == PhaseStatus::Succeeded {
                return Err(ReceiptError::DuplicatePhase(phase));
            }
            if attempt <= self.results[index].attempt {
                return Err(ReceiptError::AttemptNotMonotonic);
            }
            let previous = self.results.remove(index);
            self.history.push(previous);
        }
        self.results.push(result);
        self.results.sort_by_key(|stored| stored.phase);
        self.validate_binding_at(&self.identity.clone(), self.evidence_base.as_deref())
    }

    pub fn write_atomic(&self, path: &Path) -> Result<(), ReceiptError> {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        self.validate_binding_at(&self.identity.clone(), Some(parent))?;
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

fn validate_evidence(
    evidence: &EvidenceReference,
    evidence_base: Option<&Path>,
) -> Result<(), ReceiptError> {
    let original = Path::new(&evidence.path);
    let restored = evidence_base.and_then(|base| {
        if original.is_absolute() {
            original.file_name().map(|filename| base.join(filename))
        } else {
            Some(base.join(original))
        }
    });
    // When a receipt has been restored, never trust the old absolute path:
    // it may still exist on the runner and contain unrelated bytes.  The
    // restored artifact must be the one whose digest is checked.
    let candidates = if evidence_base.is_some() {
        [restored, None]
    } else {
        [Some(original.to_path_buf()), None]
    };
    for candidate in candidates.into_iter().flatten() {
        let Ok(metadata) = fs::symlink_metadata(&candidate) else {
            continue;
        };
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            return Err(ReceiptError::InvalidEvidence(evidence.path.clone()));
        }
        let actual = format!("sha256:{}", sha256_file(&candidate)?);
        if actual != evidence.digest {
            return Err(ReceiptError::EvidenceDigestMismatch);
        }
        return Ok(());
    }
    Err(ReceiptError::InvalidEvidence(evidence.path.clone()))
}
