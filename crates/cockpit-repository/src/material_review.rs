//! Read-only, committed-source material identity. No decision or discharge is
//! performed here; the opt-in is only reported to a future review service.

use super::{
    ObserverError, acquire_lifecycle_lock, contains_strong_instruction_injection,
    create_and_open_cap_directory, derive_governance_signals_with_diagnostics,
    effective_policy_for_contract, open_cap_directory_nofollow_strict,
    read_cap_file_nofollow_bounded, repository_id, require_current_action_admission,
    validate_work_item_id,
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
    path::{Path, PathBuf},
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

#[cfg(test)]
thread_local! {
    static REPLACEMENT_FAILURE: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
    static REPLACEMENT_BEFORE_COMMIT: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn replacement_test_boundary(root: &Path, stage: &'static str) -> Result<(), ObserverError> {
    REPLACEMENT_FAILURE.with(|failure| {
        if failure.get() == Some(stage) {
            failure.set(None);
            Err(replacement_error(
                root,
                format!("injected interruption: {stage}"),
            ))
        } else {
            Ok(())
        }
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialScannerAssessment {
    Clean,
    Finding,
    Unknown,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MaterialReviewGateProjection {
    pub raw_scanner_unknowns: Vec<String>,
    pub material_manifest_digest: Option<Digest>,
    pub review_receipt_digest: Option<Digest>,
    pub review_assurance: Option<MaterialInspectionReviewAssurance>,
    pub effective_unknowns: Vec<String>,
    pub discharged_unknowns: Vec<String>,
    pub reviewed_source_head: Option<String>,
    pub reviewed_unknown_members: Vec<ReviewedMaterialMemberIdentity>,
    pub finding_codes: Vec<String>,
    pub blocked_by_finding: bool,
    pub projection_unavailable: bool,
    pub review_decision_available: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReviewedMaterialMemberIdentity {
    path: String,
    kind: String,
    changed_hunk_digest: Digest,
    after_blob_digest: Digest,
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
                let review_decision_available = finite_authority_source_replacement_proof(
                    root,
                    contract,
                    contract_digest,
                    &request,
                    summary_path,
                )
                .is_ok_and(|proof| proof.is_some());
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
                    reviewed_source_head: None,
                    reviewed_unknown_members: Vec::new(),
                    finding_codes: request.finding_codes,
                    blocked_by_finding,
                    projection_unavailable: false,
                    review_decision_available,
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
        reviewed_source_head: may_discharge.then(|| request.reviewed_source_head.clone()),
        reviewed_unknown_members: if may_discharge {
            reviewed_unknown_members
        } else {
            Vec::new()
        },
        finding_codes: request.finding_codes,
        blocked_by_finding,
        projection_unavailable: false,
        review_decision_available,
    })
}

/// Restrict a canonical material-review discharge to exact members also
/// present in the CI comparison snapshot. The review request builder checks
/// that its committed source is clean and that its own HEAD/source identity is
/// stable before and after bounded blob reads. CI and canonical request
/// snapshots use different tree-digest algorithms, so this comparison binds
/// the canonical committed-blob proof only when both snapshots name the same
/// HEAD.
pub(crate) fn preserve_unreviewed_comparison_material_unknowns(
    comparison_snapshot: &RepositorySnapshot,
    projection: &mut MaterialReviewGateProjection,
) {
    const MATERIAL_UNKNOWN: &str = "repository_material_inspection_unavailable";

    if !projection
        .discharged_unknowns
        .iter()
        .any(|unknown| unknown == MATERIAL_UNKNOWN)
    {
        return;
    }

    let same_reviewed_head = projection
        .reviewed_source_head
        .as_deref()
        .is_some_and(|reviewed_head| comparison_snapshot.head.as_deref() == Some(reviewed_head));
    if !same_reviewed_head {
        preserve_material_unknown(
            &mut projection.discharged_unknowns,
            &mut projection.effective_unknowns,
        );
        return;
    }

    let mut unreviewed_snapshot = comparison_snapshot.clone();
    unreviewed_snapshot.change_evidence.retain(|change| {
        !projection
            .reviewed_unknown_members
            .iter()
            .any(|member| comparison_change_matches_reviewed_member(change, member))
    });
    let (signals, _) = derive_governance_signals_with_diagnostics(&unreviewed_snapshot);
    if signals
        .unknowns
        .iter()
        .any(|unknown| unknown == MATERIAL_UNKNOWN)
    {
        preserve_material_unknown(
            &mut projection.discharged_unknowns,
            &mut projection.effective_unknowns,
        );
    }
}

fn comparison_change_matches_reviewed_member(
    change: &ChangeEvidence,
    member: &ReviewedMaterialMemberIdentity,
) -> bool {
    let kind = format!("{:?}", change.kind).to_ascii_lowercase();
    if change.path != member.path || kind != member.kind {
        return false;
    }
    let Ok(changed_hunk_digest) = changed_hunk_digest(change) else {
        return false;
    };

    // ChangeEvidence intentionally does not carry committed blob digests.
    // With an exact request HEAD match, the canonical request's bounded,
    // committed afterBlobDigest is the source identity for this path; never
    // derive it from checkout after_text, which can be filtered or truncated.
    let comparison_member = ReviewedMaterialMemberIdentity {
        path: change.path.clone(),
        kind,
        changed_hunk_digest,
        after_blob_digest: member.after_blob_digest.clone(),
    };
    &comparison_member == member
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
            reviewed_source_head: None,
            reviewed_unknown_members: Vec::new(),
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

struct BoundMaterialReviewReceipt {
    root: PathBuf,
    summary: serde_json::Value,
    pointer: serde_json::Value,
    receipt: MaterialInspectionReviewDecisionReceipt,
    sidecar_path: PathBuf,
    stale_request: bool,
}

/// Only immutable file/pointer integrity. Current-profile validity is kept in
/// the strict reader below; historical integrity alone grants no discharge.
fn read_material_review_receipt_binding(
    root: &Path,
    contract: &Contract,
    request: &MaterialReviewRequest,
    summary_path: &Path,
) -> Result<Option<BoundMaterialReviewReceipt>, ObserverError> {
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
            return Ok(None);
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
        return Ok(None);
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
    Ok(Some(BoundMaterialReviewReceipt {
        root,
        pointer: pointer.clone(),
        summary,
        receipt,
        sidecar_path,
        stale_request,
    }))
}

fn read_material_review_receipt(
    root: &Path,
    contract: &Contract,
    contract_digest: &Digest,
    request: &MaterialReviewRequest,
    summary_path: &Path,
) -> Result<MaterialReviewReceiptState, ObserverError> {
    let Some(BoundMaterialReviewReceipt {
        root,
        receipt,
        sidecar_path,
        stale_request,
        ..
    }) = read_material_review_receipt_binding(root, contract, request, summary_path)?
    else {
        return Ok(MaterialReviewReceiptState::Missing);
    };
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
    if receipt.profile_digest != current_profile_digest {
        return Err(ObserverError::State {
            path: sidecar_path,
            message: "material-review receipt is bound to a different current review profile"
                .into(),
        });
    }
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthoritySourceReplacementProof {
    prior_pointer: serde_json::Value,
    prior_receipt_digest: Digest,
    historical_bindings_digest: Digest,
    journal_digest: Digest,
    current_review_digest: Digest,
    repository_snapshot_digest: Digest,
}

fn replacement_error(path: &Path, message: impl Into<String>) -> ObserverError {
    ObserverError::State {
        path: path.into(),
        message: format!("finite material-review replacement: {}", message.into()),
    }
}

fn replacement_json(
    directory: &Dir,
    name: &str,
    path: &Path,
) -> Result<(serde_json::Value, Digest), ObserverError> {
    let bytes =
        read_cap_file_nofollow_bounded(directory, name, path, MAX_BOUNDED_GIT_OUTPUT_BYTES as u64)?;
    super::reject_duplicate_json_keys(&bytes)
        .map_err(|message| replacement_error(path, message))?;
    let value = serde_json::from_slice(&bytes)
        .map_err(|error| replacement_error(path, error.to_string()))?;
    Ok((value, Digest::sha256_bytes(&bytes)))
}

/// Objects are compared recursively; a changed collection is one semantic
/// field. This supports a native whole-profile replace without hiding leaves.
fn replacement_changed_paths(
    before: &serde_json::Value,
    after: &serde_json::Value,
    prefix: &str,
    paths: &mut Vec<String>,
) {
    if before == after {
        return;
    }
    if let (Some(before), Some(after)) = (before.as_object(), after.as_object()) {
        let keys = before.keys().chain(after.keys()).collect::<BTreeSet<_>>();
        for key in keys {
            let escaped = key.replace('~', "~0").replace('/', "~1");
            replacement_changed_paths(
                before.get(key).unwrap_or(&serde_json::Value::Null),
                after.get(key).unwrap_or(&serde_json::Value::Null),
                &format!("{prefix}/{escaped}"),
                paths,
            );
        }
    } else {
        paths.push(prefix.to_owned());
    }
}

fn replacement_review_matches(
    value: &serde_json::Value,
    contract: &Contract,
    digest: &Digest,
    snapshot: &Digest,
) -> bool {
    let valid_digest_field = |key: &str| {
        value[key]
            .as_str()
            .and_then(|text| text.parse::<Digest>().ok())
            .is_some_and(|digest| valid_digest(&digest))
    };
    value["schemaVersion"] == 1
        && value["decisionId"] == "contract-preflight-review"
        && value["decision"] == "confirm_review"
        && value["repositoryId"] == contract.repository_id
        && value["workItemId"] == contract.work_item_id
        && value["contractDigest"].as_str() == Some(digest.as_str())
        && value["repositorySnapshotDigest"].as_str() == Some(snapshot.as_str())
        && valid_digest_field("preflightDecisionDigest")
        && value["recordedAt"]
            .as_str()
            .is_some_and(valid_utc_runtime_timestamp)
        && ["recordedBy", "reason"].iter().all(|key| {
            value[*key]
                .as_str()
                .is_some_and(|text| !text.trim().is_empty() && text.trim() == text)
        })
}

fn replacement_review_records(
    root: &Path,
    work_item_id: &str,
) -> Result<Vec<(serde_json::Value, Digest)>, ObserverError> {
    let root_dir =
        Dir::open_ambient_dir(root, ambient_authority()).map_err(|source| ObserverError::Read {
            path: root.into(),
            source,
        })?;
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &root.join(".ai"))?;
    let path = root.join(".ai/decisions");
    let directory = open_cap_directory_nofollow_strict(&ai, "decisions", &path)?;
    let mut names = Vec::new();
    for entry in directory.entries().map_err(|source| ObserverError::Read {
        path: path.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        if names.len() >= MAX_MATERIAL_CHANGED_FILES {
            return Err(replacement_error(
                &path,
                "too many historical review records",
            ));
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if name.starts_with(&format!("{work_item_id}.preflight-review")) && name.ends_with(".json")
        {
            names.push(name.to_owned());
        }
    }
    names.sort();
    names
        .into_iter()
        .map(|name| replacement_json(&directory, &name, &path.join(&name)))
        .collect()
}

fn replacement_receipt_matches_profile(
    receipt: &MaterialInspectionReviewDecisionReceipt,
    contract: &Contract,
) -> Result<bool, ObserverError> {
    let Some(profile) = contract
        .material_inspection_review_profile()
        .map_err(|message| replacement_error(Path::new("historical-profile"), message))?
    else {
        return Ok(false);
    };
    let mut paths = BTreeSet::new();
    Ok(json_digest(&profile)
        .map_err(|error| replacement_error(Path::new("historical-profile"), error.to_string()))?
        == receipt.profile_digest
        && profile.reviewer_actor == receipt.reviewer_actor
        && profile.authority_source == receipt.authority_source
        && receipt.assurance == MaterialInspectionReviewAssurance::SelfDeclared
        && receipt.decision == MaterialInspectionReviewDecision::AcceptPermittedUnknowns
        && valid_utc_runtime_timestamp(&receipt.recorded_at)
        && !receipt.recorded_by.trim().is_empty()
        && !receipt.rationale.trim().is_empty()
        && !receipt.residual_risk.trim().is_empty()
        && !receipt.evidence_refs.is_empty()
        && receipt.evidence_refs.iter().all(|reference| {
            !reference.path.trim().is_empty()
                && reference.path.trim() == reference.path
                && valid_digest(&reference.digest)
                && paths.insert(reference.path.clone())
        }))
}

fn replacement_pointer_history(
    root: &Path,
    contract: &Contract,
    bound: &BoundMaterialReviewReceipt,
    contracts: &BTreeMap<String, Contract>,
    current_head: &str,
) -> Result<Vec<Digest>, ObserverError> {
    let Some(values) = bound.summary.get("materialReviewReceiptHistory") else {
        return Ok(Vec::new());
    };
    let values = values
        .as_array()
        .ok_or_else(|| replacement_error(root, "pointer history must be an array"))?;
    if values.len() > MAX_MATERIAL_CHANGED_FILES {
        return Err(replacement_error(
            root,
            "pointer history exceeds bounded count",
        ));
    }
    let directory = material_review_work_item_directory(root, &contract.work_item_id, false)?;
    let path = root
        .join(".ai/evidence")
        .join(MATERIAL_REVIEW_EVIDENCE_DIRECTORY)
        .join(&contract.work_item_id);
    let git = GitRepository::discover(root)
        .map_err(|error| replacement_error(root, error.to_string()))?;
    let mut requests = BTreeSet::new();
    let mut digests = Vec::new();
    for pointer in values {
        if pointer.as_object().is_none_or(|object| {
            object.len() != 4
                || object.keys().any(|key| {
                    !["path", "digest", "requestDigest", "receiptDigest"].contains(&key.as_str())
                })
        }) || pointer == &bound.pointer
        {
            return Err(replacement_error(
                &path,
                "malformed or duplicated historical pointer",
            ));
        }
        let request = pointer["requestDigest"]
            .as_str()
            .and_then(|value| value.parse::<Digest>().ok())
            .filter(valid_digest)
            .ok_or_else(|| replacement_error(&path, "invalid historical pointer request"))?;
        if !requests.insert(request.to_string()) {
            return Err(replacement_error(
                &path,
                "duplicate historical request pointer",
            ));
        }
        let name = format!(
            "{}.json",
            request
                .as_str()
                .strip_prefix("sha256:")
                .expect("validated digest")
        );
        let relative = format!(
            ".ai/evidence/{MATERIAL_REVIEW_EVIDENCE_DIRECTORY}/{}/{name}",
            contract.work_item_id
        );
        let (value, raw) = replacement_json(&directory, &name, &path.join(&name))?;
        let receipt: MaterialInspectionReviewDecisionReceipt = serde_json::from_value(value)
            .map_err(|error| replacement_error(&path, error.to_string()))?;
        receipt
            .validate_integrity()
            .map_err(|message| replacement_error(&path, message))?;
        let historical = contracts
            .get(receipt.contract_digest.as_str())
            .ok_or_else(|| {
                replacement_error(
                    &path,
                    "historical receipt has no native Contract provenance",
                )
            })?;
        let head = receipt
            .reviewed_source_head
            .as_deref()
            .ok_or_else(|| replacement_error(&path, "historical source-head provenance missing"))?;
        if pointer["path"] != relative
            || pointer["digest"].as_str() != Some(raw.as_str())
            || pointer["receiptDigest"].as_str() != Some(receipt.receipt_digest.as_str())
            || request != receipt.request_digest
            || receipt.repository_id != contract.repository_id
            || receipt.work_item_id != contract.work_item_id
            || !replacement_receipt_matches_profile(&receipt, historical)?
            || !git
                .is_ancestor_bounded(head, current_head, 1024)
                .map_err(|error| replacement_error(&path, error.to_string()))?
        {
            return Err(replacement_error(
                &path,
                "historical pointer/sidecar identity or profile mismatch",
            ));
        }
        digests.push(raw);
    }
    Ok(digests)
}

/// H/P/C0 is a separate eligibility for NEW evidence, never a replacement for
/// strict current receipt validation or current Runtime action admission.
fn finite_authority_source_replacement_proof(
    root: &Path,
    contract: &Contract,
    contract_digest: &Digest,
    request: &MaterialReviewRequest,
    summary_path: &Path,
) -> Result<Option<AuthoritySourceReplacementProof>, ObserverError> {
    const SOURCE: &str = "/governanceProfile/materialInspectionReview/authoritySource";
    const HISTORY: &[&str] = &["scope", "acceptanceCriteria", "scenarioCoverage", "sources"];
    let unknowns = request
        .entries
        .iter()
        .filter(|entry| entry.scanner_assessment == MaterialScannerAssessment::Unknown)
        .collect::<Vec<_>>();
    if !request.review_enabled
        || contract.authority != "authorized"
        || request.blocked_by_finding
        || !request.finding_codes.is_empty()
        || request.raw_unknown_codes != ["repository_material_inspection_unavailable"]
        || unknowns.is_empty()
        || unknowns
            .iter()
            .any(|entry| !entry.reviewable || entry.after_blob_digest.is_none())
    {
        return Ok(None);
    }
    let Some(bound) = read_material_review_receipt_binding(root, contract, request, summary_path)?
    else {
        return Ok(None);
    };
    let profile = contract
        .material_inspection_review_profile()
        .map_err(|message| replacement_error(root, message))?
        .ok_or_else(|| replacement_error(root, "current profile missing"))?;
    if !bound.stale_request
        || json_digest(&profile).map_err(|error| replacement_error(root, error.to_string()))?
            == bound.receipt.profile_digest
    {
        return Ok(None);
    }
    if bound.summary["workItemId"] != contract.work_item_id
        || bound.summary["repositoryId"] != contract.repository_id
        || bound
            .summary
            .get("materialReviewReceiptHistory")
            .is_some_and(|value| !value.is_array())
    {
        return Err(replacement_error(
            summary_path,
            "invalid historical Summary identity/history",
        ));
    }
    let history = super::read_work_item_contract_amendments(root, &contract.work_item_id)?;
    super::contract_amendment::validate_transition_chain(
        root,
        &contract.work_item_id,
        &history,
        Some(contract_digest),
    )?;
    let Some(tail) = history.last() else {
        return Ok(None);
    };
    if tail.new_contract_digest != *contract_digest {
        return Err(replacement_error(
            root,
            "current Contract is not the committed native tail",
        ));
    }
    let starts = history
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            (item.previous_contract_digest == bound.receipt.contract_digest).then_some(index)
        })
        .collect::<Vec<_>>();
    let [start] = starts.as_slice() else {
        return Ok(None);
    };
    let root_dir =
        Dir::open_ambient_dir(root, ambient_authority()).map_err(|source| ObserverError::Read {
            path: root.into(),
            source,
        })?;
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &root.join(".ai"))?;
    let evidence = open_cap_directory_nofollow_strict(&ai, "evidence", &root.join(".ai/evidence"))?;
    let name = format!("{}.contract-amendments", contract.work_item_id);
    let path = root.join(".ai/evidence").join(&name);
    let directory = open_cap_directory_nofollow_strict(&evidence, &name, &path)?;
    let reviews = replacement_review_records(root, &contract.work_item_id)?;
    let mut raw_bindings = Vec::new();
    let mut historical_contracts = BTreeMap::new();
    let mut used_review_records = BTreeSet::new();
    let mut expected = bound.receipt.contract_digest.clone();
    for (index, item) in history.iter().enumerate() {
        let prepared_name = format!("{:08}.prepared.json", item.sequence);
        let committed_name = format!("{:08}.committed.json", item.sequence);
        let (prepared, prepared_raw) =
            replacement_json(&directory, &prepared_name, &path.join(&prepared_name))?;
        let (committed, committed_raw) =
            replacement_json(&directory, &committed_name, &path.join(&committed_name))?;
        let native: super::ContractAmendmentReceipt =
            serde_json::from_value(committed["receipt"].clone())
                .map_err(|error| replacement_error(&path, error.to_string()))?;
        let data = &prepared["prepared"];
        let before: Contract = serde_json::from_value(data["previousContract"].clone())
            .map_err(|error| replacement_error(&path, error.to_string()))?;
        let after: Contract = serde_json::from_value(data["newContract"].clone())
            .map_err(|error| replacement_error(&path, error.to_string()))?;
        let amendment: cockpit_protocol::ContractAmendmentRequest =
            serde_json::from_value(data["request"].clone())
                .map_err(|error| replacement_error(&path, error.to_string()))?;
        if native != *item
            || prepared["preparedDigest"] != committed["preparedDigest"]
            || json_digest(&data["previousContract"])
                .map_err(|error| replacement_error(&path, error.to_string()))?
                != item.previous_contract_digest
            || json_digest(&data["newContract"])
                .map_err(|error| replacement_error(&path, error.to_string()))?
                != item.new_contract_digest
            || json_digest(&amendment)
                .map_err(|error| replacement_error(&path, error.to_string()))?
                != item.request_digest
            || data["repositorySnapshotDigest"].as_str()
                != Some(item.repository_snapshot_digest.as_str())
        {
            return Err(replacement_error(
                &path,
                "raw historical inputs changed after native validation",
            ));
        }
        // The native loader already replays the complete prefix, including
        // Runtime-derived capability metadata on initial profile activation.
        // H/P starts at the receipt; its capabilities must remain unchanged.
        if index >= *start {
            let (derived, _) =
                cockpit_protocol::apply_contract_amendment_with_trace(&before, &amendment)
                    .map_err(|_| {
                        replacement_error(&path, "historical operation replay rejected")
                    })?;
            if derived != after {
                return Err(replacement_error(
                    &path,
                    "historical replay does not reach recorded Contract",
                ));
            }
        }
        raw_bindings.extend([prepared_raw, committed_raw]);
        historical_contracts.insert(item.previous_contract_digest.to_string(), before.clone());
        historical_contracts.insert(item.new_contract_digest.to_string(), after.clone());
        if item.policy_review_required {
            let Some((_, digest)) = reviews.iter().find(|(value, _)| {
                replacement_review_matches(
                    value,
                    contract,
                    &item.new_contract_digest,
                    &item.repository_snapshot_digest,
                ) && value["recordedAt"]
                    .as_str()
                    .zip(Some(item.recorded_at.as_str()))
                    .and_then(|(review, amendment)| {
                        chrono::DateTime::parse_from_rfc3339(review)
                            .ok()
                            .zip(chrono::DateTime::parse_from_rfc3339(amendment).ok())
                    })
                    .is_some_and(|(review, amendment)| review >= amendment)
                    && !used_review_records.contains(&value.to_string())
            }) else {
                return Err(replacement_error(
                    &path,
                    "a required historical Contract review is missing or foreign",
                ));
            };
            // A repeated Contract digest after a later transition still needs
            // its own actual decision, not a replay of an earlier file.
            used_review_records.insert(
                reviews
                    .iter()
                    .find(|(_, raw)| raw == digest)
                    .expect("selected review")
                    .0
                    .to_string(),
            );
            raw_bindings.push(digest.clone());
        }
        if index < *start {
            continue;
        }
        if item.previous_contract_digest != expected {
            return Err(replacement_error(
                &path,
                "receipt-to-anchor segment is discontinuous",
            ));
        }
        expected = item.new_contract_digest.clone();
        let mut changes = Vec::new();
        replacement_changed_paths(
            &data["previousContract"],
            &data["newContract"],
            "",
            &mut changes,
        );
        if index + 1 == history.len() {
            if changes != [SOURCE] || item.changed_values.len() != 1 {
                return Err(replacement_error(
                    &path,
                    "last adjacent transition is not one source-only change",
                ));
            }
            let change = &item.changed_values[0];
            let mut leaves = Vec::new();
            replacement_changed_paths(
                change
                    .old_value
                    .as_ref()
                    .unwrap_or(&serde_json::Value::Null),
                change
                    .new_value
                    .as_ref()
                    .unwrap_or(&serde_json::Value::Null),
                &change.path,
                &mut leaves,
            );
            if leaves != [SOURCE] {
                return Err(replacement_error(
                    &path,
                    "adjacent request mixes or reverts another semantic field",
                ));
            }
        } else {
            if before.governance_profile != after.governance_profile
                || changes
                    .iter()
                    .any(|path| !HISTORY.contains(&path.split('/').nth(1).unwrap_or_default()))
                || item.changed_values.iter().any(|change| {
                    !HISTORY.contains(&change.path.split('/').nth(1).unwrap_or_default())
                })
            {
                return Err(replacement_error(
                    &path,
                    "historical segment changes a forbidden field/profile",
                ));
            }
        }
    }
    let old_contract = historical_contracts
        .get(bound.receipt.contract_digest.as_str())
        .ok_or_else(|| replacement_error(root, "old review Contract unavailable"))?;
    if !replacement_receipt_matches_profile(&bound.receipt, old_contract)? {
        return Err(replacement_error(
            root,
            "old immutable receipt does not match its historical profile",
        ));
    }
    let context = super::RepositoryExecutionContext::capture(root)?;
    let observed = super::snapshot_digest(context.snapshot())?;
    let current_review = &bound.summary["decisionEvidence"];
    if observed != tail.repository_snapshot_digest
        || bound.summary["preflightContractDigest"].as_str() != Some(contract_digest.as_str())
        || bound.summary["preflightRepositorySnapshotDigest"].as_str() != Some(observed.as_str())
        || !replacement_review_matches(current_review, contract, contract_digest, &observed)
        || bound.summary["preflightDecisionDigest"] != current_review["preflightDecisionDigest"]
        || !reviews.iter().any(|(value, _)| value == current_review)
        || super::governance_controls::preflight_decision_evidence_state(
            root,
            &contract.work_item_id,
            contract_digest,
        ) != super::governance_controls::PreflightDecisionEvidenceState::Valid
    {
        return Err(replacement_error(
            root,
            "current source or independent native review is not bound to adjacent transition",
        ));
    }
    let reviewed = bound
        .receipt
        .reviewed_source_head
        .as_deref()
        .ok_or_else(|| replacement_error(root, "old source-head provenance missing"))?;
    let git = GitRepository::discover(root)
        .map_err(|error| replacement_error(root, error.to_string()))?;
    if !git
        .is_ancestor_bounded(reviewed, &request.reviewed_source_head, 1024)
        .map_err(|error| replacement_error(root, error.to_string()))?
        || history != super::read_work_item_contract_amendments(root, &contract.work_item_id)?
    {
        return Err(replacement_error(
            root,
            "historical ancestry/inputs changed during observation",
        ));
    }
    raw_bindings.extend(replacement_pointer_history(
        root,
        contract,
        &bound,
        &historical_contracts,
        &request.reviewed_source_head,
    )?);
    Ok(Some(AuthoritySourceReplacementProof {
        prior_pointer: bound.pointer,
        prior_receipt_digest: bound.receipt.receipt_digest,
        historical_bindings_digest: json_digest(&raw_bindings)
            .map_err(|error| replacement_error(root, error.to_string()))?,
        journal_digest: tail.journal_digest.clone(),
        current_review_digest: json_digest(current_review)
            .map_err(|error| replacement_error(root, error.to_string()))?,
        repository_snapshot_digest: observed,
    }))
}

fn material_review_contract_amendment_chain_reaches(
    root: &Path,
    work_item_id: &str,
    previous_contract_digest: &Digest,
    current_contract_digest: &Digest,
) -> Result<bool, ObserverError> {
    let amendments = super::read_work_item_contract_amendments(root, work_item_id)?;
    Ok(material_review_contract_amendment_digests_form_chain(
        amendments.into_iter().map(|amendment| {
            (
                amendment.previous_contract_digest,
                amendment.new_contract_digest,
            )
        }),
        previous_contract_digest,
        current_contract_digest,
    ))
}

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
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.to_path_buf(),
        source,
    })?;
    validate_work_item_id(work_item_id)?;
    let lifecycle_lock = acquire_lifecycle_lock(&root, work_item_id)?;
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
    let replacement_proof = finite_authority_source_replacement_proof(
        &root,
        contract,
        &document.digest,
        &request,
        &root.join(format!(".ai/work-items/active/{work_item_id}.summary.json")),
    )?;
    require_current_action_admission(
        &root,
        work_item_id,
        "record_material_review_decision",
        runtime,
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
    ) {
        Ok(MaterialReviewReceiptState::Missing) => None,
        Ok(MaterialReviewReceiptState::Current(_)) => {
            return Err(ObserverError::State {
                path: summary_path,
                message: "material-review receipt is already recorded; replay is rejected".into(),
            });
        }
        Ok(MaterialReviewReceiptState::Stale) => Some(
            summary
                .get("materialReviewReceipt")
                .cloned()
                .ok_or_else(|| ObserverError::State {
                    path: summary_path.clone(),
                    message: "stale material-review receipt pointer disappeared".into(),
                })?,
        ),
        Err(error) => match replacement_proof.as_ref() {
            Some(proof) if summary.get("materialReviewReceipt") == Some(&proof.prior_pointer) => {
                Some(proof.prior_pointer.clone())
            }
            _ => return Err(error),
        },
    };

    if let Some(proof) = replacement_proof.as_ref() {
        if prior_pointer.as_ref() != Some(&proof.prior_pointer) {
            return Err(replacement_error(
                &summary_path,
                "historical pointer changed before replacement",
            ));
        }
        return record_prepared_material_replacement(
            MaterialReplacementContext {
                root: &root,
                active: &active,
                contract,
                contract_digest: &document.digest,
                request: &request,
                input,
                runtime,
                summary_bytes: &summary_bytes,
                proof,
                lifecycle_lock: &lifecycle_lock,
            },
            receipt,
        );
    }

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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MaterialReplacementPreparation {
    schema_version: u32,
    repository_id: String,
    work_item_id: String,
    contract_digest: Digest,
    request_digest: Digest,
    profile_digest: Digest,
    proof_digest: Digest,
    input_digest: Digest,
    runtime: RuntimeContext,
    previous_summary_digest: Digest,
    target_summary: serde_json::Value,
    receipt: MaterialInspectionReviewDecisionReceipt,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PreparedMaterialReplacement {
    prepared: MaterialReplacementPreparation,
    prepared_digest: Digest,
}

struct MaterialReplacementContext<'a> {
    root: &'a Path,
    active: &'a Dir,
    contract: &'a Contract,
    contract_digest: &'a Digest,
    request: &'a MaterialReviewRequest,
    input: &'a MaterialInspectionReviewDecisionInput,
    runtime: &'a RuntimeContext,
    summary_bytes: &'a [u8],
    proof: &'a AuthoritySourceReplacementProof,
    lifecycle_lock: &'a super::lifecycle_lock_guard::LifecycleLockGuard,
}

fn replacement_optional_bytes(
    directory: &Dir,
    name: &str,
    path: &Path,
) -> Result<Option<Vec<u8>>, ObserverError> {
    match read_cap_file_nofollow_bounded(directory, name, path, MAX_BOUNDED_GIT_OUTPUT_BYTES as u64)
    {
        Ok(bytes) => Ok(Some(bytes)),
        Err(ObserverError::Read { source, .. }) if source.kind() == io::ErrorKind::NotFound => {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

fn replacement_target_summary(
    context: &MaterialReplacementContext<'_>,
    receipt: &MaterialInspectionReviewDecisionReceipt,
) -> Result<serde_json::Value, ObserverError> {
    let mut summary: serde_json::Value = serde_json::from_slice(context.summary_bytes)
        .map_err(|error| replacement_error(context.root, error.to_string()))?;
    if summary["materialReviewReceipt"] != context.proof.prior_pointer {
        return Err(replacement_error(
            context.root,
            "old Summary no longer binds the historical pointer",
        ));
    }
    let history = summary
        .as_object_mut()
        .ok_or_else(|| replacement_error(context.root, "Summary must be an object"))?
        .entry("materialReviewReceiptHistory")
        .or_insert_with(|| serde_json::json!([]))
        .as_array_mut()
        .ok_or_else(|| replacement_error(context.root, "pointer history must be an array"))?;
    if history.contains(&context.proof.prior_pointer) {
        return Err(replacement_error(
            context.root,
            "historical pointer already occurs in history",
        ));
    }
    history.push(context.proof.prior_pointer.clone());
    let bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|error| replacement_error(context.root, error.to_string()))?;
    let hex = receipt
        .request_digest
        .as_str()
        .strip_prefix("sha256:")
        .ok_or_else(|| replacement_error(context.root, "invalid prepared request digest"))?;
    summary["materialReviewReceipt"] = serde_json::json!({
        "path":format!(".ai/evidence/{MATERIAL_REVIEW_EVIDENCE_DIRECTORY}/{}/{hex}.json",context.contract.work_item_id),
        "digest":Digest::sha256_bytes(&bytes),
        "requestDigest":receipt.request_digest,
        "receiptDigest":receipt.receipt_digest,
    });
    Ok(summary)
}

fn replacement_prepared_digest(
    prepared: &MaterialReplacementPreparation,
) -> Result<Digest, ObserverError> {
    json_digest(&("ai-cockpit:material-replacement-prepare:v1", prepared)).map_err(|error| {
        replacement_error(Path::new("material-replacement-prepare"), error.to_string())
    })
}

fn revalidate_material_replacement(
    context: &MaterialReplacementContext<'_>,
    prepared: &MaterialReplacementPreparation,
) -> Result<(), ObserverError> {
    context
        .lifecycle_lock
        .require_subject(context.root, &context.contract.work_item_id)?;
    let document = read_active_contract_document(context.root, &context.contract.work_item_id)
        .map_err(|error| replacement_error(context.root, error.to_string()))?;
    if document.digest != *context.contract_digest || document.contract != *context.contract {
        return Err(replacement_error(
            context.root,
            "current Contract changed during replacement",
        ));
    }
    let request = material_review_request_with_contract_digest(
        context.root,
        &document.contract,
        &document.digest,
    )
    .map_err(|error| replacement_error(context.root, error.to_string()))?;
    if request.request_digest != context.request.request_digest {
        return Err(replacement_error(
            context.root,
            "current material changed during replacement",
        ));
    }
    let summary_name = format!("{}.summary.json", context.contract.work_item_id);
    let summary_path = context
        .root
        .join(".ai/work-items/active")
        .join(&summary_name);
    let proof = finite_authority_source_replacement_proof(
        context.root,
        context.contract,
        context.contract_digest,
        &request,
        &summary_path,
    )?
    .ok_or_else(|| {
        replacement_error(context.root, "finite current replacement proof unavailable")
    })?;
    if &proof != context.proof
        || json_digest(&proof)
            .map_err(|error| replacement_error(context.root, error.to_string()))?
            != prepared.proof_digest
    {
        return Err(replacement_error(
            context.root,
            "current authorization or native history changed",
        ));
    }
    require_current_action_admission(
        context.root,
        &context.contract.work_item_id,
        "record_material_review_decision",
        context.runtime,
    )?;
    let current = read_cap_file_nofollow_bounded(
        context.active,
        &summary_name,
        &summary_path,
        MAX_BOUNDED_GIT_OUTPUT_BYTES as u64,
    )?;
    if Digest::sha256_bytes(&current) != prepared.previous_summary_digest {
        return Err(replacement_error(
            &summary_path,
            "Summary compare-and-swap conflict; pending evidence remains noncurrent",
        ));
    }
    Ok(())
}

fn validate_material_replacement_preparation(
    context: &MaterialReplacementContext<'_>,
    envelope: &PreparedMaterialReplacement,
) -> Result<(), ObserverError> {
    let prepared = &envelope.prepared;
    if prepared.schema_version != 1
        || prepared.repository_id != context.contract.repository_id
        || prepared.work_item_id != context.contract.work_item_id
        || prepared.contract_digest != *context.contract_digest
        || prepared.request_digest != context.request.request_digest
        || Some(&prepared.profile_digest)
            != context
                .request
                .material_inspection_review_profile_digest
                .as_ref()
        || prepared.proof_digest
            != json_digest(context.proof)
                .map_err(|error| replacement_error(context.root, error.to_string()))?
        || prepared.input_digest
            != json_digest(context.input)
                .map_err(|error| replacement_error(context.root, error.to_string()))?
        || prepared.runtime != *context.runtime
        || prepared.previous_summary_digest != Digest::sha256_bytes(context.summary_bytes)
        || envelope.prepared_digest != replacement_prepared_digest(prepared)?
    {
        return Err(replacement_error(
            context.root,
            "prepared replacement is not this exact pending transaction",
        ));
    }
    let recorded_by = format!(
        "runtime:{}:{}",
        context.runtime.runtime_version, context.runtime.runtime_digest
    );
    let expected = validate_material_review_decision_with_contract_digest(
        context.contract,
        context.contract_digest,
        context.request,
        context.input,
        &recorded_by,
        &prepared.receipt.recorded_at,
    )
    .map_err(|error| replacement_error(context.root, error.to_string()))?;
    if expected != prepared.receipt
        || replacement_target_summary(context, &prepared.receipt)? != prepared.target_summary
    {
        return Err(replacement_error(
            context.root,
            "sealed receipt or target Summary does not match the current decision",
        ));
    }
    revalidate_material_replacement(context, prepared)
}

fn write_replacement_summary(
    active: &Dir,
    name: &str,
    path: &Path,
    bytes: &[u8],
) -> Result<(), ObserverError> {
    use cap_fs_ext::OpenOptionsFollowExt as _;
    use std::io::Write as _;
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temporary = format!(".{name}.replacement-{}-{sequence}.tmp", std::process::id());
    let mut options = cap_std::fs::OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(cap_fs_ext::FollowSymlinks::No);
    let mut file = active
        .open_with(&temporary, &options)
        .map_err(|source| ObserverError::Read {
            path: path.with_file_name(&temporary),
            source,
        })?
        .into_std();
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|source| ObserverError::Read {
            path: path.with_file_name(&temporary),
            source,
        })?;
    drop(file);
    active
        .rename(&temporary, active, name)
        .map_err(|source| ObserverError::Read {
            path: path.into(),
            source,
        })
}

fn record_prepared_material_replacement(
    context: MaterialReplacementContext<'_>,
    proposed_receipt: MaterialInspectionReviewDecisionReceipt,
) -> Result<MaterialInspectionReviewDecisionReceipt, ObserverError> {
    let directory =
        material_review_work_item_directory(context.root, &context.contract.work_item_id, false)?;
    let path = context
        .root
        .join(".ai/evidence")
        .join(MATERIAL_REVIEW_EVIDENCE_DIRECTORY)
        .join(&context.contract.work_item_id);
    let hex = context
        .request
        .request_digest
        .as_str()
        .strip_prefix("sha256:")
        .ok_or_else(|| replacement_error(context.root, "invalid replacement request digest"))?;
    let prepared_name = format!("{hex}.prepare.json");
    let sidecar_name = format!("{hex}.json");
    let prepared_path = path.join(&prepared_name);
    let sidecar_path = path.join(&sidecar_name);
    let pending = replacement_optional_bytes(&directory, &prepared_name, &prepared_path)?;
    let envelope = if let Some(bytes) = pending.as_ref() {
        super::reject_duplicate_json_keys(bytes)
            .map_err(|message| replacement_error(&prepared_path, message))?;
        serde_json::from_slice::<PreparedMaterialReplacement>(bytes)
            .map_err(|error| replacement_error(&prepared_path, error.to_string()))?
    } else {
        if replacement_optional_bytes(&directory, &sidecar_name, &sidecar_path)?.is_some() {
            return Err(replacement_error(
                &sidecar_path,
                "unbound sidecar without sealed prepare cannot be adopted",
            ));
        }
        let prepared = MaterialReplacementPreparation {
            schema_version: 1,
            repository_id: context.contract.repository_id.clone(),
            work_item_id: context.contract.work_item_id.clone(),
            contract_digest: context.contract_digest.clone(),
            request_digest: context.request.request_digest.clone(),
            profile_digest: proposed_receipt.profile_digest.clone(),
            proof_digest: json_digest(context.proof)
                .map_err(|error| replacement_error(context.root, error.to_string()))?,
            input_digest: json_digest(context.input)
                .map_err(|error| replacement_error(context.root, error.to_string()))?,
            runtime: context.runtime.clone(),
            previous_summary_digest: Digest::sha256_bytes(context.summary_bytes),
            target_summary: replacement_target_summary(&context, &proposed_receipt)?,
            receipt: proposed_receipt,
        };
        PreparedMaterialReplacement {
            prepared_digest: replacement_prepared_digest(&prepared)?,
            prepared,
        }
    };
    validate_material_replacement_preparation(&context, &envelope)?;
    let prepared_bytes = serde_json::to_vec_pretty(&envelope)
        .map_err(|error| replacement_error(&prepared_path, error.to_string()))?;
    let receipt_bytes = serde_json::to_vec_pretty(&envelope.prepared.receipt)
        .map_err(|error| replacement_error(&sidecar_path, error.to_string()))?;
    let target_bytes = serde_json::to_vec_pretty(&envelope.prepared.target_summary)
        .map_err(|error| replacement_error(context.root, error.to_string()))?;
    if [
        prepared_bytes.len(),
        receipt_bytes.len(),
        target_bytes.len(),
    ]
    .into_iter()
    .any(|size| size > MAX_BOUNDED_GIT_OUTPUT_BYTES)
        || pending
            .as_ref()
            .is_some_and(|bytes| bytes != &prepared_bytes)
    {
        return Err(replacement_error(
            &prepared_path,
            "oversized or changed immutable preparation",
        ));
    }
    #[cfg(test)]
    replacement_test_boundary(context.root, "before_prepare")?;
    super::usage::write_immutable_sidecar(
        &directory,
        &prepared_name,
        &prepared_path,
        &prepared_bytes,
    )?;
    #[cfg(test)]
    replacement_test_boundary(context.root, "after_prepare")?;
    #[cfg(test)]
    replacement_test_boundary(context.root, "before_sidecar")?;
    super::usage::write_immutable_sidecar(
        &directory,
        &sidecar_name,
        &sidecar_path,
        &receipt_bytes,
    )?;
    #[cfg(test)]
    replacement_test_boundary(context.root, "after_sidecar")?;
    #[cfg(test)]
    REPLACEMENT_BEFORE_COMMIT.with(|callback| {
        if let Some(callback) = callback.borrow_mut().take() {
            callback();
        }
    });
    // Recompute both source/authority proof and raw Summary CAS immediately
    // before the only commit point. Staged files never discharge Unknowns.
    revalidate_material_replacement(&context, &envelope.prepared)?;
    #[cfg(test)]
    replacement_test_boundary(context.root, "before_summary")?;
    let summary_name = format!("{}.summary.json", context.contract.work_item_id);
    let summary_path = context
        .root
        .join(".ai/work-items/active")
        .join(&summary_name);
    let result =
        write_replacement_summary(context.active, &summary_name, &summary_path, &target_bytes);
    #[cfg(test)]
    let result = result.and_then(|()| replacement_test_boundary(context.root, "after_summary"));
    // A returned error can occur after rename. Read the canonical state to
    // distinguish committed, exact pending, and conflict; never blind retry.
    let canonical = read_cap_file_nofollow_bounded(
        context.active,
        &summary_name,
        &summary_path,
        MAX_BOUNDED_GIT_OUTPUT_BYTES as u64,
    )?;
    if canonical == target_bytes {
        let actual = read_cap_file_nofollow_bounded(
            &directory,
            &sidecar_name,
            &sidecar_path,
            MAX_BOUNDED_GIT_OUTPUT_BYTES as u64,
        )?;
        if actual != receipt_bytes {
            return Err(replacement_error(
                &sidecar_path,
                "committed pointer/sidecar conflict",
            ));
        }
        return Ok(envelope.prepared.receipt);
    }
    if Digest::sha256_bytes(&canonical) != envelope.prepared.previous_summary_digest {
        return Err(replacement_error(
            &summary_path,
            "replacement commit conflict; inspect canonical state",
        ));
    }
    result?;
    Err(replacement_error(
        &summary_path,
        "Summary remained pending after commit; inspect canonical state",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transaction_git(root: &Path, args: &[&str]) {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn transaction_fixture_amend(
        root: &Path,
        runtime: &RuntimeContext,
        changes: serde_json::Value,
    ) {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        let path = root.join(format!(".ai/work-items/active/{ITEM}.contract.json"));
        let contract: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let sequence = crate::read_work_item_contract_amendments(root, ITEM)
            .unwrap()
            .len()
            + 1;
        crate::amend_work_item_contract_with_runtime(root, ITEM, &serde_json::json!({
            "schemaVersion":1, "changeId":format!("synthetic-transaction-{sequence}"),
            "expectedContractDigest":digest_json(&contract).unwrap(),
            "reason":"Synthetic native transaction fixture, not actual person authority.", "changes":changes
        }), "", runtime).unwrap();
        crate::preflight_work_item_with_runtime(root, &path, runtime).unwrap();
        let contract: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let summary: serde_json::Value = serde_json::from_slice(
            &fs::read(root.join(format!(".ai/work-items/active/{ITEM}.summary.json"))).unwrap(),
        )
        .unwrap();
        crate::record_work_item_governance_controls_with_runtime(root, ITEM, &serde_json::json!({"decisionEvidence":{
            "schemaVersion":1, "decisionId":"contract-preflight-review", "decision":"confirm_review",
            "repositoryId":crate::repository_id(root).to_string(), "workItemId":ITEM,
            "contractDigest":digest_json(&contract).unwrap(), "preflightDecisionDigest":summary["preflightDecisionDigest"],
            "repositorySnapshotDigest":summary["preflightRepositorySnapshotDigest"],
            "recordedAt":chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos,true),
            "recordedBy":"human:transaction-fixture", "reason":"Synthetic exact native fixture review; no real person impersonated."
        }}), runtime).unwrap();
    }

    fn transaction_fixture() -> (
        tempfile::TempDir,
        RuntimeContext,
        MaterialInspectionReviewDecisionInput,
        PathBuf,
        Vec<u8>,
    ) {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        transaction_git(root, &["init", "-q"]);
        fs::write(root.join("README.md"), "base\n").unwrap();
        transaction_git(root, &["add", "."]);
        transaction_git(
            root,
            &[
                "-c",
                "user.name=fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "base",
            ],
        );
        crate::attach(root).unwrap();
        let runtime = RuntimeContext {
            runtime_version: "transaction-fixture".into(),
            runtime_digest: Digest::sha256_bytes(b"transaction-fixture"),
            protocol_version: cockpit_protocol::PROTOCOL_VERSION,
        };
        crate::start_work_item_with_options_and_runtime(
            root,
            ITEM,
            "recover an exact material decision",
            "preserve history during replacement",
            &["README.md".into(), "src/**".into()],
            &crate::WorkItemStartOptions {
                authority: "authorized".into(),
                risk: "high".into(),
                acceptance_criteria: vec!["bounded synthetic current decision".into()],
                ..Default::default()
            },
            &runtime,
        )
        .unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/material.rs"),"fn material() -> String { let mut text=String::from(\"token\"); text.push_str(\"ization\"); text }\n").unwrap();
        transaction_git(root, &["add", "."]);
        transaction_git(
            root,
            &[
                "-c",
                "user.name=fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "material",
            ],
        );
        transaction_fixture_amend(
            root,
            &runtime,
            serde_json::json!([{"path":"/governanceProfile","operation":"set","value":{"materialInspectionReview":{
                "schemaVersion":1,"permittedUnknown":"repository_material_inspection_unavailable","permittedCause":"readable_committed_rust_syntax_unknown",
                "assurance":"self_declared","reviewerActor":"agent:transaction-fixture","authoritySource":"fixture:old-risk","acceptResidualRisk":true
            }}}]),
        );
        let request = crate::plan_work_item_material_review(root, ITEM).unwrap();
        let mut input: MaterialInspectionReviewDecisionInput = serde_json::from_value(serde_json::json!({
            "schemaVersion":1,"decision":"accept_permitted_unknowns","requestDigest":request.request_digest,
            "reviewerActor":"agent:transaction-fixture","authoritySource":"fixture:old-risk","assurance":"self_declared",
            "evidenceRefs":[{"path":"synthetic-review","digest":Digest::sha256_bytes(b"synthetic review")}],
            "rationale":"Synthetic fixture reviews its exact material.","residualRisk":"Synthetic readable syntax scanner unknown."
        })).unwrap();
        record_work_item_material_review(root, ITEM, &input, &runtime).unwrap();
        let summary: serde_json::Value = serde_json::from_slice(
            &fs::read(root.join(format!(".ai/work-items/active/{ITEM}.summary.json"))).unwrap(),
        )
        .unwrap();
        let old_sidecar = root.join(summary["materialReviewReceipt"]["path"].as_str().unwrap());
        let old_bytes = fs::read(&old_sidecar).unwrap();
        transaction_fixture_amend(
            root,
            &runtime,
            serde_json::json!([
                {"path":"/scope","operation":"add","value":"docs/fixture.md"},
                {"path":"/acceptanceCriteria","operation":"add","value":"retain native mixed history"},
                {"path":"/scenarioCoverage","operation":"add","value":{"scenario":"replacement transaction","required":true,"status":"unverified","evidence":[],"expected":"recover exact pending decision","verificationPlan":"cargo test --lib material_review"}},
                {"path":"/sources","operation":"add","value":{"path":".ai/evidence/synthetic-history","reason":"synthetic native history"}}
            ]),
        );
        transaction_fixture_amend(
            root,
            &runtime,
            serde_json::json!([{"path":"/governanceProfile/materialInspectionReview/authoritySource","operation":"set","value":"fixture:new-risk"}]),
        );
        input.request_digest = crate::plan_work_item_material_review(root, ITEM)
            .unwrap()
            .request_digest;
        input.authority_source = "fixture:new-risk".into();
        (directory, runtime, input, old_sidecar, old_bytes)
    }

    #[test]
    fn replacement_pending_retry_reuses_immutable_receipt() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        let (directory, runtime, input, old_sidecar, old_bytes) = transaction_fixture();
        let root = directory.path();
        let summary = root.join(format!(".ai/work-items/active/{ITEM}.summary.json"));
        let before = fs::read(&summary).unwrap();
        REPLACEMENT_FAILURE.with(|failure| failure.set(Some("after_sidecar")));
        let error = record_work_item_material_review(root, ITEM, &input, &runtime)
            .expect_err("actual injected interruption");
        assert!(error.to_string().contains("injected interruption"));
        assert_eq!(fs::read(&summary).unwrap(), before);
        assert_eq!(fs::read(&old_sidecar).unwrap(), old_bytes);
        let sidecar = root.join(format!(
            ".ai/evidence/material-inspection-review/{ITEM}/{}.json",
            input
                .request_digest
                .as_str()
                .strip_prefix("sha256:")
                .unwrap()
        ));
        let staged =
            fs::read(&sidecar).expect("immutable current sidecar published before interruption");
        let retried = record_work_item_material_review(root, ITEM, &input, &runtime);
        let receipt = match retried {
            Ok(receipt) => receipt,
            Err(error) => {
                let retained = directory.keep();
                panic!(
                    "exact pending retry must reuse staged original receipt; retained {retained:?}: {error}"
                )
            }
        };
        assert_eq!(serde_json::to_vec_pretty(&receipt).unwrap(), staged);
        assert_eq!(fs::read(sidecar).unwrap(), staged);
        assert_eq!(fs::read(old_sidecar).unwrap(), old_bytes);
        record_work_item_material_review(root, ITEM, &input, &runtime)
            .expect_err("committed replay rejects");
    }

    fn transaction_artifacts(root: &Path) -> BTreeMap<String, Vec<u8>> {
        fs::read_dir(
            root.join(".ai/evidence/material-inspection-review/WI-REPLACEMENT-TRANSACTION"),
        )
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (
                entry.file_name().into_string().unwrap(),
                fs::read(entry.path()).unwrap(),
            )
        })
        .collect()
    }

    #[test]
    fn replacement_transaction_failure_matrix() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        for stage in [
            "before_prepare",
            "after_prepare",
            "before_sidecar",
            "after_sidecar",
            "before_summary",
            "after_summary",
        ] {
            let (directory, runtime, input, old_sidecar, old_bytes) = transaction_fixture();
            let root = directory.path();
            let summary_path = root.join(format!(".ai/work-items/active/{ITEM}.summary.json"));
            let before = fs::read(&summary_path).unwrap();
            let old_pointer=serde_json::from_slice::<serde_json::Value>(&before).unwrap()["materialReviewReceipt"].clone();
            REPLACEMENT_FAILURE.with(|failure| failure.set(Some(stage)));
            let result = record_work_item_material_review(root, ITEM, &input, &runtime);
            let receipt = if stage == "after_summary" {
                result.expect("post-rename error is resolved from actual committed canonical state")
            } else {
                let error = result.expect_err("exact injected publication boundary must interrupt");
                assert!(
                    error.to_string().contains("injected interruption"),
                    "{stage}: {error}"
                );
                assert_eq!(fs::read(&summary_path).unwrap(), before, "{stage}");
                let status =
                    crate::work_item_status_snapshot_with_runtime(root, ITEM, &runtime).unwrap();
                assert!(
                    status.review_receipt_digest.is_none(),
                    "staged evidence cannot discharge at {stage}"
                );
                assert!(
                    status
                        .effective_unknowns
                        .contains(&"repository_material_inspection_unavailable".into())
                );
                let names = transaction_artifacts(root);
                if stage == "before_prepare" {
                    assert_eq!(names.len(), 1);
                }
                if ["after_prepare", "before_sidecar"].contains(&stage) {
                    assert_eq!(names.len(), 2);
                }
                record_work_item_material_review(root, ITEM, &input, &runtime)
                    .expect("exact pending continuation commits")
            };
            assert_eq!(fs::read(&old_sidecar).unwrap(), old_bytes);
            let committed = fs::read(&summary_path).unwrap();
            let summary: serde_json::Value = serde_json::from_slice(&committed).unwrap();
            assert_eq!(
                summary["materialReviewReceiptHistory"],
                serde_json::json!([old_pointer])
            );
            let sidecar = root.join(summary["materialReviewReceipt"]["path"].as_str().unwrap());
            assert_eq!(
                fs::read(sidecar).unwrap(),
                serde_json::to_vec_pretty(&receipt).unwrap()
            );
            let artifacts = transaction_artifacts(root);
            record_work_item_material_review(root, ITEM, &input, &runtime)
                .expect_err("committed replay must not write");
            assert_eq!(transaction_artifacts(root), artifacts);
            assert_eq!(fs::read(summary_path).unwrap(), committed);
        }
    }

    #[test]
    fn replacement_pending_rejects_wrong_input_runtime_and_rehashed_target() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        let (directory, runtime, input, _old_sidecar, _old_bytes) = transaction_fixture();
        let root = directory.path();
        let summary_path = root.join(format!(".ai/work-items/active/{ITEM}.summary.json"));
        let before = fs::read(&summary_path).unwrap();
        REPLACEMENT_FAILURE.with(|failure| failure.set(Some("after_prepare")));
        record_work_item_material_review(root, ITEM, &input, &runtime)
            .expect_err("retain actual sealed pending");
        let original = transaction_artifacts(root);
        let mut wrong = input.clone();
        wrong.rationale.push_str(" different decision");
        record_work_item_material_review(root, ITEM, &wrong, &runtime)
            .expect_err("wrong input cannot consume pending");
        let mut wrong_runtime = runtime.clone();
        wrong_runtime.runtime_digest = Digest::sha256_bytes(b"different fixture runtime");
        record_work_item_material_review(root, ITEM, &input, &wrong_runtime)
            .expect_err("wrong runtime cannot consume pending");
        assert_eq!(transaction_artifacts(root), original);
        assert_eq!(fs::read(&summary_path).unwrap(), before);
        let name = format!(
            "{}.prepare.json",
            input
                .request_digest
                .as_str()
                .strip_prefix("sha256:")
                .unwrap()
        );
        let path = root
            .join(".ai/evidence/material-inspection-review")
            .join(ITEM)
            .join(&name);
        let mut envelope: PreparedMaterialReplacement =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        envelope.prepared.target_summary["injectedForeignSummaryField"] = serde_json::json!(true);
        envelope.prepared_digest = replacement_prepared_digest(&envelope.prepared).unwrap();
        fs::write(&path, serde_json::to_vec_pretty(&envelope).unwrap()).unwrap();
        let corrupted = transaction_artifacts(root);
        record_work_item_material_review(root, ITEM, &input, &runtime)
            .expect_err("rehashed target is not a valid sealed current decision");
        assert_eq!(transaction_artifacts(root), corrupted);
        assert_eq!(fs::read(summary_path).unwrap(), before);
    }

    #[test]
    fn replacement_rechecks_source_review_and_summary_before_commit() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        for mutation in ["summary", "source", "review", "active-directory"] {
            let (directory, runtime, input, old_sidecar, old_bytes) = transaction_fixture();
            let root = directory.path();
            let summary_path = root.join(format!(".ai/work-items/active/{ITEM}.summary.json"));
            let before = fs::read(&summary_path).unwrap();
            let mut changed: serde_json::Value = serde_json::from_slice(&before).unwrap();
            let old_pointer = changed["materialReviewReceipt"].clone();
            let mutation_root = root.to_path_buf();
            let mutation_summary = summary_path.clone();
            match mutation {
                "summary" => changed["concurrentFixtureNote"] = serde_json::json!(true),
                "review" => {
                    changed["decisionEvidence"]["recordedBy"] =
                        serde_json::json!("human:unbound-fixture")
                }
                _ => {}
            }
            let expected = if ["source", "active-directory"].contains(&mutation) {
                before.clone()
            } else {
                serde_json::to_vec_pretty(&changed).unwrap()
            };
            let callback_bytes = expected.clone();
            REPLACEMENT_BEFORE_COMMIT.with(|callback| {
                *callback.borrow_mut() = Some(Box::new(move || {
                    if mutation == "source" {
                        fs::write(
                            mutation_root.join("README.md"),
                            "source changed before commit\n",
                        )
                        .unwrap();
                    } else if mutation == "active-directory" {
                        let active = mutation_root.join(".ai/work-items/active");
                        let previous =
                            mutation_root.join(".ai/work-items/retained-active-before-swap");
                        fs::rename(&active, &previous).unwrap();
                        fs::create_dir(&active).unwrap();
                        for entry in fs::read_dir(&previous).unwrap() {
                            let entry = entry.unwrap();
                            fs::copy(entry.path(), active.join(entry.file_name())).unwrap();
                        }
                    } else {
                        fs::write(mutation_summary, callback_bytes).unwrap();
                    }
                }))
            });
            let result = record_work_item_material_review(root, ITEM, &input, &runtime);
            let error = match result {
                Err(error) => error,
                Ok(_) => {
                    let retained = directory.keep();
                    panic!(
                        "actual precommit mutation {mutation} must reject, retained {retained:?}"
                    )
                }
            };
            if mutation == "summary" {
                assert!(error.to_string().contains("compare-and-swap"), "{error}");
            }
            assert_eq!(
                fs::read(&summary_path).unwrap(),
                expected,
                "producer must preserve concurrent state"
            );
            let current: serde_json::Value =
                serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
            assert_eq!(current["materialReviewReceipt"], old_pointer);
            assert_eq!(fs::read(old_sidecar).unwrap(), old_bytes);
            assert_eq!(
                transaction_artifacts(root).len(),
                3,
                "staged evidence remains pending, not silently deleted"
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
