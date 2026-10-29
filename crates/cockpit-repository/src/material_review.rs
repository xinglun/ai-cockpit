//! Read-only, committed-source material identity. No decision or discharge is
//! performed here; the opt-in is only reported to a future review service.

use super::{
    ObserverError, acquire_lifecycle_lock, contains_strong_instruction_injection,
    create_and_open_cap_directory, derive_governance_signals_with_diagnostics,
    effective_policy_for_contract, open_cap_directory_nofollow_strict,
    read_cap_file_nofollow_bounded, repository_id, validate_work_item_id,
};
use crate::rust_material::{MaterialUnknownCause, RustMaterialAssessment};
#[cfg(windows)]
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
#[cfg(windows)]
use cap_std::fs::OpenOptions as CapOpenOptions;
use cap_std::{ambient_authority, fs::Dir};
use cockpit_core::Digest;
use cockpit_git::{
    BoundedGitOutput, ChangeContentState, ChangeEvidence, ChangeKind, GitError, GitRepository,
    MAX_BOUNDED_GIT_OUTPUT_BYTES, MAX_CHANGE_TEXT_BYTES, RepositorySnapshot,
};
use cockpit_protocol::{
    Contract, MATERIAL_INSPECTION_REVIEW_CAPABILITY,
    MATERIAL_INSPECTION_REVIEW_DECISION_SCHEMA_VERSION, MaterialInspectionReviewAssurance,
    MaterialInspectionReviewDecision, MaterialInspectionReviewDecisionInput,
    MaterialInspectionReviewDecisionReceipt, RuntimeContext, digest_json,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
#[cfg(not(any(unix, windows)))]
use std::fs::OpenOptions;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io,
    path::Path,
};
use thiserror::Error;

const SCANNER_SEMANTIC_VERSION: &str = "rust-material-v1";
const ANALYSIS_TARGET_SEMANTIC_PROFILE: &str = "cross-platform-rust-source-v1";
const MAX_MATERIAL_BLOB_BYTES: u64 = 16 * 1024 * 1024;
const MAX_MATERIAL_CHANGED_FILES: usize = 256;
const MAX_MATERIAL_TOTAL_BLOB_BYTES: u64 = 32 * 1024 * 1024;
const MAX_MATERIAL_TOTAL_TEXT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_PATCH_BYTES: usize = MAX_BOUNDED_GIT_OUTPUT_BYTES;
const MAX_ACTIVE_CONTRACT_BYTES: u64 = MAX_BOUNDED_GIT_OUTPUT_BYTES as u64;
const MATERIAL_REVIEW_EVIDENCE_DIRECTORY: &str = "material-inspection-review";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialScannerAssessment {
    Clean,
    Finding,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaterialReviewEntry {
    pub path: String,
    pub kind: String,
    pub content_state: String,
    pub scanner_assessment: MaterialScannerAssessment,
    pub unknown_cause: Option<MaterialUnknownCause>,
    pub reviewable: bool,
    pub changed_hunk_digest: Digest,
    pub after_blob_digest: Option<Digest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaterialReviewRequest {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub contract_digest: Digest,
    pub immutable_contract_base_revision: String,
    pub source_snapshot_digest: Digest,
    pub material_manifest_digest: Digest,
    pub scanner_semantic_version: String,
    pub analysis_implementation_digest: Digest,
    pub analysis_target_semantic_profile: String,
    pub material_inspection_review_profile_digest: Option<Digest>,
    pub effective_policy_digest: Digest,
    pub entries: Vec<MaterialReviewEntry>,
    pub raw_unknown_codes: Vec<String>,
    pub finding_codes: Vec<String>,
    pub blocked_by_finding: bool,
    pub review_enabled: bool,
    pub review_diagnostic: Option<String>,
    pub request_digest: Digest,
    /// Provenance only. It is excluded from request_digest so an identical
    /// source manifest in a legitimate descendant remains comparable.
    pub reviewed_source_head: String,
}

impl<'de> Deserialize<'de> for MaterialUnknownCause {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "missing_complete_source" => Ok(Self::MissingCompleteSource),
            "non_text_material" => Ok(Self::NonTextMaterial),
            "source_over_budget" => Ok(Self::SourceOverBudget),
            "invalid_provenance" => Ok(Self::InvalidProvenance),
            "parse_unavailable" => Ok(Self::ParseUnavailable),
            "analysis_incomplete" => Ok(Self::AnalysisIncomplete),
            "readable_committed_rust_syntax_unknown" => {
                Ok(Self::ReadableCommittedRustSyntaxUnknown)
            }
            _ => Err(serde::de::Error::custom("unknown material-review cause")),
        }
    }
}

/// A profile transition is provenance only: it preserves the exact prior
/// decision and its request so a later reader can classify that decision as
/// stale history. It never approves the new profile or discharges its Unknowns.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MaterialReviewProfileTransition {
    schema_version: u32,
    repository_id: String,
    work_item_id: String,
    previous_contract: serde_json::Value,
    previous_contract_digest: Digest,
    next_contract: serde_json::Value,
    next_contract_digest: Digest,
    previous_profile_digest: Digest,
    next_profile_digest: Digest,
    previous_request: MaterialReviewRequest,
    next_request: MaterialReviewRequest,
    previous_pointer: serde_json::Value,
    previous_receipt: MaterialInspectionReviewDecisionReceipt,
    transition_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MaterialReviewGateProjection {
    pub raw_scanner_unknowns: Vec<String>,
    pub material_manifest_digest: Option<Digest>,
    pub review_receipt_digest: Option<Digest>,
    pub review_assurance: Option<MaterialInspectionReviewAssurance>,
    pub effective_unknowns: Vec<String>,
    pub discharged_unknowns: Vec<String>,
    pub comparison_material: Option<CanonicalComparisonMaterialProof>,
    pub finding_codes: Vec<String>,
    pub blocked_by_finding: bool,
    pub projection_unavailable: bool,
    pub review_decision_available: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReviewedMaterialMemberIdentity {
    path: String,
    kind: String,
    changed_hunk_digest: Digest,
    after_blob_digest: Digest,
}

/// Private evidence from one stable, clean committed-source request. The
/// request builder reads HEAD:path blobs with bounded Git operations before
/// constructing these identities; comparison checkout text supplies no proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CanonicalComparisonMaterialProof {
    source_head: String,
    reviewed: Vec<ReviewedMaterialMemberIdentity>,
    clean: Vec<ReviewedMaterialMemberIdentity>,
}

impl CanonicalComparisonMaterialProof {
    fn from_request(request: &MaterialReviewRequest, reviewed: bool) -> Option<Self> {
        if request.blocked_by_finding
            || !request.finding_codes.is_empty()
            || (!request.raw_unknown_codes.is_empty() && !reviewed)
        {
            return None;
        }
        let member = |entry: &MaterialReviewEntry| {
            let blob = entry.after_blob_digest.as_ref()?;
            valid_digest(blob).then(|| ReviewedMaterialMemberIdentity {
                path: entry.path.clone(),
                kind: entry.kind.clone(),
                changed_hunk_digest: entry.changed_hunk_digest.clone(),
                after_blob_digest: blob.clone(),
            })
        };
        Some(Self {
            source_head: request.reviewed_source_head.clone(),
            reviewed: request
                .entries
                .iter()
                .filter(|entry| {
                    reviewed
                        && entry.reviewable
                        && entry.scanner_assessment == MaterialScannerAssessment::Unknown
                        && entry.unknown_cause
                            == Some(MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown)
                })
                .filter_map(member)
                .collect(),
            clean: request
                .entries
                .iter()
                .filter(|entry| {
                    entry.scanner_assessment == MaterialScannerAssessment::Clean
                        && entry.unknown_cause.is_none()
                        && entry.content_state == "text"
                        && entry.path.to_ascii_lowercase().ends_with(".rs")
                })
                .filter_map(member)
                .collect(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MaterialReviewReceiptState {
    Missing,
    Current(Box<MaterialInspectionReviewDecisionReceipt>),
    Stale,
}

#[derive(Debug, Error)]
pub enum MaterialReviewRequestError {
    #[error("Git observation failed: {0}")]
    Git(String),
    #[error("Contract identity is not bound to this repository")]
    ContractIdentity,
    #[error("non-.ai source has staged, unstaged, or untracked changes: {0}")]
    DirtySource(String),
    #[error("material source {path} is unavailable: {reason}")]
    SourceUnavailable { path: String, reason: String },
    #[error("material identity could not be computed: {0}")]
    Identity(String),
    #[error("material request exceeded {budget} budget (limit: {limit})")]
    BudgetExceeded { budget: &'static str, limit: u64 },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MaterialReviewDecisionValidationError {
    #[error("Contract material review profile is invalid: {0}")]
    InvalidProfile(String),
    #[error("material review decision is not enabled by the current Contract")]
    ReviewNotEnabled,
    #[error("material review request identity does not match the current Contract")]
    RequestIdentityMismatch,
    #[error("material review request digest does not match canonical request content")]
    RequestDigestMismatch,
    #[error("material review request contains a Finding")]
    FindingPresent,
    #[error("material review request contains an unknown outside the approved profile")]
    UnexpectedUnknown,
    #[error("material review request has no reviewable unknown")]
    NoReviewableUnknown,
    #[error("material review input schema or text fields are invalid")]
    InvalidInput,
    #[error("reviewerActor does not match the approved Contract profile")]
    ReviewerMismatch,
    #[error("authoritySource does not match the approved Contract profile")]
    AuthorityMismatch,
    #[error("evidence references must be nonempty, unique, and carry valid digests")]
    InvalidEvidenceReferences,
    #[error("Runtime recorder provenance or timestamp is invalid")]
    InvalidRuntimeProvenance,
    #[error("material review receipt could not be finalized: {0}")]
    ReceiptIntegrity(String),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestEntry<'a> {
    path: &'a str,
    kind: &'a str,
    content_state: &'a str,
    added_lines: &'a [String],
    added_line_origins: Vec<(usize, usize)>,
    removed_lines: &'a [String],
    changed_hunk_digest: &'a Digest,
    after_blob_digest: &'a Option<Digest>,
}

fn json_digest(value: &impl Serialize) -> Result<Digest, MaterialReviewRequestError> {
    digest_json(value).map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))
}

fn changed_hunk_digest(change: &ChangeEvidence) -> Result<Digest, MaterialReviewRequestError> {
    let origins = change
        .added_line_origins
        .iter()
        .map(|origin| (origin.after_line, origin.hunk_index))
        .collect::<Vec<_>>();
    json_digest(&(&change.added_lines, &origins, &change.removed_lines))
}

fn material_review_request_digest(
    request: &MaterialReviewRequest,
) -> Result<Digest, MaterialReviewRequestError> {
    let mut validity = serde_json::to_value(request)
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    let fields = validity
        .as_object_mut()
        .expect("typed request is an object");
    fields.remove("requestDigest");
    fields.remove("reviewedSourceHead");
    fields.remove("reviewDiagnostic");
    json_digest(&("ai-cockpit:material-review-request:v1", validity))
}

fn valid_digest(digest: &Digest) -> bool {
    digest.as_str().parse::<Digest>().is_ok()
}

fn valid_utc_runtime_timestamp(value: &str) -> bool {
    value.trim() == value
        && value.ends_with('Z')
        && chrono::DateTime::parse_from_rfc3339(value)
            .is_ok_and(|timestamp| timestamp.offset().local_minus_utc() == 0)
}

/// Validate a self-declared decision against the exact typed Contract and
/// canonical request, then produce its deterministic receipt. `recorded_by`
/// and `recorded_at` must come from the caller's Runtime context; this pure
/// function does not authenticate either real-world identity.
pub fn validate_material_review_decision(
    contract: &Contract,
    request: &MaterialReviewRequest,
    input: &MaterialInspectionReviewDecisionInput,
    recorded_by: &str,
    recorded_at: &str,
) -> Result<MaterialInspectionReviewDecisionReceipt, MaterialReviewDecisionValidationError> {
    let contract_digest = json_digest(contract)
        .map_err(|_| MaterialReviewDecisionValidationError::RequestIdentityMismatch)?;
    validate_material_review_decision_with_contract_digest(
        contract,
        &contract_digest,
        request,
        input,
        recorded_by,
        recorded_at,
    )
}

fn validate_material_review_decision_with_contract_digest(
    contract: &Contract,
    contract_digest: &Digest,
    request: &MaterialReviewRequest,
    input: &MaterialInspectionReviewDecisionInput,
    recorded_by: &str,
    recorded_at: &str,
) -> Result<MaterialInspectionReviewDecisionReceipt, MaterialReviewDecisionValidationError> {
    let profile = contract
        .material_inspection_review_profile()
        .map_err(MaterialReviewDecisionValidationError::InvalidProfile)?
        .ok_or(MaterialReviewDecisionValidationError::ReviewNotEnabled)?;
    if !contract
        .required_runtime_capabilities
        .iter()
        .any(|capability| capability == MATERIAL_INSPECTION_REVIEW_CAPABILITY)
        || !request.review_enabled
    {
        return Err(MaterialReviewDecisionValidationError::ReviewNotEnabled);
    }
    let profile_digest = json_digest(&profile)
        .map_err(|_| MaterialReviewDecisionValidationError::RequestIdentityMismatch)?;
    if request.schema_version != 1
        || request.repository_id != contract.repository_id
        || request.work_item_id != contract.work_item_id
        || request.contract_digest != *contract_digest
        || request.immutable_contract_base_revision != contract.base_revision
        || request.material_inspection_review_profile_digest.as_ref() != Some(&profile_digest)
    {
        return Err(MaterialReviewDecisionValidationError::RequestIdentityMismatch);
    }
    if request.blocked_by_finding
        || !request.finding_codes.is_empty()
        || request
            .entries
            .iter()
            .any(|entry| entry.scanner_assessment == MaterialScannerAssessment::Finding)
    {
        return Err(MaterialReviewDecisionValidationError::FindingPresent);
    }
    if request.raw_unknown_codes.len() != 1
        || request.raw_unknown_codes[0] != profile.permitted_unknown
    {
        return Err(MaterialReviewDecisionValidationError::UnexpectedUnknown);
    }
    let mut reviewable_unknowns = 0;
    for entry in &request.entries {
        if entry.scanner_assessment == MaterialScannerAssessment::Unknown {
            reviewable_unknowns += 1;
            if !entry.reviewable
                || entry.unknown_cause
                    != Some(MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown)
            {
                return Err(MaterialReviewDecisionValidationError::UnexpectedUnknown);
            }
        }
    }
    if reviewable_unknowns == 0 {
        return Err(MaterialReviewDecisionValidationError::NoReviewableUnknown);
    }
    if !valid_digest(&request.contract_digest)
        || !valid_digest(&request.source_snapshot_digest)
        || !valid_digest(&request.material_manifest_digest)
        || !valid_digest(&request.analysis_implementation_digest)
        || !valid_digest(&request.effective_policy_digest)
        || !request
            .material_inspection_review_profile_digest
            .as_ref()
            .is_some_and(valid_digest)
        || request.entries.iter().any(|entry| {
            !valid_digest(&entry.changed_hunk_digest)
                || entry
                    .after_blob_digest
                    .as_ref()
                    .is_some_and(|digest| !valid_digest(digest))
        })
    {
        return Err(MaterialReviewDecisionValidationError::RequestIdentityMismatch);
    }
    let canonical_request_digest = material_review_request_digest(request)
        .map_err(|_| MaterialReviewDecisionValidationError::RequestIdentityMismatch)?;
    if !valid_digest(&request.request_digest)
        || canonical_request_digest != request.request_digest
        || input.request_digest != request.request_digest
        || !valid_digest(&input.request_digest)
    {
        return Err(MaterialReviewDecisionValidationError::RequestDigestMismatch);
    }
    if input.schema_version != MATERIAL_INSPECTION_REVIEW_DECISION_SCHEMA_VERSION
        || input.assurance != MaterialInspectionReviewAssurance::SelfDeclared
        || profile.assurance != "self_declared"
        || input.rationale.trim().is_empty()
        || input.rationale.trim() != input.rationale
        || input.residual_risk.trim().is_empty()
        || input.residual_risk.trim() != input.residual_risk
        || input.reviewer_actor.trim().is_empty()
        || input.reviewer_actor.trim() != input.reviewer_actor
        || input.authority_source.trim().is_empty()
        || input.authority_source.trim() != input.authority_source
    {
        return Err(MaterialReviewDecisionValidationError::InvalidInput);
    }
    if input.reviewer_actor != profile.reviewer_actor {
        return Err(MaterialReviewDecisionValidationError::ReviewerMismatch);
    }
    if input.authority_source != profile.authority_source {
        return Err(MaterialReviewDecisionValidationError::AuthorityMismatch);
    }
    let mut evidence_refs = input.evidence_refs.clone();
    let mut evidence_paths = BTreeSet::new();
    if evidence_refs.is_empty()
        || evidence_refs.iter().any(|reference| {
            reference.path.trim().is_empty()
                || reference.path.trim() != reference.path
                || !valid_digest(&reference.digest)
                || !evidence_paths.insert(reference.path.clone())
        })
    {
        return Err(MaterialReviewDecisionValidationError::InvalidEvidenceReferences);
    }
    evidence_refs.sort_by(|left, right| left.path.cmp(&right.path));
    if recorded_by.trim().is_empty()
        || recorded_by.trim() != recorded_by
        || !valid_utc_runtime_timestamp(recorded_at)
    {
        return Err(MaterialReviewDecisionValidationError::InvalidRuntimeProvenance);
    }

    let mut receipt = MaterialInspectionReviewDecisionReceipt {
        schema_version: MATERIAL_INSPECTION_REVIEW_DECISION_SCHEMA_VERSION,
        repository_id: request.repository_id.clone(),
        work_item_id: request.work_item_id.clone(),
        contract_digest: request.contract_digest.clone(),
        material_manifest_digest: request.material_manifest_digest.clone(),
        profile_digest,
        request_digest: request.request_digest.clone(),
        reviewed_source_head: Some(request.reviewed_source_head.clone()),
        decision: MaterialInspectionReviewDecision::AcceptPermittedUnknowns,
        reviewer_actor: input.reviewer_actor.clone(),
        recorded_by: recorded_by.to_owned(),
        authority_source: input.authority_source.clone(),
        assurance: input.assurance,
        evidence_refs,
        rationale: input.rationale.clone(),
        residual_risk: input.residual_risk.clone(),
        recorded_at: recorded_at.to_owned(),
        receipt_digest: Digest::sha256_bytes(b"uncomputed"),
    };
    receipt.receipt_digest = receipt
        .canonical_digest()
        .map_err(MaterialReviewDecisionValidationError::ReceiptIntegrity)?;
    receipt
        .validate_integrity()
        .map_err(MaterialReviewDecisionValidationError::ReceiptIntegrity)?;
    Ok(receipt)
}

/// Domain-separated, length-delimited source bytes. The fixed path list is
/// compiled into the binary, so compiler output, host path, and binary SHA do
/// not affect the result. A source-byte change changes the digest even when
/// SCANNER_SEMANTIC_VERSION is accidentally left unchanged.
fn implementation_digest_from_sources(sources: &[(&str, &[u8])]) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(b"ai-cockpit:material-analysis-implementation:v1\0");
    for (path, bytes) in sources {
        hasher.update((path.len() as u64).to_le_bytes());
        hasher.update(path.as_bytes());
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    }
    Digest::sha256_bytes(&hasher.finalize())
}

fn analysis_implementation_digest() -> Digest {
    implementation_digest_from_sources(&[
        ("Cargo.lock", include_bytes!("../../../Cargo.lock")),
        (
            "crates/cockpit-core/src/lib.rs",
            include_bytes!("../../cockpit-core/src/lib.rs"),
        ),
        (
            "crates/cockpit-git/src/lib.rs",
            include_bytes!("../../cockpit-git/src/lib.rs"),
        ),
        (
            "crates/cockpit-protocol/src/lib.rs",
            include_bytes!("../../cockpit-protocol/src/lib.rs"),
        ),
        (
            "crates/cockpit-protocol/src/contract_amendment.rs",
            include_bytes!("../../cockpit-protocol/src/contract_amendment.rs"),
        ),
        (
            "crates/cockpit-repository/Cargo.toml",
            include_bytes!("../Cargo.toml"),
        ),
        (
            "crates/cockpit-repository/src/governance_controls.rs",
            include_bytes!("governance_controls.rs"),
        ),
        (
            "crates/cockpit-repository/src/lib.rs",
            include_bytes!("lib.rs"),
        ),
        (
            "crates/cockpit-repository/src/material_review.rs",
            include_bytes!("material_review.rs"),
        ),
        (
            "crates/cockpit-repository/src/rust_material.rs",
            include_bytes!("rust_material.rs"),
        ),
        (
            "crates/cockpit-repository/src/status_projection.rs",
            include_bytes!("status_projection.rs"),
        ),
    ])
}

fn git_output_error(output: &BoundedGitOutput) -> MaterialReviewRequestError {
    MaterialReviewRequestError::Git(String::from_utf8_lossy(&output.stderr).into_owned())
}

fn reject_nonstandard_index_flags(git: &GitRepository) -> Result<(), MaterialReviewRequestError> {
    let output = git
        .index_flags_bounded(MAX_PATCH_BYTES)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    if !output.success {
        return Err(git_output_error(&output));
    }
    for record in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        if record.len() < 2 {
            return Err(MaterialReviewRequestError::Identity(
                "malformed index flags".into(),
            ));
        }
        let (flag, path) = record.split_at(2);
        if path != b".ai" && !path.starts_with(b".ai/") && flag != b"H " {
            return Err(MaterialReviewRequestError::DirtySource(format!(
                "nonstandard index flag for {}",
                String::from_utf8_lossy(path)
            )));
        }
    }
    Ok(())
}

fn committed_blob_ids(
    git: &GitRepository,
    revision: &str,
) -> Result<BTreeMap<String, String>, MaterialReviewRequestError> {
    let output = git
        .committed_tree_bounded(revision, MAX_PATCH_BYTES)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    if !output.success {
        return Err(git_output_error(&output));
    }
    let mut objects = BTreeMap::new();
    for record in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let Some(tab) = record.iter().position(|byte| *byte == b'\t') else {
            return Err(MaterialReviewRequestError::Identity(
                "malformed committed tree entry".into(),
            ));
        };
        let metadata = &record[..tab];
        let path = &record[tab + 1..];
        let path = std::str::from_utf8(path)
            .map_err(|_| MaterialReviewRequestError::Identity("non-UTF-8 committed path".into()))?;
        if path == ".ai" || path.starts_with(".ai/") {
            continue;
        }
        let fields = std::str::from_utf8(metadata).map_err(|_| {
            MaterialReviewRequestError::Identity("malformed committed tree metadata".into())
        })?;
        let mut fields = fields.split_whitespace();
        let (Some(_mode), Some(kind), Some(id)) = (fields.next(), fields.next(), fields.next())
        else {
            return Err(MaterialReviewRequestError::Identity(
                "malformed committed tree metadata".into(),
            ));
        };
        if kind != "blob" {
            return Err(MaterialReviewRequestError::Identity(
                "non-blob committed material".into(),
            ));
        }
        objects.insert(path.to_owned(), id.to_owned());
    }
    Ok(objects)
}

fn no_hunk_blob_identity_is_proven(
    change: &cockpit_git::ChangeEvidence,
    base_blob_id: Option<&str>,
    head_blob_id: &str,
    deleted_base_blob_ids: &BTreeSet<String>,
    head_blob_is_empty: bool,
) -> bool {
    if !change.added_lines.is_empty() || !change.removed_lines.is_empty() {
        return true;
    }
    match change.kind {
        ChangeKind::Modified => base_blob_id == Some(head_blob_id),
        ChangeKind::Added => head_blob_is_empty || deleted_base_blob_ids.contains(head_blob_id),
        ChangeKind::Renamed => {
            base_blob_id == Some(head_blob_id) || deleted_base_blob_ids.contains(head_blob_id)
        }
        _ => false,
    }
}

fn committed_blob(
    git: &GitRepository,
    id: &str,
    max_output_bytes: usize,
    aggregate_budget_limited: bool,
) -> Result<Vec<u8>, MaterialReviewRequestError> {
    let output = git
        .blob_bounded(id, max_output_bytes)
        .map_err(|error| match error {
            GitError::OutputLimitExceeded { .. } if aggregate_budget_limited => {
                MaterialReviewRequestError::BudgetExceeded {
                    budget: "total_blob_bytes",
                    limit: MAX_MATERIAL_TOTAL_BLOB_BYTES,
                }
            }
            GitError::OutputLimitExceeded { .. } => MaterialReviewRequestError::Identity(
                "committed source exceeds per-blob request budget".into(),
            ),
            error => MaterialReviewRequestError::Git(error.to_string()),
        })?;
    if !output.success {
        return Err(git_output_error(&output));
    }
    Ok(output.stdout)
}

fn verify_checkout_source(
    root: &Path,
    relative_path: &str,
) -> Result<(), MaterialReviewRequestError> {
    let display_path = root.join(relative_path);
    let file = open_checkout_source_nofollow(root, Path::new(relative_path)).map_err(|error| {
        MaterialReviewRequestError::SourceUnavailable {
            path: display_path.display().to_string(),
            reason: error.to_string(),
        }
    })?;
    let opened =
        file.metadata()
            .map_err(|error| MaterialReviewRequestError::SourceUnavailable {
                path: display_path.display().to_string(),
                reason: error.to_string(),
            })?;
    if !opened.is_file() {
        return Err(MaterialReviewRequestError::SourceUnavailable {
            path: display_path.display().to_string(),
            reason: "symlink or non-regular source".into(),
        });
    }
    if opened.len() > MAX_MATERIAL_BLOB_BYTES {
        return Err(MaterialReviewRequestError::SourceUnavailable {
            path: display_path.display().to_string(),
            reason: "source exceeds bounded request budget".into(),
        });
    }
    let copied = io::copy(
        &mut io::Read::take(file, MAX_MATERIAL_BLOB_BYTES + 1),
        &mut io::sink(),
    )
    .map_err(|error| MaterialReviewRequestError::SourceUnavailable {
        path: display_path.display().to_string(),
        reason: error.to_string(),
    })?;
    let final_file =
        open_checkout_source_nofollow(root, Path::new(relative_path)).map_err(|error| {
            MaterialReviewRequestError::SourceUnavailable {
                path: display_path.display().to_string(),
                reason: error.to_string(),
            }
        })?;
    let final_metadata =
        final_file
            .metadata()
            .map_err(|error| MaterialReviewRequestError::SourceUnavailable {
                path: display_path.display().to_string(),
                reason: error.to_string(),
            })?;
    #[cfg(unix)]
    let same_file = {
        use std::os::unix::fs::MetadataExt;
        opened.dev() == final_metadata.dev() && opened.ino() == final_metadata.ino()
    };
    #[cfg(not(unix))]
    let same_file = opened.len() == final_metadata.len();
    if copied != opened.len()
        || final_metadata.len() != opened.len()
        || !final_metadata.is_file()
        || !same_file
    {
        return Err(MaterialReviewRequestError::SourceUnavailable {
            path: display_path.display().to_string(),
            reason: "source changed during bounded read".into(),
        });
    }
    Ok(())
}

#[cfg(unix)]
fn open_checkout_source_nofollow(root: &Path, relative_path: &Path) -> io::Result<File> {
    use std::{
        ffi::CString,
        os::{
            fd::{AsRawFd, FromRawFd, OwnedFd},
            unix::ffi::OsStrExt,
        },
        path::Component,
    };

    let root = fs::canonicalize(root)?;
    if !root.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "checkout root is not absolute",
        ));
    }
    let root_fd = unsafe {
        libc::open(
            c"/".as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
        )
    };
    if root_fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let mut directory = unsafe { OwnedFd::from_raw_fd(root_fd) };
    for component in root.components() {
        match component {
            Component::RootDir | Component::CurDir => continue,
            Component::Normal(name) => {
                let name = CString::new(name.as_bytes())
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NUL path"))?;
                let next_fd = unsafe {
                    libc::openat(
                        directory.as_raw_fd(),
                        name.as_ptr(),
                        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                    )
                };
                if next_fd < 0 {
                    return Err(io::Error::last_os_error());
                }
                directory = unsafe { OwnedFd::from_raw_fd(next_fd) };
            }
            Component::ParentDir | Component::Prefix(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "checkout source path escapes root",
                ));
            }
        }
    }
    let mut components = relative_path.components().peekable();
    while let Some(component) = components.next() {
        let Component::Normal(name) = component else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "checkout source path is not a normalized relative path",
            ));
        };
        let name = CString::new(name.as_bytes())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NUL path"))?;
        let is_final = components.peek().is_none();
        let flags = if is_final {
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK
        } else {
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW
        };
        let next_fd = unsafe { libc::openat(directory.as_raw_fd(), name.as_ptr(), flags) };
        if next_fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let next = unsafe { OwnedFd::from_raw_fd(next_fd) };
        if is_final {
            return Ok(File::from(next));
        }
        directory = next;
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "checkout source path is empty",
    ))
}

#[cfg(windows)]
fn open_checkout_source_nofollow(root: &Path, relative_path: &Path) -> io::Result<File> {
    use std::path::Component;

    let mut components = relative_path.components().peekable();
    if components
        .clone()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "checkout source path is not a normalized relative path",
        ));
    }
    if components.peek().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "checkout source path is empty",
        ));
    }

    let canonical_root = fs::canonicalize(root)?;
    let mut parent = Dir::open_ambient_dir(&canonical_root, cap_std::ambient_authority())?;
    let root_handle = parent.try_clone()?.into_std_file();
    let canonical_root_handle_path = windows_handle_path(&root_handle)?;
    if canonical_root_handle_path != canonical_root {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "opened checkout root no longer matches its canonical path",
        ));
    }
    ensure_windows_handle_is_contained(&canonical_root_handle_path, &root_handle)?;
    if !root_handle.metadata()?.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "canonical checkout root is not a directory",
        ));
    }

    while let Some(Component::Normal(component)) = components.next() {
        if components.peek().is_some() {
            let directory = parent.open_dir_nofollow(component)?;
            let directory_handle = directory.try_clone()?.into_std_file();
            ensure_windows_handle_is_contained(&canonical_root_handle_path, &directory_handle)?;
            if !directory_handle.metadata()?.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "checkout source parent is not a directory",
                ));
            }
            parent = directory;
        } else {
            let mut options = CapOpenOptions::new();
            options.read(true).follow(FollowSymlinks::No);
            let file = parent.open_with(component, &options)?.into_std();
            ensure_windows_handle_is_contained(&canonical_root_handle_path, &file)?;
            return Ok(file);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "checkout source path is empty",
    ))
}

#[cfg(windows)]
fn ensure_windows_handle_is_contained(root: &Path, file: &File) -> io::Result<()> {
    use std::{mem::size_of, os::windows::io::AsRawHandle};
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_ATTRIBUTE_TAG_INFO, FileAttributeTagInfo,
        GetFileInformationByHandleEx,
    };

    let mut attributes = FILE_ATTRIBUTE_TAG_INFO::default();
    let succeeded = unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileAttributeTagInfo,
            (&mut attributes as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
        )
    };
    if succeeded == 0 {
        return Err(io::Error::last_os_error());
    }
    if attributes.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "checkout source path contains a reparse point",
        ));
    }

    let opened_path = windows_handle_path(file)?;
    if opened_path.strip_prefix(root).is_err() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "opened checkout source handle escapes canonical root",
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn windows_handle_path(file: &File) -> io::Result<std::path::PathBuf> {
    use std::os::{windows::ffi::OsStringExt, windows::io::AsRawHandle};
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_NAME_NORMALIZED, GetFinalPathNameByHandleW, VOLUME_NAME_DOS,
    };

    let mut buffer = vec![0_u16; 32_768];
    let length = unsafe {
        GetFinalPathNameByHandleW(
            file.as_raw_handle(),
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            FILE_NAME_NORMALIZED | VOLUME_NAME_DOS,
        )
    };
    if length == 0 {
        return Err(io::Error::last_os_error());
    }
    if length as usize >= buffer.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "opened checkout source path exceeds Windows API bound",
        ));
    }
    Ok(std::path::PathBuf::from(std::ffi::OsString::from_wide(
        &buffer[..length as usize],
    )))
}

#[cfg(not(any(unix, windows)))]
fn open_checkout_source_nofollow(root: &Path, relative_path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).open(root.join(relative_path))
}

fn read_active_contract_document(
    root: &Path,
    work_item_id: &str,
) -> Result<super::CanonicalContractDocument, MaterialReviewRequestError> {
    let root_dir = Dir::open_ambient_dir(root, ambient_authority())
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    let ai_path = root.join(".ai");
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &ai_path)
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    let work_items_path = ai_path.join("work-items");
    let work_items = open_cap_directory_nofollow_strict(&ai, "work-items", &work_items_path)
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    let active_path = work_items_path.join("active");
    let active = open_cap_directory_nofollow_strict(&work_items, "active", &active_path)
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    let name = format!("{work_item_id}.contract.json");
    let path = active_path.join(&name);
    let bytes = read_cap_file_nofollow_bounded(&active, &name, &path, MAX_ACTIVE_CONTRACT_BYTES)
        .map_err(|error| MaterialReviewRequestError::SourceUnavailable {
            path: path.display().to_string(),
            reason: error.to_string(),
        })?;
    super::parse_contract_document(&bytes, &path)
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))
}

pub fn material_review_request(
    root: &Path,
    contract: &Contract,
) -> Result<MaterialReviewRequest, MaterialReviewRequestError> {
    let contract_digest = json_digest(contract)?;
    material_review_request_with_contract_digest(root, contract, &contract_digest)
}

fn material_review_request_with_contract_digest(
    root: &Path,
    contract: &Contract,
    contract_digest: &Digest,
) -> Result<MaterialReviewRequest, MaterialReviewRequestError> {
    let git = GitRepository::discover(root)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    let root = git.root();
    if contract.repository_id != repository_id(root).to_string() {
        return Err(MaterialReviewRequestError::ContractIdentity);
    }
    reject_nonstandard_index_flags(&git)?;
    let working = git
        .source_snapshot_bounded(MAX_PATCH_BYTES)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    let dirty = working
        .changed_paths
        .iter()
        .filter(|path| *path != ".ai" && !path.starts_with(".ai/"))
        .cloned()
        .collect::<Vec<_>>();
    if !dirty.is_empty() {
        return Err(MaterialReviewRequestError::DirtySource(dirty.join(", ")));
    }
    let mut snapshot = git
        .source_snapshot_against_bounded(&contract.base_revision, MAX_PATCH_BYTES)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    let head = snapshot
        .head
        .clone()
        .ok_or_else(|| MaterialReviewRequestError::Identity("committed HEAD is required".into()))?;
    if working.head.as_deref() != Some(head.as_str())
        || working.source_tree_digest != snapshot.source_tree_digest.as_deref().unwrap_or_default()
    {
        return Err(MaterialReviewRequestError::Identity(
            "source changed during request observation".into(),
        ));
    }
    let is_ancestor = git
        .is_ancestor_bounded(&contract.base_revision, &head, MAX_PATCH_BYTES)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    if !is_ancestor {
        return Err(MaterialReviewRequestError::Identity(
            "Contract base is not an ancestor of committed HEAD".into(),
        ));
    }
    snapshot
        .change_evidence
        .sort_by(|left, right| left.path.cmp(&right.path));
    let changed_file_count = snapshot
        .change_evidence
        .iter()
        .filter(|change| change.path != ".ai" && !change.path.starts_with(".ai/"))
        .count();
    if changed_file_count > MAX_MATERIAL_CHANGED_FILES {
        return Err(MaterialReviewRequestError::BudgetExceeded {
            budget: "changed_file_count",
            limit: MAX_MATERIAL_CHANGED_FILES as u64,
        });
    }
    let source_snapshot_digest = snapshot
        .source_tree_digest
        .as_deref()
        .ok_or_else(|| {
            MaterialReviewRequestError::Identity("source tree digest is missing".into())
        })?
        .parse::<Digest>()
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    let base_objects = committed_blob_ids(&git, &contract.base_revision)?;
    let committed_objects = committed_blob_ids(&git, &head)?;
    let deleted_base_blob_ids = snapshot
        .change_evidence
        .iter()
        .filter(|change| change.kind == ChangeKind::Deleted)
        .filter_map(|change| base_objects.get(&change.path).cloned())
        .collect::<BTreeSet<_>>();
    let mut committed_digests = BTreeMap::new();
    let mut total_blob_bytes = 0_u64;
    let mut total_text_bytes = 0_u64;
    for change in snapshot
        .change_evidence
        .iter_mut()
        .filter(|change| change.path != ".ai" && !change.path.starts_with(".ai/"))
    {
        if change.kind == ChangeKind::Deleted {
            continue;
        }
        let id = committed_objects.get(&change.path).ok_or_else(|| {
            MaterialReviewRequestError::SourceUnavailable {
                path: change.path.clone(),
                reason: "source has no committed blob".into(),
            }
        })?;
        let remaining_blob_bytes = MAX_MATERIAL_TOTAL_BLOB_BYTES.saturating_sub(total_blob_bytes);
        if remaining_blob_bytes == 0 {
            return Err(MaterialReviewRequestError::BudgetExceeded {
                budget: "total_blob_bytes",
                limit: MAX_MATERIAL_TOTAL_BLOB_BYTES,
            });
        }
        let blob_limit = MAX_MATERIAL_BLOB_BYTES.min(remaining_blob_bytes);
        let bytes = committed_blob(
            &git,
            id,
            blob_limit as usize,
            remaining_blob_bytes < MAX_MATERIAL_BLOB_BYTES,
        )?;
        total_blob_bytes = total_blob_bytes.saturating_add(bytes.len() as u64);
        // The committed object supplies the digest and scan bytes. Checkout
        // filters can normalize line endings, so working-tree bytes are never
        // compared with a blob ID.
        committed_digests.insert(change.path.clone(), Digest::sha256_bytes(&bytes));
        if matches!(
            change.content_state,
            ChangeContentState::Binary | ChangeContentState::TooLarge
        ) {
            continue;
        }
        if bytes.len() > 4 * 1024 * 1024 {
            change.content_state = ChangeContentState::TooLarge;
            change.after_text = None;
            continue;
        }
        if change.content_state == ChangeContentState::Unavailable
            && change.added_lines.is_empty()
            && change.removed_lines.is_empty()
            && !no_hunk_blob_identity_is_proven(
                change,
                base_objects.get(&change.path).map(String::as_str),
                id,
                &deleted_base_blob_ids,
                bytes.is_empty(),
            )
        {
            continue;
        }
        match String::from_utf8(bytes) {
            Ok(text) => {
                if change.content_state == ChangeContentState::Unavailable
                    && text.len() > MAX_CHANGE_TEXT_BYTES
                {
                    change.content_state = ChangeContentState::TooLarge;
                    change.after_text = None;
                } else {
                    let next_text_bytes = total_text_bytes.saturating_add(text.len() as u64);
                    if next_text_bytes > MAX_MATERIAL_TOTAL_TEXT_BYTES {
                        return Err(MaterialReviewRequestError::BudgetExceeded {
                            budget: "total_text_bytes",
                            limit: MAX_MATERIAL_TOTAL_TEXT_BYTES,
                        });
                    }
                    change.content_state = ChangeContentState::Text;
                    change.after_text = Some(text);
                    total_text_bytes = next_text_bytes;
                }
            }
            Err(_) => {
                change.content_state = ChangeContentState::Binary;
                change.after_text = None;
            }
        }
    }
    let (signals, rust_diagnostics) = derive_governance_signals_with_diagnostics(&snapshot);
    let mut entries = Vec::new();
    let mut manifest_hasher = Sha256::new();
    manifest_hasher.update(b"ai-cockpit:material-manifest:v1\0");
    for change in snapshot
        .change_evidence
        .iter()
        .filter(|change| change.path != ".ai" && !change.path.starts_with(".ai/"))
    {
        let after_blob_digest = if change.kind == ChangeKind::Deleted {
            None
        } else {
            verify_checkout_source(root, &change.path)?;
            committed_digests.get(&change.path).cloned()
        };
        let kind = format!("{:?}", change.kind).to_ascii_lowercase();
        let content_state = format!("{:?}", change.content_state).to_ascii_lowercase();
        let origins = change
            .added_line_origins
            .iter()
            .map(|origin| (origin.after_line, origin.hunk_index))
            .collect::<Vec<_>>();
        let changed_hunk_digest = changed_hunk_digest(change)?;
        let diagnosis = rust_diagnostics.get(&change.path);
        let scanner_assessment = if !matches!(
            change.content_state,
            ChangeContentState::Text | ChangeContentState::Deleted
        ) {
            MaterialScannerAssessment::Unknown
        } else if let Some(diagnosis) = diagnosis {
            match diagnosis.assessment {
                RustMaterialAssessment::Clean => MaterialScannerAssessment::Clean,
                RustMaterialAssessment::Finding => MaterialScannerAssessment::Finding,
                RustMaterialAssessment::Unknown => MaterialScannerAssessment::Unknown,
            }
        } else {
            let added_text = change.added_lines.join("\n");
            let material_text = if change.added_lines.is_empty() && change.kind == ChangeKind::Added
            {
                change.after_text.as_deref().unwrap_or("")
            } else {
                &added_text
            };
            if contains_strong_instruction_injection(material_text) {
                MaterialScannerAssessment::Finding
            } else {
                MaterialScannerAssessment::Clean
            }
        };
        let complete_provenance = change.added_lines.len() == change.added_line_origins.len();
        let unknown_cause = if diagnosis
            .is_some_and(|diagnosis| diagnosis.assessment == RustMaterialAssessment::Unknown)
            && !complete_provenance
        {
            Some(MaterialUnknownCause::InvalidProvenance)
        } else if let Some(diagnosis) = diagnosis {
            diagnosis.unknown_cause
        } else if !matches!(
            change.content_state,
            ChangeContentState::Text | ChangeContentState::Deleted
        ) {
            Some(match change.content_state {
                ChangeContentState::Binary => MaterialUnknownCause::NonTextMaterial,
                ChangeContentState::TooLarge => MaterialUnknownCause::SourceOverBudget,
                _ => MaterialUnknownCause::MissingCompleteSource,
            })
        } else {
            None
        };
        let reviewable =
            unknown_cause == Some(MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown);
        let entry = MaterialReviewEntry {
            path: change.path.clone(),
            kind,
            content_state,
            scanner_assessment,
            unknown_cause,
            reviewable,
            changed_hunk_digest,
            after_blob_digest,
        };
        let manifest_entry = ManifestEntry {
            path: &entry.path,
            kind: &entry.kind,
            content_state: &entry.content_state,
            added_lines: &change.added_lines,
            added_line_origins: origins,
            removed_lines: &change.removed_lines,
            changed_hunk_digest: &entry.changed_hunk_digest,
            after_blob_digest: &entry.after_blob_digest,
        };
        let bytes = serde_json::to_vec(&manifest_entry)
            .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
        manifest_hasher.update((bytes.len() as u64).to_le_bytes());
        manifest_hasher.update(bytes);
        entries.push(entry);
    }
    let patch_digest = snapshot
        .diff_digest
        .parse::<Digest>()
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    manifest_hasher.update(patch_digest.as_str().as_bytes());
    let material_manifest_digest = Digest::sha256_bytes(&manifest_hasher.finalize());
    let policy = effective_policy_for_contract(root, contract)
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    let profile = contract
        .material_inspection_review_profile()
        .map_err(MaterialReviewRequestError::Identity)?;
    let profile_digest = profile.as_ref().map(json_digest).transpose()?;
    let review_enabled = profile.is_some()
        && contract
            .required_runtime_capabilities
            .iter()
            .any(|capability| capability == MATERIAL_INSPECTION_REVIEW_CAPABILITY);
    let finding_codes = signals.findings;
    let blocked_by_finding = !finding_codes.is_empty();
    let mut request = MaterialReviewRequest {
        schema_version: 1,
        repository_id: contract.repository_id.clone(),
        work_item_id: contract.work_item_id.clone(),
        contract_digest: contract_digest.clone(),
        immutable_contract_base_revision: contract.base_revision.clone(),
        source_snapshot_digest,
        material_manifest_digest,
        scanner_semantic_version: SCANNER_SEMANTIC_VERSION.into(),
        analysis_implementation_digest: analysis_implementation_digest(),
        analysis_target_semantic_profile: ANALYSIS_TARGET_SEMANTIC_PROFILE.into(),
        material_inspection_review_profile_digest: profile_digest,
        effective_policy_digest: json_digest(&policy)?,
        entries,
        raw_unknown_codes: signals.unknowns,
        finding_codes,
        blocked_by_finding,
        review_enabled,
        review_diagnostic: (!review_enabled).then(|| "material_review_not_enabled".into()),
        request_digest: Digest::sha256_bytes(b"uncomputed"),
        reviewed_source_head: head,
    };
    request.request_digest = material_review_request_digest(&request)?;
    let final_snapshot = git
        .source_snapshot_bounded(MAX_PATCH_BYTES)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    reject_nonstandard_index_flags(&git)?;
    let final_dirty = final_snapshot
        .changed_paths
        .iter()
        .any(|path| path != ".ai" && !path.starts_with(".ai/"));
    if final_dirty
        || final_snapshot.head.as_deref() != Some(request.reviewed_source_head.as_str())
        || final_snapshot.source_tree_digest != working.source_tree_digest
    {
        return Err(MaterialReviewRequestError::Identity(
            "source changed during request observation".into(),
        ));
    }
    Ok(request)
}

/// Recompute the canonical Contract-base material projection and validate a
/// previously recorded decision against its immutable sidecar and Summary
/// pointer. This is read-only and fails closed when source material is dirty.
pub(crate) fn material_review_gate_projection(
    root: &Path,
    contract: &Contract,
    summary_path: &Path,
) -> Result<MaterialReviewGateProjection, ObserverError> {
    let document =
        read_active_contract_document(root, &contract.work_item_id).map_err(|error| {
            ObserverError::State {
                path: root.to_path_buf(),
                message: format!("material-review projection unavailable: {error}"),
            }
        })?;
    if document.contract != *contract {
        return Err(ObserverError::State {
            path: root.to_path_buf(),
            message: "material-review Contract changed during projection".into(),
        });
    }
    material_review_gate_projection_with_contract_digest(
        root,
        contract,
        &document.digest,
        summary_path,
    )
}

pub(crate) fn material_review_gate_projection_with_contract_digest(
    root: &Path,
    contract: &Contract,
    contract_digest: &Digest,
    summary_path: &Path,
) -> Result<MaterialReviewGateProjection, ObserverError> {
    let request = material_review_request_with_contract_digest(root, contract, contract_digest)
        .map_err(|error| ObserverError::State {
            path: root.to_path_buf(),
            message: format!("material-review projection unavailable: {error}"),
        })?;
    let blocked_by_finding = request.blocked_by_finding || !request.finding_codes.is_empty();
    let (receipt, stale_receipt) =
        match read_material_review_receipt(root, contract, contract_digest, &request, summary_path)
        {
            Ok(MaterialReviewReceiptState::Missing) => (None, false),
            Ok(MaterialReviewReceiptState::Current(receipt)) => (Some(*receipt), false),
            Ok(MaterialReviewReceiptState::Stale) => (None, true),
            Err(_error) => {
                let mut effective_unknowns = request.raw_unknown_codes.clone();
                effective_unknowns.push("material_review_receipt_invalid".into());
                effective_unknowns.sort();
                effective_unknowns.dedup();
                return Ok(MaterialReviewGateProjection {
                    raw_scanner_unknowns: request.raw_unknown_codes,
                    material_manifest_digest: Some(request.material_manifest_digest),
                    review_receipt_digest: None,
                    review_assurance: None,
                    effective_unknowns,
                    discharged_unknowns: Vec::new(),
                    comparison_material: None,
                    finding_codes: request.finding_codes,
                    blocked_by_finding,
                    projection_unavailable: false,
                    review_decision_available: false,
                });
            }
        };
    let mut effective_unknowns = request.raw_unknown_codes.clone();
    let unknown_entries = request
        .entries
        .iter()
        .filter(|entry| entry.scanner_assessment == MaterialScannerAssessment::Unknown)
        .collect::<Vec<_>>();
    let has_reviewable_unknown = unknown_entries.iter().any(|entry| entry.reviewable);
    if stale_receipt && has_reviewable_unknown {
        effective_unknowns.push("material_review_receipt_stale".into());
    }
    let all_unknowns_reviewable = unknown_entries.iter().all(|entry| entry.reviewable);
    let reviewed_unknown_members = unknown_entries
        .iter()
        .filter_map(|entry| {
            Some(ReviewedMaterialMemberIdentity {
                path: entry.path.clone(),
                kind: entry.kind.clone(),
                changed_hunk_digest: entry.changed_hunk_digest.clone(),
                after_blob_digest: entry.after_blob_digest.clone()?,
            })
        })
        .collect::<Vec<_>>();
    let all_unknowns_have_blob_proof = reviewed_unknown_members.len() == unknown_entries.len();
    let permitted_unknown_set = request.raw_unknown_codes.len() == 1
        && request.raw_unknown_codes[0] == "repository_material_inspection_unavailable";
    let review_decision_available = request.review_enabled
        && receipt.is_none()
        && !blocked_by_finding
        && has_reviewable_unknown
        && all_unknowns_reviewable
        && all_unknowns_have_blob_proof
        && permitted_unknown_set;
    let may_discharge = request.review_enabled
        && receipt.is_some()
        && !blocked_by_finding
        && has_reviewable_unknown
        && all_unknowns_reviewable
        && all_unknowns_have_blob_proof
        && permitted_unknown_set;
    if may_discharge {
        effective_unknowns
            .retain(|unknown| unknown != "repository_material_inspection_unavailable");
    }
    effective_unknowns.sort();
    effective_unknowns.dedup();
    // Capture both proof classes from this exact request, after its receipt
    // validation. No second request or manifest-only identity join is used.
    let comparison_material =
        CanonicalComparisonMaterialProof::from_request(&request, may_discharge);
    Ok(MaterialReviewGateProjection {
        raw_scanner_unknowns: request.raw_unknown_codes,
        material_manifest_digest: Some(request.material_manifest_digest),
        review_receipt_digest: receipt
            .as_ref()
            .map(|receipt| receipt.receipt_digest.clone()),
        review_assurance: receipt.map(|receipt| receipt.assurance),
        effective_unknowns,
        discharged_unknowns: if may_discharge {
            vec!["repository_material_inspection_unavailable".into()]
        } else {
            Vec::new()
        },
        comparison_material,
        finding_codes: request.finding_codes,
        blocked_by_finding,
        projection_unavailable: false,
        review_decision_available,
    })
}

/// Jointly cover exact reviewed syntax members and complete canonical Clean
/// blobs, then inspect every residual comparison member. Unknowns are never
/// accepted merely because another member has a valid review receipt.
pub(crate) fn preserve_unreviewed_comparison_material_unknowns(
    comparison_snapshot: &RepositorySnapshot,
    projection: &mut MaterialReviewGateProjection,
) {
    const MATERIAL_UNKNOWN: &str = "repository_material_inspection_unavailable";
    let Some(proof) = projection.comparison_material.as_ref() else {
        return;
    };
    if comparison_snapshot.head.as_deref() != Some(proof.source_head.as_str()) {
        preserve_material_unknown(
            &mut projection.discharged_unknowns,
            &mut projection.effective_unknowns,
        );
        return;
    }
    let (signals, _) = derive_governance_signals_with_diagnostics(comparison_snapshot);
    if !signals.findings.is_empty() {
        preserve_material_unknown(
            &mut projection.discharged_unknowns,
            &mut projection.effective_unknowns,
        );
        return;
    }
    let mut residual = comparison_snapshot.clone();
    residual.change_evidence.retain(|change| {
        if change.content_state != ChangeContentState::Text {
            return true;
        }
        let diagnosis = crate::rust_material::diagnose_rust_material(change);
        let reviewed = matches!(
            diagnosis.unknown_cause,
            Some(
                MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown
                    | MaterialUnknownCause::MissingCompleteSource
            )
        ) && proof
            .reviewed
            .iter()
            .any(|member| comparison_change_matches_member(change, member));
        let clean = matches!(
            change.kind,
            ChangeKind::Added | ChangeKind::Modified | ChangeKind::Renamed | ChangeKind::Copied
        ) && diagnosis.unknown_cause
            == Some(MaterialUnknownCause::MissingCompleteSource)
            && proof
                .clean
                .iter()
                .any(|member| comparison_change_matches_member(change, member));
        !(reviewed || clean)
    });
    let (remaining, _) = derive_governance_signals_with_diagnostics(&residual);
    if remaining
        .unknowns
        .iter()
        .any(|unknown| unknown == MATERIAL_UNKNOWN)
        || !remaining.findings.is_empty()
    {
        preserve_material_unknown(
            &mut projection.discharged_unknowns,
            &mut projection.effective_unknowns,
        );
    } else if !projection
        .effective_unknowns
        .iter()
        .any(|unknown| unknown == MATERIAL_UNKNOWN)
    {
        projection.discharged_unknowns.push(MATERIAL_UNKNOWN.into());
        projection.discharged_unknowns.sort();
        projection.discharged_unknowns.dedup();
    }
}

fn comparison_change_matches_member(
    change: &ChangeEvidence,
    member: &ReviewedMaterialMemberIdentity,
) -> bool {
    // With the same HEAD, this exact path identifies the blob read by the
    // canonical builder. Validate its digest without manufacturing a second
    // comparison digest from checkout text or a copy of the expected value.
    valid_digest(&member.after_blob_digest)
        && change.path == member.path
        && format!("{:?}", change.kind).to_ascii_lowercase() == member.kind
        && changed_hunk_digest(change).is_ok_and(|digest| digest == member.changed_hunk_digest)
}

#[cfg(test)]
pub(crate) fn clean_comparison_is_covered_for_test(
    comparison: &RepositorySnapshot,
    request: &MaterialReviewRequest,
) -> bool {
    let Some(proof) = CanonicalComparisonMaterialProof::from_request(request, false) else {
        return false;
    };
    if comparison.head.as_deref() != Some(proof.source_head.as_str()) {
        return false;
    }
    let mut projection = MaterialReviewGateProjection {
        raw_scanner_unknowns: Vec::new(),
        material_manifest_digest: Some(request.material_manifest_digest.clone()),
        review_receipt_digest: None,
        review_assurance: None,
        effective_unknowns: Vec::new(),
        discharged_unknowns: Vec::new(),
        comparison_material: Some(proof),
        finding_codes: Vec::new(),
        blocked_by_finding: false,
        projection_unavailable: false,
        review_decision_available: false,
    };
    preserve_unreviewed_comparison_material_unknowns(comparison, &mut projection);
    projection
        .discharged_unknowns
        .iter()
        .any(|u| u == "repository_material_inspection_unavailable")
}

fn preserve_material_unknown(
    discharged_unknowns: &mut Vec<String>,
    effective_unknowns: &mut Vec<String>,
) {
    const MATERIAL_UNKNOWN: &str = "repository_material_inspection_unavailable";
    discharged_unknowns.retain(|unknown| unknown != MATERIAL_UNKNOWN);
    effective_unknowns.push(MATERIAL_UNKNOWN.into());
    effective_unknowns.sort();
    effective_unknowns.dedup();
}

pub(crate) fn apply_material_review_gate_to_decision(
    root: &Path,
    contract: &Contract,
    summary_path: &Path,
    decision: &mut cockpit_core::GovernanceDecision,
) -> MaterialReviewGateProjection {
    let projection = match material_review_gate_projection(root, contract, summary_path) {
        Ok(projection) => projection,
        Err(_) => MaterialReviewGateProjection {
            raw_scanner_unknowns: Vec::new(),
            material_manifest_digest: None,
            review_receipt_digest: None,
            review_assurance: None,
            effective_unknowns: vec!["material_review_projection_unavailable".into()],
            discharged_unknowns: Vec::new(),
            comparison_material: None,
            finding_codes: Vec::new(),
            blocked_by_finding: false,
            projection_unavailable: true,
            review_decision_available: false,
        },
    };
    let projection = apply_material_review_projection_to_decision(projection, decision);
    if projection.blocked_by_finding {
        decision.state = cockpit_core::DecisionState::Red;
        decision.outcome_state = "not_ready".into();
        decision.review_state = Some("blocked".into());
    } else if !decision.unknowns.is_empty() && decision.state == cockpit_core::DecisionState::Green
    {
        decision.state = cockpit_core::DecisionState::Yellow;
        decision.outcome_state = "verification_pending".into();
        decision.review_state = Some("verification_pending".into());
    }
    projection
}

pub(crate) fn apply_material_review_projection_to_decision(
    projection: MaterialReviewGateProjection,
    decision: &mut cockpit_core::GovernanceDecision,
) -> MaterialReviewGateProjection {
    decision
        .unknowns
        .retain(|unknown| !projection.discharged_unknowns.contains(unknown));
    decision
        .unknowns
        .extend(projection.effective_unknowns.iter().cloned());
    if projection.projection_unavailable {
        decision
            .blockers
            .push("material_review_projection_unavailable".into());
    }
    if projection.blocked_by_finding {
        decision
            .blockers
            .extend(projection.finding_codes.iter().cloned());
    }
    decision.unknowns.sort();
    decision.unknowns.dedup();
    decision.blockers.sort();
    decision.blockers.dedup();
    decision.raw_scanner_unknowns = projection.raw_scanner_unknowns.clone();
    decision.material_manifest_digest = projection.material_manifest_digest.clone();
    decision.review_receipt_digest = projection.review_receipt_digest.clone();
    decision.review_assurance = projection
        .review_assurance
        .map(|assurance| match assurance {
            MaterialInspectionReviewAssurance::SelfDeclared => "self_declared".into(),
        });
    decision.effective_unknowns = decision.unknowns.clone();
    projection
}

pub(crate) fn require_material_review_gate(
    root: &Path,
    contract: &Contract,
    summary_path: &Path,
    boundary: &str,
) -> Result<MaterialReviewGateProjection, ObserverError> {
    let projection = material_review_gate_projection(root, contract, summary_path)?;
    if projection.blocked_by_finding {
        return Err(ObserverError::State {
            path: summary_path.to_path_buf(),
            message: format!("{boundary} is blocked by a canonical material Finding"),
        });
    }
    if !projection.effective_unknowns.is_empty() {
        return Err(ObserverError::State {
            path: summary_path.to_path_buf(),
            message: format!(
                "{boundary} is blocked by canonical material Unknowns: {}",
                projection.effective_unknowns.join(", ")
            ),
        });
    }
    Ok(projection)
}

pub(super) fn prepare_profile_transition(
    root: &Path,
    work_item_id: &str,
    previous_contract_value: &serde_json::Value,
    previous_contract_digest: &Digest,
    next_contract_value: &serde_json::Value,
    next_contract_digest: &Digest,
    previous_summary: &serde_json::Value,
) -> Result<Option<serde_json::Value>, ObserverError> {
    let previous_contract: Contract = serde_json::from_value(previous_contract_value.clone())
        .map_err(|error| ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: format!("previous amendment Contract is invalid: {error}"),
        })?;
    let next_contract: Contract =
        serde_json::from_value(next_contract_value.clone()).map_err(|error| {
            ObserverError::State {
                path: root.join(".ai/work-items/active"),
                message: format!("next amendment Contract is invalid: {error}"),
            }
        })?;
    if previous_contract.repository_id != repository_id(root).to_string()
        || next_contract.repository_id != previous_contract.repository_id
        || previous_contract.work_item_id != work_item_id
        || next_contract.work_item_id != work_item_id
    {
        return Err(ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: "material-review profile transition changed Work Item identity".into(),
        });
    }
    let previous_profile = previous_contract
        .material_inspection_review_profile()
        .map_err(|message| ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: format!("previous material-review profile is invalid: {message}"),
        })?;
    let next_profile = next_contract
        .material_inspection_review_profile()
        .map_err(|message| ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: format!("next material-review profile is invalid: {message}"),
        })?;
    let previous_profile_digest = previous_profile
        .as_ref()
        .map(json_digest)
        .transpose()
        .map_err(|error| ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: error.to_string(),
        })?;
    let next_profile_digest =
        next_profile
            .as_ref()
            .map(json_digest)
            .transpose()
            .map_err(|error| ObserverError::State {
                path: root.join(".ai/work-items/active"),
                message: error.to_string(),
            })?;
    if previous_profile_digest == next_profile_digest {
        return Ok(None);
    }

    let Some(previous_pointer) = previous_summary
        .get("materialReviewReceipt")
        .filter(|pointer| !pointer.is_null())
        .cloned()
    else {
        return Ok(None);
    };
    if previous_profile.is_none() || next_profile.is_none() {
        return Err(ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: "cannot transition a material-review profile while a receipt exists unless both profiles are enabled".into(),
        });
    }

    let mut previous_request = material_review_request_with_contract_digest(
        root,
        &previous_contract,
        previous_contract_digest,
    )
    .map_err(|error| ObserverError::State {
        path: root.join(".ai/work-items/active"),
        message: format!("cannot validate the prior material-review request: {error}"),
    })?;
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let previous_receipt = match read_material_review_receipt(
        root,
        &previous_contract,
        previous_contract_digest,
        &previous_request,
        &summary_path,
    )? {
        MaterialReviewReceiptState::Current(receipt) => *receipt,
        MaterialReviewReceiptState::Stale => {
            return Err(ObserverError::State {
                path: summary_path,
                message: "cannot change a material-review profile while its receipt is stale; record a current decision first".into(),
            });
        }
        MaterialReviewReceiptState::Missing => {
            return Err(ObserverError::State {
                path: summary_path,
                message: "material-review Summary pointer disappeared while preparing a profile transition".into(),
            });
        }
    };
    let reviewed_source_head = previous_receipt
        .reviewed_source_head
        .as_deref()
        .ok_or_else(|| ObserverError::State {
            path: summary_path.clone(),
            message: "prior material-review receipt lacks reviewed source-head provenance".into(),
        })?;
    // The request digest deliberately excludes reviewedSourceHead. Preserve
    // the receipt's exact historic head in the transition while retaining the
    // current request's unchanged manifest and digest. read_material_review_receipt
    // above has already checked that this historic head is an ancestor of the
    // current checkout and that the receipt matches every digest-bound field.
    previous_request.reviewed_source_head = reviewed_source_head.to_owned();
    let previous_request_digest =
        material_review_request_digest(&previous_request).map_err(|error| {
            ObserverError::State {
                path: summary_path.clone(),
                message: format!("cannot validate the captured prior request: {error}"),
            }
        })?;
    if previous_request_digest != previous_request.request_digest {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message:
                "prior material-review request changed while capturing its historic source head"
                    .into(),
        });
    }
    let next_request =
        material_review_request_with_contract_digest(root, &next_contract, next_contract_digest)
            .map_err(|error| ObserverError::State {
                path: root.join(".ai/work-items/active"),
                message: format!("cannot bind the next material-review request: {error}"),
            })?;
    let mut transition = MaterialReviewProfileTransition {
        schema_version: 1,
        repository_id: previous_contract.repository_id.clone(),
        work_item_id: work_item_id.to_owned(),
        previous_contract: previous_contract_value.clone(),
        previous_contract_digest: previous_contract_digest.clone(),
        next_contract: next_contract_value.clone(),
        next_contract_digest: next_contract_digest.clone(),
        previous_profile_digest: previous_profile_digest.expect("profile is present"),
        next_profile_digest: next_profile_digest.expect("profile is present"),
        previous_request,
        next_request,
        previous_pointer,
        previous_receipt,
        transition_digest: Digest::sha256_bytes(b"uncommitted-material-review-profile-transition"),
    };
    let mut value = serde_json::to_value(&transition).map_err(|error| ObserverError::State {
        path: root.join(".ai/work-items/active"),
        message: format!("cannot encode material-review profile transition: {error}"),
    })?;
    transition.transition_digest = material_review_profile_transition_digest(&value)?;
    value["transitionDigest"] = serde_json::json!(transition.transition_digest);
    Ok(Some(value))
}

fn material_review_profile_transition_digest(
    transition: &serde_json::Value,
) -> Result<Digest, ObserverError> {
    let mut core = transition.clone();
    core.as_object_mut()
        .ok_or_else(|| ObserverError::State {
            path: "material-review-profile-transition".into(),
            message: "profile transition must be a JSON object".into(),
        })?
        .remove("transitionDigest");
    digest_json(&("ai-cockpit:material-review-profile-transition:v1", core)).map_err(|error| {
        ObserverError::State {
            path: "material-review-profile-transition".into(),
            message: error.to_string(),
        }
    })
}

fn validate_material_review_profile_transition(
    transition_value: &serde_json::Value,
    amendment: &super::ContractAmendmentReceipt,
    receipt: &MaterialInspectionReviewDecisionReceipt,
    pointer: &serde_json::Value,
    current_contract: &Contract,
    current_profile_digest: &Digest,
) -> Result<bool, ObserverError> {
    let transition: MaterialReviewProfileTransition =
        serde_json::from_value(transition_value.clone()).map_err(|error| ObserverError::State {
            path: "material-review-profile-transition".into(),
            message: format!("invalid profile transition record: {error}"),
        })?;
    let actual_digest = material_review_profile_transition_digest(transition_value)?;
    if transition.schema_version != 1
        || transition.transition_digest != actual_digest
        || transition.repository_id != current_contract.repository_id
        || transition.work_item_id != current_contract.work_item_id
        || transition.repository_id != amendment.repository_id
        || transition.work_item_id != amendment.work_item_id
        || transition.previous_contract_digest != amendment.previous_contract_digest
        || transition.next_contract_digest != amendment.new_contract_digest
        || transition.previous_contract_digest != receipt.contract_digest
        || transition.previous_receipt != *receipt
        || transition.previous_pointer != *pointer
        || transition.previous_profile_digest != receipt.profile_digest
        || transition.next_profile_digest != *current_profile_digest
    {
        return Ok(false);
    }

    let previous_contract: Contract = serde_json::from_value(transition.previous_contract.clone())
        .map_err(|error| ObserverError::State {
            path: "material-review-profile-transition".into(),
            message: format!("invalid previous Contract in profile transition: {error}"),
        })?;
    let next_contract: Contract = serde_json::from_value(transition.next_contract.clone())
        .map_err(|error| ObserverError::State {
            path: "material-review-profile-transition".into(),
            message: format!("invalid next Contract in profile transition: {error}"),
        })?;
    if json_digest(&transition.previous_contract).map_err(|error| ObserverError::State {
        path: "material-review-profile-transition".into(),
        message: error.to_string(),
    })? != transition.previous_contract_digest
        || json_digest(&transition.next_contract).map_err(|error| ObserverError::State {
            path: "material-review-profile-transition".into(),
            message: error.to_string(),
        })? != transition.next_contract_digest
    {
        return Ok(false);
    }
    let previous_profile = previous_contract
        .material_inspection_review_profile()
        .map_err(|message| ObserverError::State {
            path: "material-review-profile-transition".into(),
            message: format!("invalid previous profile in transition: {message}"),
        })?
        .ok_or_else(|| ObserverError::State {
            path: "material-review-profile-transition".into(),
            message: "previous profile is missing from transition".into(),
        })?;
    let next_profile = next_contract
        .material_inspection_review_profile()
        .map_err(|message| ObserverError::State {
            path: "material-review-profile-transition".into(),
            message: format!("invalid next profile in transition: {message}"),
        })?
        .ok_or_else(|| ObserverError::State {
            path: "material-review-profile-transition".into(),
            message: "next profile is missing from transition".into(),
        })?;
    if json_digest(&previous_profile).map_err(|error| ObserverError::State {
        path: "material-review-profile-transition".into(),
        message: error.to_string(),
    })? != transition.previous_profile_digest
        || json_digest(&next_profile).map_err(|error| ObserverError::State {
            path: "material-review-profile-transition".into(),
            message: error.to_string(),
        })? != transition.next_profile_digest
        || previous_contract.repository_id != transition.repository_id
        || next_contract.repository_id != transition.repository_id
        || previous_contract.work_item_id != transition.work_item_id
        || next_contract.work_item_id != transition.work_item_id
        || current_contract.repository_id != transition.repository_id
        || current_contract.work_item_id != transition.work_item_id
    {
        return Ok(false);
    }
    let previous_request_digest = material_review_request_digest(&transition.previous_request)
        .map_err(|error| ObserverError::State {
            path: "material-review-profile-transition".into(),
            message: error.to_string(),
        })?;
    let next_request_digest =
        material_review_request_digest(&transition.next_request).map_err(|error| {
            ObserverError::State {
                path: "material-review-profile-transition".into(),
                message: error.to_string(),
            }
        })?;
    if previous_request_digest != transition.previous_request.request_digest
        || next_request_digest != transition.next_request.request_digest
        || transition.previous_request.contract_digest != transition.previous_contract_digest
        || transition.next_request.contract_digest != transition.next_contract_digest
        || transition
            .previous_request
            .material_inspection_review_profile_digest
            .as_ref()
            != Some(&transition.previous_profile_digest)
        || transition
            .next_request
            .material_inspection_review_profile_digest
            .as_ref()
            != Some(&transition.next_profile_digest)
        || transition.previous_request.repository_id != transition.repository_id
        || transition.next_request.repository_id != transition.repository_id
        || transition.previous_request.work_item_id != transition.work_item_id
        || transition.next_request.work_item_id != transition.work_item_id
        || receipt.request_digest != transition.previous_request.request_digest
        || receipt.profile_digest != transition.previous_profile_digest
    {
        return Ok(false);
    }
    receipt
        .validate_integrity()
        .map_err(|message| ObserverError::State {
            path: "material-review-profile-transition".into(),
            message,
        })?;
    let input = MaterialInspectionReviewDecisionInput {
        schema_version: receipt.schema_version,
        decision: receipt.decision,
        request_digest: receipt.request_digest.clone(),
        reviewer_actor: receipt.reviewer_actor.clone(),
        authority_source: receipt.authority_source.clone(),
        assurance: receipt.assurance,
        evidence_refs: receipt.evidence_refs.clone(),
        rationale: receipt.rationale.clone(),
        residual_risk: receipt.residual_risk.clone(),
    };
    let expected = validate_material_review_decision_with_contract_digest(
        &previous_contract,
        &transition.previous_contract_digest,
        &transition.previous_request,
        &input,
        &receipt.recorded_by,
        &receipt.recorded_at,
    )
    .map_err(|error| ObserverError::State {
        path: "material-review-profile-transition".into(),
        message: format!(
            "prior review does not validate against its captured Contract and request: {error}"
        ),
    })?;
    if expected != *receipt || transition.next_profile_digest != *current_profile_digest {
        return Ok(false);
    }
    Ok(true)
}

fn read_material_review_receipt(
    root: &Path,
    contract: &Contract,
    contract_digest: &Digest,
    request: &MaterialReviewRequest,
    summary_path: &Path,
) -> Result<MaterialReviewReceiptState, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.to_path_buf(),
        source,
    })?;
    let relative_summary_path =
        summary_path
            .strip_prefix(&root)
            .map_err(|_| ObserverError::State {
                path: summary_path.to_path_buf(),
                message: "material-review Summary path escapes repository".into(),
            })?;
    let parent_relative = relative_summary_path
        .parent()
        .ok_or_else(|| ObserverError::State {
            path: summary_path.to_path_buf(),
            message: "material-review Summary has no parent directory".into(),
        })?;
    let expected_summary_name = format!("{}.summary.json", contract.work_item_id);
    if parent_relative != Path::new(".ai/work-items/active")
        && parent_relative != Path::new(".ai/work-items/archive")
        || relative_summary_path
            .file_name()
            .and_then(|name| name.to_str())
            != Some(expected_summary_name.as_str())
    {
        return Err(ObserverError::State {
            path: summary_path.to_path_buf(),
            message: "material-review Summary path is outside its Work Item directory".into(),
        });
    }
    let root_dir = Dir::open_ambient_dir(&root, ambient_authority()).map_err(|source| {
        ObserverError::Read {
            path: root.clone(),
            source,
        }
    })?;
    let ai_path = root.join(".ai");
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &ai_path)?;
    let work_items_path = ai_path.join("work-items");
    let work_items = open_cap_directory_nofollow_strict(&ai, "work-items", &work_items_path)?;
    let directory_name = parent_relative
        .file_name()
        .and_then(|name| name.to_str())
        .expect("validated Work Item directory name");
    let work_item_directory_path = work_items_path.join(directory_name);
    let work_item_directory =
        open_cap_directory_nofollow_strict(&work_items, directory_name, &work_item_directory_path)?;
    let summary_name = format!("{}.summary.json", contract.work_item_id);
    let summary_bytes = match read_cap_file_nofollow_bounded(
        &work_item_directory,
        &summary_name,
        summary_path,
        MAX_BOUNDED_GIT_OUTPUT_BYTES as u64,
    ) {
        Ok(bytes) => bytes,
        Err(_error) if !summary_path.exists() => {
            return Ok(MaterialReviewReceiptState::Missing);
        }
        Err(error) => return Err(error),
    };
    super::reject_duplicate_json_keys(&summary_bytes).map_err(|message| ObserverError::State {
        path: summary_path.to_path_buf(),
        message: format!("invalid Summary JSON: {message}"),
    })?;
    let summary: serde_json::Value =
        serde_json::from_slice(&summary_bytes).map_err(|error| ObserverError::State {
            path: summary_path.to_path_buf(),
            message: format!("invalid Summary JSON: {error}"),
        })?;
    let Some(pointer) = summary.get("materialReviewReceipt") else {
        return Ok(MaterialReviewReceiptState::Missing);
    };
    let path = pointer.get("path").and_then(serde_json::Value::as_str);
    let file_digest = pointer.get("digest").and_then(serde_json::Value::as_str);
    let request_digest = pointer
        .get("requestDigest")
        .and_then(serde_json::Value::as_str);
    let receipt_digest = pointer
        .get("receiptDigest")
        .and_then(serde_json::Value::as_str);
    let pointer_request_digest = request_digest
        .and_then(|digest| digest.parse::<Digest>().ok())
        .ok_or_else(|| ObserverError::State {
            path: summary_path.to_path_buf(),
            message: "material-review Summary request digest is invalid".into(),
        })?;
    let request_hex = pointer_request_digest
        .as_str()
        .strip_prefix("sha256:")
        .ok_or_else(|| ObserverError::State {
            path: summary_path.to_path_buf(),
            message: "material-review Summary request digest is invalid".into(),
        })?;
    let sidecar_name = format!("{request_hex}.json");
    let expected_relative_path = format!(
        ".ai/evidence/{MATERIAL_REVIEW_EVIDENCE_DIRECTORY}/{}/{sidecar_name}",
        contract.work_item_id
    );
    if path != Some(expected_relative_path.as_str()) {
        return Err(ObserverError::State {
            path: summary_path.to_path_buf(),
            message: "material-review Summary pointer path does not bind its request digest".into(),
        });
    }
    let stale_request = pointer_request_digest != request.request_digest;
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &root.join(".ai"))?;
    let evidence_path = root.join(".ai/evidence");
    let evidence = open_cap_directory_nofollow_strict(&ai, "evidence", &evidence_path)?;
    let review_path = evidence_path.join(MATERIAL_REVIEW_EVIDENCE_DIRECTORY);
    let review = open_cap_directory_nofollow_strict(
        &evidence,
        MATERIAL_REVIEW_EVIDENCE_DIRECTORY,
        &review_path,
    )?;
    let work_item_path = review_path.join(&contract.work_item_id);
    let work_item =
        open_cap_directory_nofollow_strict(&review, &contract.work_item_id, &work_item_path)?;
    let sidecar_path = work_item_path.join(&sidecar_name);
    let sidecar_bytes = read_cap_file_nofollow_bounded(
        &work_item,
        &sidecar_name,
        &sidecar_path,
        MAX_BOUNDED_GIT_OUTPUT_BYTES as u64,
    )?;
    let actual_file_digest = Digest::sha256_bytes(&sidecar_bytes);
    if file_digest != Some(actual_file_digest.as_str()) {
        return Err(ObserverError::State {
            path: sidecar_path,
            message: "material-review sidecar file digest does not match Summary pointer".into(),
        });
    }
    let receipt: MaterialInspectionReviewDecisionReceipt =
        serde_json::from_slice(&sidecar_bytes).map_err(|error| ObserverError::State {
            path: sidecar_path.clone(),
            message: format!("invalid material-review receipt: {error}"),
        })?;
    super::reject_duplicate_json_keys(&sidecar_bytes).map_err(|message| ObserverError::State {
        path: sidecar_path.clone(),
        message: format!("invalid material-review receipt: {message}"),
    })?;
    receipt
        .validate_integrity()
        .map_err(|message| ObserverError::State {
            path: sidecar_path.clone(),
            message,
        })?;
    if receipt_digest != Some(receipt.receipt_digest.as_str())
        || receipt.request_digest != pointer_request_digest
        || receipt.work_item_id != contract.work_item_id
        || receipt.repository_id != contract.repository_id
    {
        return Err(ObserverError::State {
            path: summary_path.to_path_buf(),
            message: "material-review receipt identity or digest does not match Summary pointer"
                .into(),
        });
    }
    let current_profile = contract
        .material_inspection_review_profile()
        .map_err(|message| ObserverError::State {
            path: summary_path.to_path_buf(),
            message: format!("current material-review profile is invalid: {message}"),
        })?
        .ok_or_else(|| ObserverError::State {
            path: summary_path.to_path_buf(),
            message: "current Contract no longer enables material review".into(),
        })?;
    let current_profile_digest =
        json_digest(&current_profile).map_err(|error| ObserverError::State {
            path: summary_path.to_path_buf(),
            message: error.to_string(),
        })?;
    let contract_was_amended = receipt.contract_digest != *contract_digest;
    if contract_was_amended
        && (!stale_request
            || !material_review_contract_amendment_chain_reaches(
                &root,
                &contract.work_item_id,
                &receipt.contract_digest,
                contract_digest,
            )?)
    {
        return Err(ObserverError::State {
            path: sidecar_path,
            message: "material-review receipt is bound to a different current Contract without a validated amendment chain".into(),
        });
    }
    if receipt.profile_digest != current_profile_digest
        && (!stale_request
            || !contract_was_amended
            || !material_review_profile_transition_reaches(
                &root,
                &contract.work_item_id,
                &summary["materialReviewReceipt"],
                &receipt,
                contract,
                contract_digest,
                &current_profile_digest,
            )?)
    {
        return Err(ObserverError::State {
            path: sidecar_path,
            message: "material-review receipt is bound to a different current review profile without a validated append-only profile transition".into(),
        });
    }
    let reviewed_source_head =
        receipt
            .reviewed_source_head
            .as_deref()
            .ok_or_else(|| ObserverError::State {
                path: sidecar_path.clone(),
                message: "material-review receipt lacks reviewed source-head provenance".into(),
            })?;
    let git = GitRepository::discover(&root).map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let reviewed_head_is_ancestor = git
        .is_ancestor_bounded(reviewed_source_head, &request.reviewed_source_head, 1024)
        .map_err(|error| ObserverError::State {
            path: sidecar_path.clone(),
            message: format!("cannot verify reviewed source-head ancestry: {error}"),
        })?;
    if !reviewed_head_is_ancestor {
        return Err(ObserverError::State {
            path: sidecar_path.clone(),
            message: "reviewed source head is not an ancestor of the current consumer head".into(),
        });
    }
    if stale_request {
        // A valid receipt for an earlier exact request remains immutable
        // history, but it cannot discharge the current request's Unknowns.
        // A fresh decision may be recorded for the new request while keeping
        // the old sidecar and pointer in Summary history.
        return Ok(MaterialReviewReceiptState::Stale);
    }
    let input = MaterialInspectionReviewDecisionInput {
        schema_version: receipt.schema_version,
        decision: receipt.decision,
        request_digest: receipt.request_digest.clone(),
        reviewer_actor: receipt.reviewer_actor.clone(),
        authority_source: receipt.authority_source.clone(),
        assurance: receipt.assurance,
        evidence_refs: receipt.evidence_refs.clone(),
        rationale: receipt.rationale.clone(),
        residual_risk: receipt.residual_risk.clone(),
    };
    let mut receipt_request = request.clone();
    receipt_request.reviewed_source_head = reviewed_source_head.to_owned();
    let expected = validate_material_review_decision_with_contract_digest(
        contract,
        contract_digest,
        &receipt_request,
        &input,
        &receipt.recorded_by,
        &receipt.recorded_at,
    )
    .map_err(|error| ObserverError::State {
        path: sidecar_path.clone(),
        message: error.to_string(),
    })?;
    if expected != receipt {
        return Err(ObserverError::State {
            path: sidecar_path,
            message: "material-review receipt does not match current request or Contract".into(),
        });
    }
    Ok(MaterialReviewReceiptState::Current(Box::new(receipt)))
}

fn material_review_contract_amendment_chain_reaches(
    root: &Path,
    work_item_id: &str,
    previous_contract_digest: &Digest,
    current_contract_digest: &Digest,
) -> Result<bool, ObserverError> {
    super::contract_amendment::validated_contract_transition_chain_reaches(
        root,
        work_item_id,
        previous_contract_digest,
        current_contract_digest,
    )
}

fn material_review_profile_transition_reaches(
    root: &Path,
    work_item_id: &str,
    pointer: &serde_json::Value,
    receipt: &MaterialInspectionReviewDecisionReceipt,
    current_contract: &Contract,
    current_contract_digest: &Digest,
    current_profile_digest: &Digest,
) -> Result<bool, ObserverError> {
    let amendments = super::read_work_item_contract_amendments(root, work_item_id)?;
    let mut valid_transitions = 0;
    for amendment in &amendments {
        let Some(transition) = amendment.material_review_profile_transition.as_ref() else {
            continue;
        };
        if amendment.previous_contract_digest != receipt.contract_digest
            || (amendment.new_contract_digest != *current_contract_digest
                && !super::contract_amendment::validated_contract_transition_chain_reaches(
                    root,
                    work_item_id,
                    &amendment.new_contract_digest,
                    current_contract_digest,
                )?)
        {
            continue;
        }
        if validate_material_review_profile_transition(
            transition,
            amendment,
            receipt,
            pointer,
            current_contract,
            current_profile_digest,
        )? {
            valid_transitions += 1;
        }
    }
    Ok(valid_transitions == 1)
}

#[cfg(test)]
fn material_review_contract_amendment_digests_form_chain(
    amendments: impl IntoIterator<Item = (Digest, Digest)>,
    previous_contract_digest: &Digest,
    current_contract_digest: &Digest,
) -> bool {
    let amendments = amendments.into_iter().collect::<Vec<_>>();
    let incoming_anchors = amendments
        .iter()
        .enumerate()
        .filter_map(|(index, (_, new))| (new == previous_contract_digest).then_some(index))
        .collect::<Vec<_>>();
    let outgoing_anchors = amendments
        .iter()
        .enumerate()
        .filter_map(|(index, (previous, _))| {
            (previous == previous_contract_digest).then_some(index)
        })
        .collect::<Vec<_>>();

    let [start] = outgoing_anchors.as_slice() else {
        return false;
    };
    match incoming_anchors.as_slice() {
        // A receipt at the initial Contract digest has no incoming amendment;
        // its only verifiable origin is the first journal entry.
        [] if *start == 0 => {}
        // A later receipt digest must have one unique incoming edge directly
        // before its unique outgoing edge. This rejects branched histories
        // that happen to rejoin at the receipt digest.
        [incoming] if incoming + 1 == *start => {}
        _ => return false,
    }

    let mut expected_previous = previous_contract_digest.clone();
    for (amendment_previous, amendment_new) in &amendments[*start..] {
        if amendment_previous != &expected_previous {
            return false;
        }
        expected_previous = amendment_new.clone();
    }
    expected_previous == *current_contract_digest
}

/// Build the canonical read-only material-review request for one active Work
/// Item. The Contract must remain a regular repository-local file, and all
/// source identity checks are performed by `material_review_request`.
pub fn plan_work_item_material_review(
    root: &Path,
    work_item_id: &str,
) -> Result<MaterialReviewRequest, MaterialReviewRequestError> {
    let root = fs::canonicalize(root)
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    super::validate_work_item_id(work_item_id)
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    let document = read_active_contract_document(&root, work_item_id)?;
    let contract = &document.contract;
    if contract.work_item_id != work_item_id
        || contract.repository_id != super::repository_id(&root).to_string()
    {
        return Err(MaterialReviewRequestError::ContractIdentity);
    }
    material_review_request_with_contract_digest(&root, contract, &document.digest)
}

fn material_review_work_item_directory(
    root: &Path,
    work_item_id: &str,
    create: bool,
) -> Result<Dir, ObserverError> {
    let root_dir =
        Dir::open_ambient_dir(root, ambient_authority()).map_err(|source| ObserverError::Read {
            path: root.to_path_buf(),
            source,
        })?;
    let ai_path = root.join(".ai");
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &ai_path)?;
    let evidence_path = ai_path.join("evidence");
    let evidence = open_cap_directory_nofollow_strict(&ai, "evidence", &evidence_path)?;
    let review_path = evidence_path.join(MATERIAL_REVIEW_EVIDENCE_DIRECTORY);
    let review = if create {
        create_and_open_cap_directory(&evidence, MATERIAL_REVIEW_EVIDENCE_DIRECTORY, &review_path)?
    } else {
        open_cap_directory_nofollow_strict(
            &evidence,
            MATERIAL_REVIEW_EVIDENCE_DIRECTORY,
            &review_path,
        )?
    };
    let work_item_path = review_path.join(work_item_id);
    if create {
        create_and_open_cap_directory(&review, work_item_id, &work_item_path)
    } else {
        open_cap_directory_nofollow_strict(&review, work_item_id, &work_item_path)
    }
}

/// Record an exact typed material-review decision only when the Contract
/// opt-in and fresh Runtime action admission are both present. Stage one has
/// no opt-in, so this operation rejects without writing evidence or Summary.
pub fn record_work_item_material_review(
    root: &Path,
    work_item_id: &str,
    input: &MaterialInspectionReviewDecisionInput,
    runtime: &RuntimeContext,
) -> Result<MaterialInspectionReviewDecisionReceipt, ObserverError> {
    record_work_item_material_review_with_caller(root, work_item_id, input, runtime, None)
}

pub fn record_work_item_material_review_with_caller(
    root: &Path,
    work_item_id: &str,
    input: &MaterialInspectionReviewDecisionInput,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
) -> Result<MaterialInspectionReviewDecisionReceipt, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.to_path_buf(),
        source,
    })?;
    validate_work_item_id(work_item_id)?;
    let _lifecycle_lock = acquire_lifecycle_lock(&root, work_item_id)?;
    let document = read_active_contract_document(&root, work_item_id).map_err(|error| {
        ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: error.to_string(),
        }
    })?;
    let contract = &document.contract;
    if contract.work_item_id != work_item_id
        || contract.repository_id != repository_id(&root).to_string()
    {
        return Err(ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: "material-review Contract identity does not match this Work Item".into(),
        });
    }
    let request = material_review_request_with_contract_digest(&root, contract, &document.digest)
        .map_err(|error| ObserverError::State {
        path: root.join(".ai/work-items/active"),
        message: error.to_string(),
    })?;
    if !request.review_enabled {
        return Err(ObserverError::State {
            path: root
                .join(".ai/work-items/active")
                .join(format!("{work_item_id}.contract.json")),
            message: "material review decision is not enabled by the current Contract".into(),
        });
    }
    super::require_current_action_admission_with_caller(
        &root,
        work_item_id,
        "record_material_review_decision",
        runtime,
        caller,
    )?;
    let recorded_by = format!(
        "runtime:{}:{}",
        runtime.runtime_version, runtime.runtime_digest
    );
    let receipt = validate_material_review_decision_with_contract_digest(
        contract,
        &document.digest,
        &request,
        input,
        &recorded_by,
        &super::now(),
    )
    .map_err(|error| ObserverError::State {
        path: root
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json")),
        message: error.to_string(),
    })?;

    let active_path = root.join(".ai/work-items/active");
    let root_dir = Dir::open_ambient_dir(&root, ambient_authority()).map_err(|source| {
        ObserverError::Read {
            path: root.clone(),
            source,
        }
    })?;
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &root.join(".ai"))?;
    let work_items =
        open_cap_directory_nofollow_strict(&ai, "work-items", &root.join(".ai/work-items"))?;
    let active = open_cap_directory_nofollow_strict(&work_items, "active", &active_path)?;
    let summary_name = format!("{work_item_id}.summary.json");
    let summary_path = active_path.join(&summary_name);
    let summary_bytes = read_cap_file_nofollow_bounded(
        &active,
        &summary_name,
        &summary_path,
        MAX_BOUNDED_GIT_OUTPUT_BYTES as u64,
    )?;
    super::reject_duplicate_json_keys(&summary_bytes).map_err(|message| ObserverError::State {
        path: summary_path.clone(),
        message: format!("invalid Summary JSON: {message}"),
    })?;
    let mut summary: serde_json::Value =
        serde_json::from_slice(&summary_bytes).map_err(|error| ObserverError::State {
            path: summary_path.clone(),
            message: format!("invalid Summary JSON: {error}"),
        })?;
    if summary
        .get("workItemId")
        .and_then(serde_json::Value::as_str)
        != Some(work_item_id)
    {
        return Err(ObserverError::State {
            path: summary_path,
            message: "Summary identity does not match material-review Work Item".into(),
        });
    }
    let prior_pointer = match read_material_review_receipt(
        &root,
        contract,
        &document.digest,
        &request,
        &summary_path,
    )? {
        MaterialReviewReceiptState::Missing => None,
        MaterialReviewReceiptState::Current(_) => {
            return Err(ObserverError::State {
                path: summary_path,
                message: "material-review receipt is already recorded; replay is rejected".into(),
            });
        }
        MaterialReviewReceiptState::Stale => Some(
            summary
                .get("materialReviewReceipt")
                .cloned()
                .ok_or_else(|| ObserverError::State {
                    path: summary_path.clone(),
                    message: "stale material-review receipt pointer disappeared".into(),
                })?,
        ),
    };

    let summary_object = summary
        .as_object_mut()
        .ok_or_else(|| ObserverError::State {
            path: summary_path.clone(),
            message: "Summary must be a JSON object".into(),
        })?;
    if let Some(previous_pointer) = prior_pointer.as_ref() {
        let history = summary_object
            .entry("materialReviewReceiptHistory")
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));
        let history = history.as_array_mut().ok_or_else(|| ObserverError::State {
            path: summary_path.clone(),
            message: "material-review receipt history must be an array".into(),
        })?;
        if !history.contains(previous_pointer) {
            history.push(previous_pointer.clone());
        }
    }

    let evidence = material_review_work_item_directory(&root, work_item_id, true)?;
    let evidence_path = root
        .join(".ai/evidence")
        .join(MATERIAL_REVIEW_EVIDENCE_DIRECTORY)
        .join(work_item_id);
    let request_hex = request
        .request_digest
        .as_str()
        .strip_prefix("sha256:")
        .ok_or_else(|| ObserverError::State {
            path: root.join(".ai/evidence"),
            message: "material-review request digest is malformed".into(),
        })?;
    let evidence_name = format!("{request_hex}.json");
    let sidecar_path = evidence_path.join(&evidence_name);
    let sidecar_bytes =
        serde_json::to_vec_pretty(&receipt).map_err(|error| ObserverError::State {
            path: sidecar_path.clone(),
            message: error.to_string(),
        })?;
    let receipt_file_digest = Digest::sha256_bytes(&sidecar_bytes);
    let relative_sidecar_path =
        format!(".ai/evidence/{MATERIAL_REVIEW_EVIDENCE_DIRECTORY}/{work_item_id}/{evidence_name}");
    summary_object.insert(
        "materialReviewReceipt".into(),
        serde_json::json!({
            "path": relative_sidecar_path,
            "digest": receipt_file_digest,
            "requestDigest": request.request_digest,
            "receiptDigest": receipt.receipt_digest,
        }),
    );
    super::usage::write_immutable_sidecar(
        &evidence,
        &evidence_name,
        &sidecar_path,
        &sidecar_bytes,
    )?;
    super::atomic_json(&active_path.join(summary_name), &summary)?;
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        WorkItemStartOptions, amend_work_item_contract_with_runtime, attach, scaffold_work_item,
        start_work_item_with_options_and_runtime,
    };
    use cockpit_protocol::{RuntimeContext, digest_json};
    use serde_json::json;
    use std::{path::PathBuf, process::Command};

    fn run_git(root: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(["-C"])
            .arg(root)
            .args(args)
            .output()
            .expect("run git");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn git_text(root: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(["-C"])
            .arg(root)
            .args(args)
            .output()
            .expect("run git");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("git output is UTF-8")
            .trim()
            .to_owned()
    }

    fn activation_bridge_fixture(
        work_item_id: &str,
    ) -> (tempfile::TempDir, Digest, Digest, PathBuf) {
        let directory = tempfile::tempdir().expect("repository");
        let root = directory.path();
        run_git(root, &["init", "-q"]);
        run_git(root, &["config", "--local", "core.autocrlf", "false"]);
        run_git(
            root,
            &[
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit-test@example.invalid",
                "commit",
                "--allow-empty",
                "--quiet",
                "-m",
                "fixture baseline",
            ],
        );
        attach(root).expect("attach");
        std::fs::write(root.join(".gitattributes"), b"/.ai/** -text\n")
            .expect("preserve native evidence bytes in cloned fixture repositories");
        std::fs::write(root.join("README.md"), "activation bridge fixture\n").expect("README");
        run_git(root, &["add", "README.md", ".gitattributes", ".ai"]);
        run_git(
            root,
            &[
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit-test@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "fixture repository baseline",
            ],
        );
        let base = git_text(root, &["rev-parse", "HEAD"]);
        run_git(root, &["branch", "-M", "main"]);
        run_git(
            root,
            &[
                "remote",
                "add",
                "origin",
                "https://example.invalid/activation-bridge.git",
            ],
        );
        run_git(root, &["update-ref", "refs/remotes/origin/main", &base]);
        run_git(
            root,
            &[
                "symbolic-ref",
                "refs/remotes/origin/HEAD",
                "refs/remotes/origin/main",
            ],
        );
        run_git(root, &["checkout", "-qb", "activation-bridge-fixture"]);

        scaffold_work_item(root, work_item_id, "implementation").expect("scaffold");
        let contract_path = root.join(format!(
            ".ai/work-items/active/{work_item_id}.contract.json"
        ));
        let scaffolded: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&contract_path).expect("scaffold Contract"))
                .expect("valid scaffold Contract");
        let before_start = json!({
            "schemaVersion":1,
            "changeId":"declare-activation-fixture",
            "expectedContractDigest":digest_json(&scaffolded).expect("scaffold digest"),
            "reason":"isolated native activation continuity fixture",
            "changes":[
                {"path":"/intent","operation":"set","value":"Exercise material review continuity"},
                {"path":"/goal","operation":"set","value":"Retain exact receipt provenance across activation"},
                {"path":"/scope","operation":"replace","value":["README.md"]},
                {"path":"/risk","operation":"set","value":"normal"},
                {"path":"/authority","operation":"set","value":"authorized"},
                {"path":"/acceptanceCriteria","operation":"replace","value":["only validated continuity is accepted"]},
                {"path":"/verification","operation":"replace","value":["git diff --check"]}
            ]
        });
        let runtime = RuntimeContext {
            runtime_version: env!("CARGO_PKG_VERSION").into(),
            protocol_version: cockpit_protocol::PROTOCOL_VERSION,
            runtime_digest: Digest::sha256_bytes(b"isolated activation bridge test Runtime"),
        };
        amend_work_item_contract_with_runtime(root, work_item_id, &before_start, "", &runtime)
            .expect("declare fixture before activation");
        start_work_item_with_options_and_runtime(
            root,
            work_item_id,
            "Exercise material review continuity",
            "Retain exact receipt provenance across activation",
            &["README.md".into()],
            &WorkItemStartOptions {
                risk: "normal".into(),
                authority: "authorized".into(),
                acceptance_criteria: vec!["only validated continuity is accepted".into()],
                verification_commands: vec!["git diff --check".into()],
                ..WorkItemStartOptions::default()
            },
            &runtime,
        )
        .expect("record native activation transition");

        let receipt_contract: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&contract_path).expect("activated Contract"))
                .expect("valid activated Contract");
        let receipt_digest = digest_json(&receipt_contract).expect("receipt Contract digest");
        let after_activation = json!({
            "schemaVersion":1,
            "changeId":"append-after-activation",
            "expectedContractDigest":receipt_digest,
            "reason":"advance the exact Contract after the activation boundary",
            "changes":[
                {"path":"/acceptanceCriteria","operation":"replace","value":["validated activation is a required chain edge"]}
            ]
        });
        amend_work_item_contract_with_runtime(root, work_item_id, &after_activation, "", &runtime)
            .expect("amend after activation");
        let current_contract: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&contract_path).expect("current Contract"))
                .expect("valid current Contract");
        let current_digest = digest_json(&current_contract).expect("current Contract digest");
        let transition_directory =
            root.join(format!(".ai/evidence/{work_item_id}.activation-transition"));
        (
            directory,
            receipt_digest,
            current_digest,
            transition_directory,
        )
    }

    fn rebind_activation_record(
        transition_directory: &Path,
        mutate: impl FnOnce(&mut serde_json::Value),
    ) {
        let prepared_path = transition_directory.join("prepared.json");
        let committed_path = transition_directory.join("committed.json");
        let mut prepared: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&prepared_path).expect("prepared activation record"),
        )
        .expect("valid prepared activation record");
        mutate(&mut prepared["record"]);
        let record_digest = digest_json(&prepared["record"]).expect("updated record digest");
        prepared["recordDigest"] = json!(record_digest);
        std::fs::write(
            &prepared_path,
            serde_json::to_vec_pretty(&prepared).expect("encode prepared record"),
        )
        .expect("write prepared test fixture");
        let mut committed: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&committed_path).expect("committed activation record"),
        )
        .expect("valid committed activation record");
        committed["preparedDigest"] = json!(record_digest);
        std::fs::write(
            committed_path,
            serde_json::to_vec_pretty(&committed).expect("encode committed record"),
        )
        .expect("write committed test fixture");
    }

    #[test]
    fn material_review_receipt_chain_accepts_validated_activation_bridge() {
        let work_item_id = "WI-MATERIAL-ACTIVATION-BRIDGE";
        let (directory, receipt, current, _) = activation_bridge_fixture(work_item_id);
        assert!(
            material_review_contract_amendment_chain_reaches(
                directory.path(),
                work_item_id,
                &receipt,
                &current,
            )
            .expect("validate native activation bridge")
        );
    }

    #[test]
    fn material_review_receipt_chain_rejects_tampered_activation_bindings() {
        let work_item_id = "WI-MATERIAL-ACTIVATION-BRIDGE";
        for (field, value) in [
            ("journalSequence", json!(999)),
            (
                "journalDigest",
                json!(Digest::sha256_bytes(b"foreign journal")),
            ),
            (
                "previousContractDigest",
                json!(Digest::sha256_bytes(b"foreign previous Contract")),
            ),
            (
                "newContractDigest",
                json!(Digest::sha256_bytes(b"foreign new Contract")),
            ),
            ("workItemId", json!("WI-FOREIGN")),
        ] {
            let (directory, receipt, current, transition_directory) =
                activation_bridge_fixture(work_item_id);
            rebind_activation_record(&transition_directory, |record| {
                record[field] = value;
            });
            let result = material_review_contract_amendment_chain_reaches(
                directory.path(),
                work_item_id,
                &receipt,
                &current,
            );
            assert!(
                !matches!(result, Ok(true)),
                "tampered {field} must not validate an activation bridge"
            );
        }
    }

    #[test]
    fn material_review_amendment_chain_is_contiguous_from_receipt_to_current_contract() {
        let digest = |bytes: &[u8]| Digest::sha256_bytes(bytes);
        let original = digest(b"original Contract");
        let amended = digest(b"first amended Contract");
        let detached = digest(b"detached Contract");
        let receipt_contract = digest(b"receipt Contract");
        let middle = digest(b"middle Contract");
        let current = digest(b"current Contract");
        let other_terminal = digest(b"other terminal Contract");

        assert!(material_review_contract_amendment_digests_form_chain(
            vec![
                (original.clone(), amended.clone()),
                (amended.clone(), current.clone()),
            ],
            &original,
            &current,
        ));
        assert!(!material_review_contract_amendment_digests_form_chain(
            vec![
                (detached.clone(), amended.clone()),
                (original.clone(), current.clone()),
            ],
            &original,
            &current,
        ));
        assert!(material_review_contract_amendment_digests_form_chain(
            vec![
                (detached.clone(), amended),
                (original.clone(), receipt_contract.clone()),
                (receipt_contract.clone(), current.clone()),
            ],
            &receipt_contract,
            &current,
        ));
        assert!(!material_review_contract_amendment_digests_form_chain(
            vec![
                (detached, middle),
                (receipt_contract.clone(), current.clone()),
            ],
            &receipt_contract,
            &current,
        ));
        assert!(!material_review_contract_amendment_digests_form_chain(
            vec![(receipt_contract.clone(), other_terminal)],
            &receipt_contract,
            &current,
        ));
        assert!(!material_review_contract_amendment_digests_form_chain(
            vec![(original.clone(), current.clone())],
            &receipt_contract,
            &current,
        ));
    }

    #[test]
    fn material_review_amendment_chain_rejects_duplicate_receipt_anchor() {
        let digest = |bytes: &[u8]| Digest::sha256_bytes(bytes);
        let first_origin = digest(b"first origin");
        let second_origin = digest(b"second origin");
        let receipt_contract = digest(b"receipt contract");
        let current = digest(b"current contract");

        assert!(!material_review_contract_amendment_digests_form_chain(
            vec![
                (first_origin, receipt_contract.clone()),
                (second_origin, receipt_contract.clone()),
                (receipt_contract.clone(), current.clone()),
            ],
            &receipt_contract,
            &current,
        ));
    }

    #[test]
    fn material_review_initial_base_anchor_requires_a_unique_origin() {
        let digest = |bytes: &[u8]| Digest::sha256_bytes(bytes);
        let unrelated = digest(b"unrelated");
        let initial = digest(b"initial Contract");
        let middle = digest(b"middle Contract");
        let current = digest(b"current Contract");

        assert!(material_review_contract_amendment_digests_form_chain(
            vec![
                (initial.clone(), middle.clone()),
                (middle.clone(), current.clone()),
            ],
            &initial,
            &current,
        ));
        assert!(!material_review_contract_amendment_digests_form_chain(
            vec![
                (unrelated, digest(b"unrelated successor")),
                (initial.clone(), current.clone())
            ],
            &initial,
            &current,
        ));
        assert!(!material_review_contract_amendment_digests_form_chain(
            vec![
                (initial.clone(), middle),
                (initial.clone(), current.clone()),
            ],
            &initial,
            &current,
        ));
    }

    #[test]
    fn material_review_amendment_anchor_requires_adjacent_unique_receipt_edges() {
        let digest = |bytes: &[u8]| Digest::sha256_bytes(bytes);
        let origin = digest(b"origin");
        let receipt_contract = digest(b"receipt Contract");
        let detached = digest(b"detached Contract");
        let detached_successor = digest(b"detached successor");
        let current = digest(b"current Contract");

        assert!(!material_review_contract_amendment_digests_form_chain(
            vec![
                (origin, receipt_contract.clone()),
                (detached, detached_successor),
                (receipt_contract.clone(), current.clone()),
            ],
            &receipt_contract,
            &current,
        ));
    }

    #[test]
    fn material_request_digest_binds_finding_categories() {
        let digest = || Digest::sha256_bytes(b"fixture");
        let mut request = MaterialReviewRequest {
            schema_version: 1,
            repository_id: "repository".into(),
            work_item_id: "WI-TEST".into(),
            contract_digest: digest(),
            immutable_contract_base_revision: "base".into(),
            source_snapshot_digest: digest(),
            material_manifest_digest: digest(),
            scanner_semantic_version: "scanner".into(),
            analysis_implementation_digest: digest(),
            analysis_target_semantic_profile: "target".into(),
            material_inspection_review_profile_digest: None,
            effective_policy_digest: digest(),
            entries: Vec::new(),
            raw_unknown_codes: Vec::new(),
            finding_codes: Vec::new(),
            blocked_by_finding: false,
            review_enabled: false,
            review_diagnostic: None,
            request_digest: digest(),
            reviewed_source_head: "head".into(),
        };
        let without_findings = material_review_request_digest(&request).expect("request digest");
        request.finding_codes.push("test_weakening".into());
        let with_findings = material_review_request_digest(&request).expect("request digest");
        assert_ne!(without_findings, with_findings);
        request.finding_codes.clear();
        request.blocked_by_finding = true;
        let boolean_only = material_review_request_digest(&request).expect("request digest");
        assert_ne!(without_findings, boolean_only);
    }

    #[test]
    fn comparison_combines_current_review_and_complete_clean_blob() {
        let directory = tempfile::tempdir().expect("isolated repository");
        // Exercise normalization on every platform, including Windows' verbatim
        // canonical prefix. Internal receipt readers require canonical paths.
        let fixture_root = directory.path().join("..").join(
            directory
                .path()
                .file_name()
                .expect("fixture directory name"),
        );
        let canonical_root = fs::canonicalize(&fixture_root).expect("canonical fixture root");
        let root = canonical_root.as_path();
        run_git(root, &["init", "-q"]);
        run_git(root, &["config", "core.autocrlf", "false"]);
        fs::create_dir(root.join("src")).unwrap();
        let padding = "// bounded comparison padding\n".repeat(12_000);
        let before = format!("{padding}fn material() {{ let count = 1; }}\n");
        fs::write(root.join("src/clean.rs"), &before).unwrap();
        run_git(root, &["add", "."]);
        let commit = |message: &str| {
            run_git(
                root,
                &[
                    "-c",
                    "user.name=fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "commit",
                    "-qm",
                    message,
                ],
            )
        };
        commit("base");
        let base = git_text(root, &["rev-parse", "HEAD"]);
        attach(root).unwrap();
        let project: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join(".ai/project.json")).unwrap()).unwrap();
        fs::write(
            root.join("src/clean.rs"),
            format!("{padding}fn material() {{ let token_count = 2; }}\n"),
        )
        .unwrap();
        fs::write(root.join("src/reviewed.rs"),
            "fn material() -> String { let mut label = String::from(\"token\"); label.push_str(\"ization\"); label }\n").unwrap();
        run_git(root, &["add", "src"]);
        commit("candidate");
        let contract: Contract = serde_json::from_value(json!({
            "protocolVersion": 1,
            "repositoryId": repository_id(root).to_string(),
            "workItemId": "WI-COMPARISON-UNION",
            "intent": "review exact committed syntax", "goal": "combine exact material proofs",
            "scope": ["src/**"], "outOfScope": [], "risk": "high", "authority": "authorized",
            "acceptanceCriteria": ["preserve all uncovered material"], "requiredEvidenceClasses": [],
            "baseRevision": base, "projectProfileDigest": project["profileDigest"],
            "repositorySnapshotDigest": Digest::sha256_bytes(b"fixture snapshot"),
            "requiredRuntimeCapabilities": [MATERIAL_INSPECTION_REVIEW_CAPABILITY],
            "governanceProfile": {"materialInspectionReview": {
                "schemaVersion": 1, "permittedUnknown": "repository_material_inspection_unavailable",
                "permittedCause": "readable_committed_rust_syntax_unknown", "assurance": "self_declared",
                "reviewerActor": "agent:fixture", "authoritySource": "isolated-test-review",
                "acceptResidualRisk": true
            }}
        })).unwrap();
        let active = root.join(".ai/work-items/active");
        fs::create_dir_all(&active).unwrap();
        let contract_path = active.join("WI-COMPARISON-UNION.contract.json");
        let summary_path = active.join("WI-COMPARISON-UNION.summary.json");
        fs::write(
            &contract_path,
            serde_json::to_vec_pretty(&contract).unwrap(),
        )
        .unwrap();
        fs::write(
            &summary_path,
            serde_json::to_vec_pretty(&json!({
                "workItemId": contract.work_item_id, "state": "implementation_active",
                "checkpointCount": 0, "createdAt": "2026-10-07T00:00:00Z"
            }))
            .unwrap(),
        )
        .unwrap();
        let runtime = RuntimeContext {
            runtime_version: "isolated-comparison-test".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"isolated comparison test Runtime"),
        };
        let request = plan_work_item_material_review(root, &contract.work_item_id).unwrap();
        let clean = request
            .entries
            .iter()
            .find(|e| e.path == "src/clean.rs")
            .unwrap();
        assert_eq!(clean.scanner_assessment, MaterialScannerAssessment::Clean);
        assert!(clean.after_blob_digest.is_some());
        assert_eq!(request.entries.iter().filter(|e| e.reviewable).count(), 1);
        super::super::preflight_work_item_with_runtime(root, &contract_path, &runtime).unwrap();
        let input = serde_json::from_value(json!({
            "schemaVersion": 1, "decision": "accept_permitted_unknowns",
            "requestDigest": request.request_digest, "reviewerActor": "agent:fixture",
            "authoritySource": "isolated-test-review", "assurance": "self_declared",
            "evidenceRefs": [{"path": "src/reviewed.rs", "digest": request.entries.iter()
                .find(|e| e.path == "src/reviewed.rs").unwrap().after_blob_digest}],
            "rationale": "Review the exact syntax in this isolated test.",
            "residualRisk": "The machine syntax inspection remains incomplete."
        }))
        .unwrap();
        record_work_item_material_review(root, &contract.work_item_id, &input, &runtime).unwrap();
        // A relative Summary must still fail the canonical-root boundary,
        // regardless of platform-specific PathBuf parent normalization.
        let alias_summary = Path::new(".ai/work-items/active/WI-COMPARISON-UNION.summary.json");
        let alias_readback = read_material_review_receipt(
            root,
            &contract,
            &request.contract_digest,
            &request,
            alias_summary,
        );
        assert!(
            alias_readback.is_err(),
            "a relative Summary must not bypass the reader boundary: {alias_readback:?}"
        );
        let mut projection =
            material_review_gate_projection(root, &contract, &summary_path).unwrap();
        assert!(
            projection.review_receipt_digest.is_some(),
            "a real current receipt is required; projection={projection:?}; readback={:?}",
            read_material_review_receipt(
                root,
                &contract,
                &request.contract_digest,
                &request,
                &summary_path,
            )
        );
        assert!(projection.effective_unknowns.is_empty());
        let comparison = GitRepository::discover(root)
            .unwrap()
            .snapshot_against(&base)
            .unwrap();
        let truncated = comparison
            .change_evidence
            .iter()
            .find(|c| c.path == "src/clean.rs")
            .unwrap();
        assert!(truncated.after_text.is_none());
        assert_eq!(
            crate::rust_material::diagnose_rust_material(truncated).unknown_cause,
            Some(MaterialUnknownCause::MissingCompleteSource)
        );
        let canonical_projection = projection.clone();
        preserve_unreviewed_comparison_material_unknowns(&comparison, &mut projection);
        assert!(
            !projection
                .effective_unknowns
                .iter()
                .any(|u| u == "repository_material_inspection_unavailable"),
            "the current reviewed member and complete canonical Clean blob must jointly cover comparison material"
        );
        let mut reviewed_only = comparison.clone();
        reviewed_only
            .change_evidence
            .retain(|c| c.path == "src/reviewed.rs");
        let mut checked = canonical_projection.clone();
        preserve_unreviewed_comparison_material_unknowns(&reviewed_only, &mut checked);
        assert!(
            checked.effective_unknowns.is_empty(),
            "review-only coverage remains valid"
        );
        for field in [
            "head",
            "clean_path",
            "reviewed_path",
            "kind",
            "hunk",
            "origin",
            "removed",
            "binary",
            "too_large",
            "unavailable",
            "other_cause",
        ] {
            let mut invalid = comparison.clone();
            let clean_index = invalid
                .change_evidence
                .iter()
                .position(|c| c.path == "src/clean.rs")
                .unwrap();
            let reviewed_index = invalid
                .change_evidence
                .iter()
                .position(|c| c.path == "src/reviewed.rs")
                .unwrap();
            let change = &mut invalid.change_evidence[clean_index];
            match field {
                "head" => invalid.head = Some("b".repeat(40)),
                "clean_path" => change.path = "src/uninspected.rs".into(),
                "reviewed_path" => {
                    invalid.change_evidence[reviewed_index].path = "src/foreign.rs".into()
                }
                "kind" => change.kind = ChangeKind::Added,
                "hunk" => change.added_lines.push("let extra_token = 3;".into()),
                "origin" => change.added_line_origins[0].after_line += 1,
                "removed" => change.removed_lines.push("let old_token = 1;".into()),
                "binary" => change.content_state = ChangeContentState::Binary,
                "too_large" => change.content_state = ChangeContentState::TooLarge,
                "unavailable" => change.content_state = ChangeContentState::Unavailable,
                "other_cause" => {
                    let reviewed = &mut invalid.change_evidence[reviewed_index];
                    reviewed.after_text = Some("fn".into());
                    assert_ne!(
                        crate::rust_material::diagnose_rust_material(reviewed).unknown_cause,
                        Some(MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown)
                    );
                }
                _ => unreachable!(),
            }
            let mut checked = canonical_projection.clone();
            preserve_unreviewed_comparison_material_unknowns(&invalid, &mut checked);
            assert!(
                checked
                    .effective_unknowns
                    .iter()
                    .any(|u| u == "repository_material_inspection_unavailable"),
                "{field}"
            );
        }
        let mut finding = comparison.clone();
        let mut injected = finding.change_evidence[0].clone();
        injected.path = "src/foreign.rs".into();
        // Reuse the public conformance payload as test data, without changing
        // its bytes or granting it instruction authority. This fixture is also
        // pinned by governance_signals::existing_attack_fixture_has_the_immutable_expected_digest.
        let payload = include_str!(
            "../../../tests/conformance/fixtures/repository-prompt-injection/repository/material.txt"
        );
        assert_eq!(
            Digest::sha256_bytes(payload.as_bytes()).to_string(),
            "sha256:a71f4b8cce7538d11f17367cea5e106aa152268c57624e5890fffaf2eb38a814"
        );
        let text = format!("fn injected() -> &'static str {{ {payload:?} }}");
        injected.added_lines = vec![text.clone()];
        injected.added_line_origins = vec![cockpit_git::AddedLineOrigin {
            after_line: 1,
            hunk_index: 0,
        }];
        injected.removed_lines.clear();
        injected.after_text = Some(text);
        assert_eq!(
            crate::rust_material::diagnose_rust_material(&injected).assessment,
            RustMaterialAssessment::Finding
        );
        finding.change_evidence.push(injected);
        let mut checked = canonical_projection.clone();
        preserve_unreviewed_comparison_material_unknowns(&finding, &mut checked);
        assert!(
            checked
                .effective_unknowns
                .iter()
                .any(|u| u == "repository_material_inspection_unavailable")
        );

        let mut other_unknown = canonical_projection.clone();
        other_unknown
            .effective_unknowns
            .push("evidence_stale".into());
        preserve_unreviewed_comparison_material_unknowns(&comparison, &mut other_unknown);
        assert_eq!(other_unknown.effective_unknowns, vec!["evidence_stale"]);
        for field in ["receipt_missing", "foreign_request", "foreign_profile"] {
            let original = fs::read(&summary_path).unwrap();
            let original_contract = fs::read(&contract_path).unwrap();
            let mut summary: serde_json::Value = serde_json::from_slice(&original).unwrap();
            match field {
                "receipt_missing" => {
                    summary
                        .as_object_mut()
                        .unwrap()
                        .remove("materialReviewReceipt");
                }
                "foreign_request" => {
                    summary["materialReviewReceipt"]["requestDigest"] =
                        json!(Digest::sha256_bytes(b"foreign"));
                }
                "foreign_profile" => {
                    let mut value: serde_json::Value =
                        serde_json::from_slice(&original_contract).unwrap();
                    value["governanceProfile"]["materialInspectionReview"]["reviewerActor"] =
                        json!("agent:foreign");
                    fs::write(&contract_path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
                }
                _ => unreachable!(),
            }
            fs::write(&summary_path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
            if let Ok(mut invalid) = material_review_gate_projection(root, &contract, &summary_path)
            {
                preserve_unreviewed_comparison_material_unknowns(&comparison, &mut invalid);
                assert!(
                    !invalid
                        .discharged_unknowns
                        .iter()
                        .any(|u| u == "repository_material_inspection_unavailable"),
                    "{field}"
                );
            }
            fs::write(&summary_path, original).unwrap();
            fs::write(&contract_path, original_contract).unwrap();
        }
        fs::write(
            root.join("src/clean.rs"),
            format!("{padding}fn material() {{ let token_count = 3; }}\n"),
        )
        .unwrap();
        assert!(
            material_review_gate_projection(root, &contract, &summary_path).is_err(),
            "dirty blob must not create proof"
        );
        run_git(root, &["add", "src/clean.rs"]);
        commit("changed committed blob");
        let mut stale = material_review_gate_projection(root, &contract, &summary_path).unwrap();
        let changed = GitRepository::discover(root)
            .unwrap()
            .snapshot_against(&base)
            .unwrap();
        preserve_unreviewed_comparison_material_unknowns(&changed, &mut stale);
        assert!(
            stale
                .effective_unknowns
                .iter()
                .any(|u| u == "material_review_receipt_stale")
        );
        assert!(
            !stale
                .discharged_unknowns
                .iter()
                .any(|u| u == "repository_material_inspection_unavailable")
        );
    }

    #[test]
    fn implementation_digest_is_build_independent_and_source_sensitive() {
        let one = implementation_digest_from_sources(&[("scanner.rs", b"same source")]);
        let two = implementation_digest_from_sources(&[("scanner.rs", b"same source")]);
        let changed = implementation_digest_from_sources(&[("scanner.rs", b"changed source")]);
        assert_eq!(one, two);
        assert_ne!(one, changed);
        assert_ne!(
            one,
            implementation_digest_from_sources(&[("different.rs", b"same source")])
        );
    }

    #[cfg(unix)]
    #[test]
    fn checkout_source_open_refuses_symlinks_and_does_not_block_on_fifo() {
        use std::{
            ffi::CString,
            os::unix::fs::symlink,
            time::{Duration, Instant},
        };

        let directory = tempfile::tempdir().expect("temporary directory");
        let target = directory.path().join("target.rs");
        let link = directory.path().join("link.rs");
        fs::write(&target, "fn safe() {}\n").expect("source");
        symlink(&target, &link).expect("symlink");
        assert!(open_checkout_source_nofollow(directory.path(), Path::new("link.rs")).is_err());

        let outside = directory.path().join("outside");
        fs::create_dir(&outside).expect("outside directory");
        fs::write(outside.join("secret.rs"), "fn outside() {}\n").expect("outside source");
        let linked_directory = directory.path().join("linked-directory");
        symlink(&outside, &linked_directory).expect("directory symlink");
        assert!(
            open_checkout_source_nofollow(
                directory.path(),
                Path::new("linked-directory/secret.rs")
            )
            .is_err()
        );

        let fifo = directory.path().join("pipe.rs");
        let fifo_name =
            CString::new(fifo.as_os_str().as_encoded_bytes()).expect("FIFO path has no NUL");
        let created = unsafe { libc::mkfifo(fifo_name.as_ptr(), 0o600) };
        assert_eq!(created, 0, "create FIFO: {}", io::Error::last_os_error());
        let started = Instant::now();
        let opened = open_checkout_source_nofollow(directory.path(), Path::new("pipe.rs"))
            .expect("nonblocking FIFO open");
        assert!(!opened.metadata().expect("FIFO metadata").is_file());
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[cfg(windows)]
    #[test]
    fn checkout_source_open_refuses_parent_directory_junctions() {
        use std::process::Command;

        let directory = tempfile::tempdir().expect("temporary directory");
        let root = directory.path().join("checkout");
        let outside = directory.path().join("outside");
        let junction = root.join("linked-directory");
        fs::create_dir(&root).expect("checkout directory");
        fs::create_dir(&outside).expect("outside directory");
        fs::write(outside.join("secret.rs"), "fn outside() {}\n").expect("outside source");

        let output = Command::new("cmd.exe")
            .args(["/D", "/C", "mklink", "/J"])
            .arg(&junction)
            .arg(&outside)
            .output()
            .expect("create junction with cmd.exe");
        assert!(
            output.status.success(),
            "mklink /J failed; stdout: {}; stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        assert!(
            open_checkout_source_nofollow(&root, Path::new("linked-directory/secret.rs")).is_err(),
            "parent junction must not expose source outside the checkout"
        );
    }
}
