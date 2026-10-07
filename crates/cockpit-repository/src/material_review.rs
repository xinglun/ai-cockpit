//! Read-only, committed-source material identity. No decision or discharge is
//! performed here; the opt-in is only reported to a future review service.

use super::{
    contains_strong_instruction_injection, derive_governance_signals_with_diagnostics,
    effective_policy_for_contract, repository_id,
};
use crate::rust_material::{MaterialUnknownCause, RustMaterialAssessment};
#[cfg(windows)]
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
#[cfg(windows)]
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use cockpit_core::Digest;
use cockpit_git::{
    BoundedGitOutput, ChangeContentState, ChangeKind, GitError, GitRepository,
    MAX_BOUNDED_GIT_OUTPUT_BYTES, MAX_CHANGE_TEXT_BYTES,
};
use cockpit_protocol::{
    Contract, MATERIAL_INSPECTION_REVIEW_CAPABILITY,
    MATERIAL_INSPECTION_REVIEW_DECISION_SCHEMA_VERSION, MaterialInspectionReviewAssurance,
    MaterialInspectionReviewDecision, MaterialInspectionReviewDecisionInput,
    MaterialInspectionReviewDecisionReceipt, digest_json,
};
use serde::Serialize;
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
    pub blocked_by_finding: bool,
    pub review_enabled: bool,
    pub review_diagnostic: Option<String>,
    pub request_digest: Digest,
    /// Provenance only. It is excluded from request_digest so an identical
    /// source manifest in a legitimate descendant remains comparable.
    pub reviewed_source_head: String,
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
    let contract_digest = json_digest(contract)
        .map_err(|_| MaterialReviewDecisionValidationError::RequestIdentityMismatch)?;
    if request.schema_version != 1
        || request.repository_id != contract.repository_id
        || request.work_item_id != contract.work_item_id
        || request.contract_digest != contract_digest
        || request.immutable_contract_base_revision != contract.base_revision
        || request.material_inspection_review_profile_digest.as_ref() != Some(&profile_digest)
    {
        return Err(MaterialReviewDecisionValidationError::RequestIdentityMismatch);
    }
    if request.blocked_by_finding
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

pub fn material_review_request(
    root: &Path,
    contract: &Contract,
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
        let changed_hunk_digest =
            json_digest(&(&change.added_lines, &origins, &change.removed_lines))?;
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
    let mut request = MaterialReviewRequest {
        schema_version: 1,
        repository_id: contract.repository_id.clone(),
        work_item_id: contract.work_item_id.clone(),
        contract_digest: json_digest(contract)?,
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
        blocked_by_finding: !signals.findings.is_empty(),
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

#[cfg(test)]
mod tests {
    use super::*;

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
