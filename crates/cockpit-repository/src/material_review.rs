//! Committed material identity and exact opt-in review decisions. Historical
//! replacement eligibility never makes an invalid old receipt current.

use super::{
    ObserverError, acquire_lifecycle_lock, create_and_open_cap_directory,
    derive_governance_signals_with_diagnostics, open_cap_directory_nofollow_strict,
    read_cap_file_nofollow_bounded, repository_id, require_current_action_admission,
    validate_work_item_id,
};
use crate::rust_material::MaterialUnknownCause;
#[cfg(windows)]
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
#[cfg(windows)]
use cap_std::fs::OpenOptions as CapOpenOptions;
use cap_std::{ambient_authority, fs::Dir};
use cockpit_core::Digest;
use cockpit_git::{
    ChangeEvidence, GitRepository, MAX_BOUNDED_GIT_OUTPUT_BYTES, RepositorySnapshot,
};
use cockpit_protocol::{
    Contract, MATERIAL_INSPECTION_REVIEW_CAPABILITY, MaterialInspectionReviewAssurance,
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

const MAX_MATERIAL_BLOB_BYTES: u64 = 16 * 1024 * 1024;
const MAX_MATERIAL_CHANGED_FILES: usize = 256;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_artifact: Option<MaterialReviewRequestArtifactPlan>,
}

impl From<super::material_recipe::RequestV1> for MaterialReviewRequest {
    fn from(value: super::material_recipe::RequestV1) -> Self {
        use super::material_recipe::{ScannerAssessmentV1, UnknownCauseV1};
        Self {
            schema_version: value.schema_version,
            repository_id: value.repository_id,
            work_item_id: value.work_item_id,
            contract_digest: value.contract_digest,
            immutable_contract_base_revision: value.immutable_contract_base_revision,
            source_snapshot_digest: value.source_snapshot_digest,
            material_manifest_digest: value.material_manifest_digest,
            scanner_semantic_version: value.scanner_semantic_version,
            analysis_implementation_digest: value.analysis_implementation_digest,
            analysis_target_semantic_profile: value.analysis_target_semantic_profile,
            material_inspection_review_profile_digest: value
                .material_inspection_review_profile_digest,
            effective_policy_digest: value.effective_policy_digest,
            raw_unknown_codes: value.raw_unknown_codes,
            finding_codes: value.finding_codes,
            blocked_by_finding: value.blocked_by_finding,
            review_enabled: value.review_enabled,
            review_diagnostic: value.review_diagnostic,
            request_digest: value.request_digest,
            reviewed_source_head: value.reviewed_source_head,
            request_artifact: None,
            entries: value
                .entries
                .into_iter()
                .map(|entry| MaterialReviewEntry {
                    path: entry.path,
                    kind: entry.kind,
                    content_state: entry.content_state,
                    scanner_assessment: match entry.scanner_assessment {
                        ScannerAssessmentV1::Clean => MaterialScannerAssessment::Clean,
                        ScannerAssessmentV1::Finding => MaterialScannerAssessment::Finding,
                        ScannerAssessmentV1::Unknown => MaterialScannerAssessment::Unknown,
                    },
                    unknown_cause: entry.unknown_cause.map(|cause| match cause {
                        UnknownCauseV1::MissingCompleteSource => {
                            MaterialUnknownCause::MissingCompleteSource
                        }
                        UnknownCauseV1::NonTextMaterial => MaterialUnknownCause::NonTextMaterial,
                        UnknownCauseV1::SourceOverBudget => MaterialUnknownCause::SourceOverBudget,
                        UnknownCauseV1::InvalidProvenance => {
                            MaterialUnknownCause::InvalidProvenance
                        }
                        UnknownCauseV1::ParseUnavailable => MaterialUnknownCause::ParseUnavailable,
                        UnknownCauseV1::AnalysisIncomplete => {
                            MaterialUnknownCause::AnalysisIncomplete
                        }
                        UnknownCauseV1::ReadableCommittedRustSyntaxUnknown => {
                            MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown
                        }
                    }),
                    reviewable: entry.reviewable,
                    changed_hunk_digest: entry.changed_hunk_digest,
                    after_blob_digest: entry.after_blob_digest,
                })
                .collect(),
        }
    }
}

fn public_request_to_v1(value: &MaterialReviewRequest) -> super::material_recipe::RequestV1 {
    use super::material_recipe::{EntryV1, RequestV1, ScannerAssessmentV1, UnknownCauseV1};
    RequestV1 {
        schema_version: 1,
        repository_id: value.repository_id.clone(),
        work_item_id: value.work_item_id.clone(),
        contract_digest: value.contract_digest.clone(),
        immutable_contract_base_revision: value.immutable_contract_base_revision.clone(),
        source_snapshot_digest: value.source_snapshot_digest.clone(),
        material_manifest_digest: value.material_manifest_digest.clone(),
        scanner_semantic_version: value.scanner_semantic_version.clone(),
        analysis_implementation_digest: value.analysis_implementation_digest.clone(),
        analysis_target_semantic_profile: value.analysis_target_semantic_profile.clone(),
        material_inspection_review_profile_digest: value
            .material_inspection_review_profile_digest
            .clone(),
        effective_policy_digest: value.effective_policy_digest.clone(),
        raw_unknown_codes: value.raw_unknown_codes.clone(),
        finding_codes: value.finding_codes.clone(),
        blocked_by_finding: value.blocked_by_finding,
        review_enabled: value.review_enabled,
        review_diagnostic: value.review_diagnostic.clone(),
        request_digest: value.request_digest.clone(),
        reviewed_source_head: value.reviewed_source_head.clone(),
        entries: value
            .entries
            .iter()
            .map(|entry| EntryV1 {
                path: entry.path.clone(),
                kind: entry.kind.clone(),
                content_state: entry.content_state.clone(),
                scanner_assessment: match entry.scanner_assessment {
                    MaterialScannerAssessment::Clean => ScannerAssessmentV1::Clean,
                    MaterialScannerAssessment::Finding => ScannerAssessmentV1::Finding,
                    MaterialScannerAssessment::Unknown => ScannerAssessmentV1::Unknown,
                },
                unknown_cause: entry.unknown_cause.map(UnknownCauseV1::from),
                reviewable: entry.reviewable,
                changed_hunk_digest: entry.changed_hunk_digest.clone(),
                after_blob_digest: entry.after_blob_digest.clone(),
            })
            .collect(),
    }
}
/// Exact canonical bytes presented for a V2 material review. The artifact
/// contains no self-reference; decision inputs bind its digest and byte length.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialReviewRequestArtifactPlan {
    pub canonical_json: String,
    pub digest: Digest,
    pub byte_length: u64,
}

fn public_request_from_sealed_v2(
    sealed: &super::material_recipe::SealedRequestV2,
) -> Result<MaterialReviewRequest, MaterialReviewRequestError> {
    let fixed = &sealed.request;
    let mut response: MaterialReviewRequest = fixed.to_v1().into();
    response.schema_version = 2;
    response.request_digest = fixed.request_digest.clone();
    response.request_artifact = Some(MaterialReviewRequestArtifactPlan {
        canonical_json: String::from_utf8(sealed.canonical_bytes.clone())
            .map_err(|e| MaterialReviewRequestError::Identity(e.to_string()))?,
        digest: sealed.raw_digest.clone(),
        byte_length: sealed.canonical_bytes.len() as u64,
    });
    Ok(response)
}

fn sealed_v2_from_public_request(
    request: &MaterialReviewRequest,
) -> Result<super::material_recipe::SealedRequestV2, MaterialReviewRequestError> {
    let fail = |message: String| MaterialReviewRequestError::Identity(message);
    if request.schema_version != 2 {
        return Err(fail("V2 request schema is required".into()));
    }
    let artifact = request
        .request_artifact
        .as_ref()
        .ok_or_else(|| fail("V2 canonical request artifact missing".into()))?;
    let sealed =
        super::material_recipe::decode_v2(artifact.canonical_json.as_bytes()).map_err(&fail)?;
    let projected = public_request_to_v1(request);
    let expected =
        super::material_recipe::seal_v2(super::material_recipe::RequestV2::from_v1(projected))
            .map_err(&fail)?;
    if sealed.canonical_bytes != expected.canonical_bytes
        || sealed.raw_digest != artifact.digest
        || artifact.byte_length != sealed.canonical_bytes.len() as u64
        || request.request_digest != sealed.request.request_digest
    {
        return Err(fail(
            "V2 response fields or raw artifact binding differ".into(),
        ));
    }
    Ok(sealed)
}
fn v2_artifact_reference(
    request: &MaterialReviewRequest,
) -> Result<cockpit_protocol::MaterialInspectionReviewRequestArtifact, MaterialReviewRequestError> {
    let sealed = sealed_v2_from_public_request(request)?;
    let request_hex = sealed
        .request
        .request_digest
        .as_str()
        .strip_prefix("sha256:")
        .ok_or_else(|| MaterialReviewRequestError::Identity("invalid V2 request digest".into()))?;
    let raw_hex = sealed
        .raw_digest
        .as_str()
        .strip_prefix("sha256:")
        .ok_or_else(|| MaterialReviewRequestError::Identity("invalid V2 artifact digest".into()))?;
    let reference = cockpit_protocol::MaterialInspectionReviewRequestArtifact {
        path: format!(
            ".ai/evidence/{MATERIAL_REVIEW_EVIDENCE_DIRECTORY}/{}/{request_hex}.{raw_hex}.request.json",
            request.work_item_id
        ),
        raw_digest: sealed.raw_digest.clone(),
        byte_length: sealed.canonical_bytes.len() as u64,
        request_digest: sealed.request.request_digest.clone(),
        recipe_id: super::material_recipe::REQUEST_RECIPE_V2.into(),
    };
    reference
        .validate_for(&request.work_item_id, &request.request_digest)
        .map_err(MaterialReviewRequestError::Identity)?;
    Ok(reference)
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

fn json_digest(value: &impl Serialize) -> Result<Digest, MaterialReviewRequestError> {
    digest_json(value).map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))
}

fn material_review_request_digest(
    request: &MaterialReviewRequest,
) -> Result<Digest, MaterialReviewRequestError> {
    match request.schema_version {
        1 if request.request_artifact.is_none() => {
            super::material_recipe::request_digest_v1(&public_request_to_v1(request))
                .map_err(MaterialReviewRequestError::Identity)
        }
        2 => Ok(sealed_v2_from_public_request(request)?
            .request
            .request_digest),
        _ => Err(MaterialReviewRequestError::Identity(
            "unsupported material request schema/artifact".into(),
        )),
    }
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
    if !matches!(request.schema_version, 1 | 2)
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
        .map_err(|_| MaterialReviewDecisionValidationError::RequestDigestMismatch)?;
    if !valid_digest(&request.request_digest)
        || canonical_request_digest != request.request_digest
        || input.request_digest != request.request_digest
        || !valid_digest(&input.request_digest)
    {
        return Err(MaterialReviewDecisionValidationError::RequestDigestMismatch);
    }
    input
        .validate_request_binding(
            request.schema_version,
            &request.request_digest,
            request
                .request_artifact
                .as_ref()
                .map(|artifact| (&artifact.digest, artifact.byte_length)),
        )
        .map_err(|_| MaterialReviewDecisionValidationError::InvalidInput)?;
    if input.schema_version != request.schema_version
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
        schema_version: request.schema_version,
        repository_id: request.repository_id.clone(),
        work_item_id: request.work_item_id.clone(),
        contract_digest: request.contract_digest.clone(),
        material_manifest_digest: request.material_manifest_digest.clone(),
        profile_digest,
        request_digest: request.request_digest.clone(),
        request_artifact: if request.schema_version == 2 {
            Some(v2_artifact_reference(request).map_err(|error| {
                MaterialReviewDecisionValidationError::ReceiptIntegrity(error.to_string())
            })?)
        } else {
            None
        },
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
    hasher.update(b"ai-cockpit:material-analysis-implementation:v2\0");
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
            "crates/cockpit-repository/src/material_recipe.rs",
            include_bytes!("material_recipe.rs"),
        ),
        (
            "crates/cockpit-repository/src/material_analysis_v1.rs",
            include_bytes!("material_analysis_v1.rs"),
        ),
        (
            "crates/cockpit-git/src/material_snapshot_v1.rs",
            include_bytes!("../../cockpit-git/src/material_snapshot_v1.rs"),
        ),
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

pub(super) fn verify_checkout_source(
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

fn read_active_contract_bytes(
    root: &Path,
    work_item_id: &str,
) -> Result<Vec<u8>, MaterialReviewRequestError> {
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
    Ok(bytes)
}

fn read_active_contract_document(
    root: &Path,
    work_item_id: &str,
) -> Result<super::CanonicalContractDocument, MaterialReviewRequestError> {
    let bytes = read_active_contract_bytes(root, work_item_id)?;
    let path = root.join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
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
    let implementation = analysis_implementation_digest();
    let request = super::material_analysis_v1::build_request_v1_from_current(
        root,
        contract,
        contract_digest,
        &implementation,
    )?;
    let sealed =
        super::material_recipe::seal_v2(super::material_recipe::RequestV2::from_v1(request))
            .map_err(MaterialReviewRequestError::Identity)?;
    public_request_from_sealed_v2(&sealed)
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
    let Ok(changed_hunk_digest) = super::material_analysis_v1::changed_hunk_digest(change) else {
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

fn receipt_name_from_pointer(
    pointer: &serde_json::Value,
    work_item_id: &str,
    request: &Digest,
) -> Result<String, String> {
    let request_hex = request
        .as_str()
        .strip_prefix("sha256:")
        .ok_or("invalid pointer request digest")?;
    let prefix = format!(".ai/evidence/{MATERIAL_REVIEW_EVIDENCE_DIRECTORY}/{work_item_id}/");
    let name = pointer["path"]
        .as_str()
        .and_then(|path| path.strip_prefix(&prefix))
        .ok_or("material-review pointer is outside its exact Work Item")?;
    if name == format!("{request_hex}.json") {
        return Ok(name.to_owned());
    }
    let raw = name
        .strip_prefix(&format!("{request_hex}."))
        .and_then(|tail| tail.strip_suffix(".receipt.json"))
        .ok_or("material-review pointer path does not bind request/artifact digests")?;
    format!("sha256:{raw}")
        .parse::<Digest>()
        .map_err(|_| "invalid pointer artifact digest")?;
    Ok(name.to_owned())
}

fn receipt_file_name(receipt: &MaterialInspectionReviewDecisionReceipt) -> Result<String, String> {
    let request = receipt
        .request_digest
        .as_str()
        .strip_prefix("sha256:")
        .ok_or("invalid receipt request digest")?;
    match (receipt.schema_version, receipt.request_artifact.as_ref()) {
        (1, None) => Ok(format!("{request}.json")),
        (2, Some(reference)) => {
            reference.validate_for(&receipt.work_item_id, &receipt.request_digest)?;
            let raw = reference
                .raw_digest
                .as_str()
                .strip_prefix("sha256:")
                .ok_or("invalid receipt artifact digest")?;
            Ok(format!("{request}.{raw}.receipt.json"))
        }
        _ => Err("unsupported material-review receipt schema/artifact".into()),
    }
}

/// Read the original artifact without substituting this consumer's provenance.
fn read_receipt_artifact_v2(
    root: &Path,
    receipt: &MaterialInspectionReviewDecisionReceipt,
) -> Result<Option<super::material_recipe::SealedRequestV2>, ObserverError> {
    if receipt.schema_version == 1 && receipt.request_artifact.is_none() {
        return Ok(None);
    }
    let fail = |message: String| replacement_error(root, message);
    if receipt.schema_version != 2 {
        return Err(fail("unsupported artifact receipt version".into()));
    }
    let reference = receipt
        .request_artifact
        .as_ref()
        .ok_or_else(|| fail("V2 receipt artifact reference missing".into()))?;
    reference
        .validate_for(&receipt.work_item_id, &receipt.request_digest)
        .map_err(&fail)?;
    let directory = material_review_work_item_directory(root, &receipt.work_item_id, false)?;
    let name = Path::new(&reference.path)
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| fail("invalid artifact file name".into()))?;
    let path = root.join(&reference.path);
    let bytes = read_cap_file_nofollow_bounded(
        &directory,
        name,
        &path,
        super::material_recipe::ARTIFACT_LIMIT as u64,
    )?;
    if bytes.len() as u64 != reference.byte_length
        || Digest::sha256_bytes(&bytes) != reference.raw_digest
    {
        return Err(fail(
            "stored material artifact raw digest or byte length differs".into(),
        ));
    }
    let sealed = super::material_recipe::decode_v2(&bytes).map_err(&fail)?;
    let request = &sealed.request;
    if request.request_digest != receipt.request_digest
        || request.repository_id != receipt.repository_id
        || request.work_item_id != receipt.work_item_id
        || request.contract_digest != receipt.contract_digest
        || request.material_manifest_digest != receipt.material_manifest_digest
        || request.material_inspection_review_profile_digest.as_ref()
            != Some(&receipt.profile_digest)
        || receipt.reviewed_source_head.as_deref() != Some(request.reviewed_source_head.as_str())
    {
        return Err(fail(
            "stored material artifact differs from its receipt identity".into(),
        ));
    }
    Ok(Some(sealed))
}

struct MaterialReviewReceiptDocument {
    root: PathBuf,
    summary: serde_json::Value,
    pointer: serde_json::Value,
    sidecar_path: PathBuf,
    sidecar_bytes: Vec<u8>,
    pointer_request_digest: Digest,
}

struct BoundMaterialReviewReceipt {
    root: PathBuf,
    summary: serde_json::Value,
    pointer: serde_json::Value,
    receipt: MaterialInspectionReviewDecisionReceipt,
    request_artifact: Option<super::material_recipe::SealedRequestV2>,
    sidecar_path: PathBuf,
    stale_request: bool,
}

/// Only immutable file/pointer integrity. Current-profile validity is kept in
/// the strict reader below; historical integrity alone grants no discharge.
fn read_material_review_receipt_document(
    root: &Path,
    contract: &Contract,
    summary_path: &Path,
) -> Result<Option<MaterialReviewReceiptDocument>, ObserverError> {
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
    let file_digest = pointer.get("digest").and_then(serde_json::Value::as_str);
    let request_digest = pointer
        .get("requestDigest")
        .and_then(serde_json::Value::as_str);
    let pointer_request_digest = request_digest
        .and_then(|digest| digest.parse::<Digest>().ok())
        .ok_or_else(|| ObserverError::State {
            path: summary_path.to_path_buf(),
            message: "material-review Summary request digest is invalid".into(),
        })?;
    let sidecar_name =
        receipt_name_from_pointer(pointer, &contract.work_item_id, &pointer_request_digest)
            .map_err(|message| ObserverError::State {
                path: summary_path.to_owned(),
                message,
            })?;
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
    Ok(Some(MaterialReviewReceiptDocument {
        root,
        pointer: pointer.clone(),
        summary,
        sidecar_path,
        sidecar_bytes,
        pointer_request_digest,
    }))
}

fn read_material_review_receipt_binding(
    root: &Path,
    contract: &Contract,
    request: &MaterialReviewRequest,
    summary_path: &Path,
) -> Result<Option<BoundMaterialReviewReceipt>, ObserverError> {
    let Some(MaterialReviewReceiptDocument {
        root,
        pointer,
        summary,
        sidecar_path,
        sidecar_bytes,
        pointer_request_digest,
    }) = read_material_review_receipt_document(root, contract, summary_path)?
    else {
        return Ok(None);
    };
    let stale_request = pointer_request_digest != request.request_digest;
    let receipt_digest = pointer
        .get("receiptDigest")
        .and_then(serde_json::Value::as_str);
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
    let expected_name =
        receipt_file_name(&receipt).map_err(|message| replacement_error(&sidecar_path, message))?;
    if sidecar_path.file_name().and_then(|s| s.to_str()) != Some(expected_name.as_str()) {
        return Err(replacement_error(
            &sidecar_path,
            "receipt version/artifact does not match pointer path",
        ));
    }
    let request_artifact = read_receipt_artifact_v2(&root, &receipt)?;
    Ok(Some(BoundMaterialReviewReceipt {
        root,
        pointer: pointer.clone(),
        summary,
        receipt,
        request_artifact,
        sidecar_path,
        stale_request,
    }))
}

/// Historical V1 consumes its original wire recipe and raw canonical Contract.
/// It cannot grant a material decision for current source development.
pub(super) fn historical_material_review_v1_projection(
    root: &Path,
    contract: &Contract,
    summary_path: &Path,
) -> Result<Option<MaterialReviewGateProjection>, ObserverError> {
    let Some(document) = read_material_review_receipt_document(root, contract, summary_path)?
    else {
        return Ok(None);
    };
    let fail = |message: String| ObserverError::State {
        path: summary_path.to_owned(),
        message,
    };
    super::reject_duplicate_json_keys(&document.sidecar_bytes).map_err(&fail)?;
    let version: serde_json::Value =
        serde_json::from_slice(&document.sidecar_bytes).map_err(|error| fail(error.to_string()))?;
    match version["schemaVersion"].as_u64() {
        Some(1) => {}
        Some(2) => return Ok(None), // supported current V2 engine, never a V1 fallback
        _ => {
            return Err(fail(
                "unsupported historical material receipt schema".into(),
            ));
        }
    }
    let receipt =
        super::material_recipe::decode_receipt_v1(&document.sidecar_bytes).map_err(&fail)?;
    if document.pointer["receiptDigest"].as_str() != Some(receipt.receipt_digest.as_str())
        || receipt.request_digest != document.pointer_request_digest
        || receipt.repository_id != contract.repository_id
        || receipt.work_item_id != contract.work_item_id
    {
        return Err(fail(
            "historical material receipt does not match its Summary identity".into(),
        ));
    }
    let reviewed = receipt
        .reviewed_source_head
        .as_deref()
        .ok_or_else(|| fail("historical reviewed source head missing".into()))?;
    let raw_contract = read_active_contract_bytes(&document.root, &contract.work_item_id)
        .map_err(|e| fail(e.to_string()))?;
    let request = super::material_analysis_v1::rebuild_historical_request_v1(
        &document.root,
        &raw_contract,
        reviewed,
    )
    .map_err(|e| fail(e.to_string()))?;
    super::material_analysis_v1::validate_legacy_decision_v1(&raw_contract, &request, &receipt)
        .map_err(|e| fail(e.to_string()))?;
    Ok(Some(MaterialReviewGateProjection {
        raw_scanner_unknowns: request.raw_unknown_codes,
        material_manifest_digest: Some(request.material_manifest_digest),
        review_receipt_digest: Some(receipt.receipt_digest),
        review_assurance: Some(MaterialInspectionReviewAssurance::SelfDeclared),
        effective_unknowns: Vec::new(),
        discharged_unknowns: vec!["repository_material_inspection_unavailable".into()],
        reviewed_source_head: Some(reviewed.to_owned()),
        reviewed_unknown_members: Vec::new(),
        finding_codes: request.finding_codes,
        blocked_by_finding: request.blocked_by_finding,
        projection_unavailable: false,
        review_decision_available: false,
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
        request_artifact,
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
        request_artifact_digest: receipt
            .request_artifact
            .as_ref()
            .map(|artifact| artifact.raw_digest.clone()),
        request_artifact_byte_length: receipt
            .request_artifact
            .as_ref()
            .map(|artifact| artifact.byte_length),
        reviewer_actor: receipt.reviewer_actor.clone(),
        authority_source: receipt.authority_source.clone(),
        assurance: receipt.assurance,
        evidence_refs: receipt.evidence_refs.clone(),
        rationale: receipt.rationale.clone(),
        residual_risk: receipt.residual_risk.clone(),
    };
    let mut receipt_request = match request_artifact {
        Some(artifact) => public_request_from_sealed_v2(&artifact)
            .map_err(|error| replacement_error(&sidecar_path, error.to_string()))?,
        None => request.clone(),
    };
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
    current_authority_digest: Digest,
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
            let path = format!("{prefix}/{escaped}");
            match (before.get(key), after.get(key)) {
                (Some(before), Some(after)) => {
                    replacement_changed_paths(before, after, &path, paths)
                }
                _ => paths.push(path),
            }
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
        let name = receipt_name_from_pointer(pointer, &contract.work_item_id, &request)
            .map_err(|message| replacement_error(&path, message))?;
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
        if let Some(artifact) = read_receipt_artifact_v2(root, &receipt)? {
            digests.push(artifact.raw_digest);
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
    let authenticated_reviews =
        super::review_authority::historical_preflight_reviews(root, &contract.work_item_id)?;
    // CURRENT is independently checked even for a legacy-shaped transaction.
    // An unavailable, revoked, held or mismatched source never falls back to JSON.
    let current_authority =
        super::review_authority::current_preflight_review(root, &contract.work_item_id)?;
    let legacy_current = bound
        .summary
        .get("decisionEvidence")
        .is_some_and(|v| !v.is_null());
    let mut used_approvals = BTreeSet::new();
    let mut used_source_events = BTreeSet::new();
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
            let amendment_at = chrono::DateTime::parse_from_rfc3339(&item.recorded_at)
                .map_err(|_| replacement_error(&path, "amendment time is invalid"))?;
            let verified = authenticated_reviews.iter().find(|proof| {
                let binding = &proof.receipt().request.binding;
                binding.contract_digest == item.new_contract_digest
                    && binding.repository_snapshot_digest == item.repository_snapshot_digest
                    && proof.decided_at() >= amendment_at
                    && !used_approvals.contains(&proof.approval_identity())
                    && !used_source_events.contains(&proof.source_identity())
            }).ok_or_else(|| replacement_error(&path, "a required historical review has no independently activated exact source proof"))?;
            used_approvals.insert(verified.approval_identity());
            used_source_events.insert(verified.source_identity());
            raw_bindings.push(verified.raw_digest().clone());
            // Legacy JSON is historical structure only. Preserve its original
            // transaction assertions in that version, alongside the separate
            // authenticated proof; a v2-only workflow writes no v1 projection.
            if legacy_current {
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
    let current_binding = &current_authority.receipt().request.binding;
    if observed != tail.repository_snapshot_digest
        || bound.summary["preflightContractDigest"].as_str() != Some(contract_digest.as_str())
        || bound.summary["preflightRepositorySnapshotDigest"].as_str() != Some(observed.as_str())
        || current_binding.contract_digest != *contract_digest
        || current_binding.repository_snapshot_digest != observed
        || current_binding
            .preflight_decision_digest
            .as_ref()
            .map(|d| d.as_str())
            != bound.summary["preflightDecisionDigest"].as_str()
    {
        return Err(replacement_error(
            root,
            "current source or independent v2 review is not bound to adjacent transition",
        ));
    }
    let current_authority_digest = current_authority.proof_digest()?;
    raw_bindings.push(current_authority.raw_digest().clone());
    let current_review_digest = if legacy_current {
        let current_review = &bound.summary["decisionEvidence"];
        if !replacement_review_matches(current_review, contract, contract_digest, &observed)
            || bound.summary["preflightDecisionDigest"] != current_review["preflightDecisionDigest"]
            || !reviews.iter().any(|(value, _)| value == current_review)
        {
            return Err(replacement_error(
                root,
                "legacy transaction review structure is not bound to adjacent transition",
            ));
        }
        json_digest(current_review).map_err(|error| replacement_error(root, error.to_string()))?
    } else {
        current_authority_digest.clone()
    };
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
        current_review_digest,
        current_authority_digest,
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
    super::review_authority::execute_human_operation_with_unavailable_diagnostic(
        root,
        work_item_id,
        cockpit_protocol::review_authority::HumanReviewOperation::MaterialReview,
        &serde_json::json!({"input":input,"runtime":runtime}),
        || record_work_item_material_review_business(root, work_item_id, input, runtime),
        || {
            // This diagnostic can only reject; it supplies no authorization.
            if plan_work_item_material_review(root, work_item_id)
                .is_ok_and(|request| !request.review_enabled)
            {
                Some(ObserverError::State {
                    path: root
                        .join(".ai/work-items/active")
                        .join(format!("{work_item_id}.contract.json")),
                    message: "material review decision is not enabled by the current Contract"
                        .into(),
                })
            } else {
                None
            }
        },
    )
}

fn record_work_item_material_review_business(
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
    require_current_action_admission(
        &root,
        work_item_id,
        "record_material_review_decision",
        runtime,
    )?;
    let replacement_proof = finite_authority_source_replacement_proof(
        &root,
        contract,
        &document.digest,
        &request,
        &root.join(format!(".ai/work-items/active/{work_item_id}.summary.json")),
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
    let summary: serde_json::Value =
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

    if let Some(proof) = replacement_proof.as_ref()
        && prior_pointer.as_ref() != Some(&proof.prior_pointer)
    {
        return Err(replacement_error(
            &summary_path,
            "historical pointer changed before replacement",
        ));
    }
    record_prepared_material_replacement(
        MaterialReplacementContext {
            root: &root,
            active: &active,
            contract,
            contract_digest: &document.digest,
            request: &request,
            input,
            runtime,
            summary_bytes: &summary_bytes,
            proof: replacement_proof.as_ref(),
            prior_pointer: prior_pointer.as_ref(),
            lifecycle_lock: &lifecycle_lock,
        },
        receipt,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MaterialPreparationKind {
    Initial,
    AuthoritySourceReplacement,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MaterialReplacementPreparation {
    schema_version: u32,
    kind: MaterialPreparationKind,
    repository_id: String,
    work_item_id: String,
    contract_digest: Digest,
    request_digest: Digest,
    profile_digest: Digest,
    proof_digest: Option<Digest>,
    prior_pointer: Option<serde_json::Value>,
    input_digest: Digest,
    runtime: RuntimeContext,
    previous_summary_digest: Digest,
    target_summary: serde_json::Value,
    request_artifact: cockpit_protocol::MaterialInspectionReviewRequestArtifact,
    canonical_request_json: String,
    receipt_bytes: String,
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
    proof: Option<&'a AuthoritySourceReplacementProof>,
    prior_pointer: Option<&'a serde_json::Value>,
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
    let fail = |message| replacement_error(context.root, message);
    let mut summary: serde_json::Value =
        serde_json::from_slice(context.summary_bytes).map_err(|error| fail(error.to_string()))?;
    if summary.get("materialReviewReceipt") != context.prior_pointer {
        return Err(fail(
            "Summary no longer binds the previous material pointer".into(),
        ));
    }
    if let Some(previous) = context.prior_pointer {
        let history = summary
            .as_object_mut()
            .ok_or_else(|| fail("Summary must be an object".into()))?
            .entry("materialReviewReceiptHistory")
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .ok_or_else(|| fail("pointer history must be an array".into()))?;
        if history.contains(previous) {
            return Err(fail("historical pointer already occurs in history".into()));
        }
        history.push(previous.clone());
    } else if summary
        .get("materialReviewReceiptHistory")
        .is_some_and(|value| !value.is_array())
    {
        return Err(fail("pointer history must be an array".into()));
    }
    let bytes = serde_json::to_vec_pretty(receipt).map_err(|error| fail(error.to_string()))?;
    let name = receipt_file_name(receipt).map_err(&fail)?;
    summary["materialReviewReceipt"] = serde_json::json!({
        "path":format!(".ai/evidence/{MATERIAL_REVIEW_EVIDENCE_DIRECTORY}/{}/{name}",context.contract.work_item_id),
        "digest":Digest::sha256_bytes(&bytes), "requestDigest":receipt.request_digest, "receiptDigest":receipt.receipt_digest,
    });
    Ok(summary)
}

fn replacement_prepared_digest(
    prepared: &MaterialReplacementPreparation,
) -> Result<Digest, ObserverError> {
    json_digest(&("ai-cockpit:material-review-prepare:v2", prepared))
        .map_err(|error| replacement_error(Path::new("material-review-prepare"), error.to_string()))
}

fn revalidate_material_replacement(
    context: &MaterialReplacementContext<'_>,
    prepared: &MaterialReplacementPreparation,
    evidence: &Dir,
) -> Result<(), ObserverError> {
    context
        .lifecycle_lock
        .require_subject(context.root, &context.contract.work_item_id)?;
    revalidate_replacement_directories(context, evidence)?;
    let document = read_active_contract_document(context.root, &context.contract.work_item_id)
        .map_err(|error| replacement_error(context.root, error.to_string()))?;
    if document.digest != *context.contract_digest || document.contract != *context.contract {
        return Err(replacement_error(
            context.root,
            "current Contract changed during material transaction",
        ));
    }
    let request = material_review_request_with_contract_digest(
        context.root,
        &document.contract,
        &document.digest,
    )
    .map_err(|error| replacement_error(context.root, error.to_string()))?;
    if request.request_digest != context.request.request_digest
        || request
            .request_artifact
            .as_ref()
            .map(|a| (&a.digest, a.byte_length))
            != context
                .request
                .request_artifact
                .as_ref()
                .map(|a| (&a.digest, a.byte_length))
    {
        return Err(replacement_error(
            context.root,
            "current material or exact reviewed artifact changed",
        ));
    }
    let summary_name = format!("{}.summary.json", context.contract.work_item_id);
    let summary_path = context
        .root
        .join(".ai/work-items/active")
        .join(&summary_name);
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
    let proof = finite_authority_source_replacement_proof(
        context.root,
        context.contract,
        context.contract_digest,
        &request,
        &summary_path,
    )?;
    if proof.as_ref() != context.proof
        || proof
            .as_ref()
            .map(json_digest)
            .transpose()
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
    Ok(())
}

fn validate_material_replacement_preparation(
    context: &MaterialReplacementContext<'_>,
    envelope: &PreparedMaterialReplacement,
    evidence: &Dir,
) -> Result<(), ObserverError> {
    let prepared = &envelope.prepared;
    let fail = |message| replacement_error(context.root, message);
    let artifact =
        sealed_v2_from_public_request(context.request).map_err(|e| fail(e.to_string()))?;
    let reference = v2_artifact_reference(context.request).map_err(|e| fail(e.to_string()))?;
    let kind = if context.proof.is_some() {
        MaterialPreparationKind::AuthoritySourceReplacement
    } else {
        MaterialPreparationKind::Initial
    };
    if prepared.schema_version != 2
        || prepared.kind != kind
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
            != context
                .proof
                .map(json_digest)
                .transpose()
                .map_err(|e| fail(e.to_string()))?
        || prepared.prior_pointer.as_ref() != context.prior_pointer
        || prepared.input_digest != json_digest(context.input).map_err(|e| fail(e.to_string()))?
        || prepared.runtime != *context.runtime
        || prepared.previous_summary_digest != Digest::sha256_bytes(context.summary_bytes)
        || prepared.request_artifact != reference
        || prepared.canonical_request_json.as_bytes() != artifact.canonical_bytes
        || envelope.prepared_digest != replacement_prepared_digest(prepared)?
    {
        return Err(fail(
            "sealed preparation is not this exact pending material transaction".into(),
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
    .map_err(|e| fail(e.to_string()))?;
    if expected != prepared.receipt
        || prepared.receipt_bytes.as_bytes()
            != serde_json::to_vec_pretty(&expected).map_err(|e| fail(e.to_string()))?
        || replacement_target_summary(context, &prepared.receipt)? != prepared.target_summary
    {
        return Err(fail(
            "sealed receipt bytes or target Summary differ from the current decision".into(),
        ));
    }
    revalidate_material_replacement(context, prepared, evidence)
}

fn replacement_directory_identity(directory: &Dir, path: &Path) -> Result<[u64; 2], ObserverError> {
    // Handle-derived identifiers also work on Windows, matching the native
    // lifecycle lock's identity model rather than comparing mutable mtimes.
    use cap_fs_ext::MetadataExt as _;
    let metadata = directory
        .dir_metadata()
        .map_err(|source| ObserverError::Read {
            path: path.into(),
            source,
        })?;
    Ok([metadata.dev(), metadata.ino()])
}

fn revalidate_replacement_directories(
    context: &MaterialReplacementContext<'_>,
    evidence: &Dir,
) -> Result<(), ObserverError> {
    let root_dir = Dir::open_ambient_dir(context.root, ambient_authority()).map_err(|source| {
        ObserverError::Read {
            path: context.root.into(),
            source,
        }
    })?;
    let ai_path = context.root.join(".ai");
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &ai_path)?;
    let items_path = ai_path.join("work-items");
    let items = open_cap_directory_nofollow_strict(&ai, "work-items", &items_path)?;
    let active_path = items_path.join("active");
    let active = open_cap_directory_nofollow_strict(&items, "active", &active_path)?;
    if replacement_directory_identity(&active, &active_path)?
        != replacement_directory_identity(context.active, &active_path)?
    {
        return Err(replacement_error(
            &active_path,
            "active directory identity changed before canonical commit",
        ));
    }
    let current_evidence =
        material_review_work_item_directory(context.root, &context.contract.work_item_id, false)?;
    let evidence_path = context
        .root
        .join(".ai/evidence")
        .join(MATERIAL_REVIEW_EVIDENCE_DIRECTORY)
        .join(&context.contract.work_item_id);
    if replacement_directory_identity(&current_evidence, &evidence_path)?
        != replacement_directory_identity(evidence, &evidence_path)?
    {
        return Err(replacement_error(
            &evidence_path,
            "evidence directory identity changed before canonical commit",
        ));
    }
    Ok(())
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
    // Complete artifact bounds and review binding are checked before creating evidence paths.
    let artifact = sealed_v2_from_public_request(context.request)
        .map_err(|error| replacement_error(context.root, error.to_string()))?;
    let reference = v2_artifact_reference(context.request)
        .map_err(|error| replacement_error(context.root, error.to_string()))?;
    let directory =
        material_review_work_item_directory(context.root, &context.contract.work_item_id, true)?;
    let path = context
        .root
        .join(".ai/evidence")
        .join(MATERIAL_REVIEW_EVIDENCE_DIRECTORY)
        .join(&context.contract.work_item_id);
    let sidecar_name =
        receipt_file_name(&proposed_receipt).map_err(|error| replacement_error(&path, error))?;
    let stem = sidecar_name
        .strip_suffix(".receipt.json")
        .ok_or_else(|| replacement_error(&path, "V2 receipt file name required"))?;
    let prepared_name = format!("{stem}.prepare.json");
    let request_name = format!("{stem}.request.json");
    let prepared_path = path.join(&prepared_name);
    let sidecar_path = path.join(&sidecar_name);
    let request_path = path.join(&request_name);
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
        let receipt_bytes = serde_json::to_string_pretty(&proposed_receipt)
            .map_err(|error| replacement_error(&sidecar_path, error.to_string()))?;
        let prepared = MaterialReplacementPreparation {
            schema_version: 2,
            kind: if context.proof.is_some() {
                MaterialPreparationKind::AuthoritySourceReplacement
            } else {
                MaterialPreparationKind::Initial
            },
            repository_id: context.contract.repository_id.clone(),
            work_item_id: context.contract.work_item_id.clone(),
            contract_digest: context.contract_digest.clone(),
            request_digest: context.request.request_digest.clone(),
            profile_digest: proposed_receipt.profile_digest.clone(),
            proof_digest: context
                .proof
                .map(json_digest)
                .transpose()
                .map_err(|error| replacement_error(context.root, error.to_string()))?,
            prior_pointer: context.prior_pointer.cloned(),
            input_digest: json_digest(context.input)
                .map_err(|error| replacement_error(context.root, error.to_string()))?,
            runtime: context.runtime.clone(),
            previous_summary_digest: Digest::sha256_bytes(context.summary_bytes),
            target_summary: replacement_target_summary(&context, &proposed_receipt)?,
            request_artifact: reference,
            canonical_request_json: String::from_utf8(artifact.canonical_bytes.clone())
                .map_err(|error| replacement_error(&request_path, error.to_string()))?,
            receipt_bytes,
            receipt: proposed_receipt,
        };
        PreparedMaterialReplacement {
            prepared_digest: replacement_prepared_digest(&prepared)?,
            prepared,
        }
    };
    validate_material_replacement_preparation(&context, &envelope, &directory)?;
    let prepared_bytes = serde_json::to_vec_pretty(&envelope)
        .map_err(|error| replacement_error(&prepared_path, error.to_string()))?;
    let receipt_bytes = envelope.prepared.receipt_bytes.as_bytes();
    let request_bytes = envelope.prepared.canonical_request_json.as_bytes();
    let target_bytes = serde_json::to_vec_pretty(&envelope.prepared.target_summary)
        .map_err(|error| replacement_error(context.root, error.to_string()))?;
    if [
        prepared_bytes.len(),
        receipt_bytes.len(),
        target_bytes.len(),
    ]
    .into_iter()
    .any(|size| size > MAX_BOUNDED_GIT_OUTPUT_BYTES)
        || request_bytes.len() > super::material_recipe::ARTIFACT_LIMIT
        || pending
            .as_ref()
            .is_some_and(|bytes| bytes != &prepared_bytes)
    {
        return Err(replacement_error(
            &prepared_path,
            "oversized or changed immutable preparation",
        ));
    }
    // Reject pre-existing hostile or conflicting targets before adding any
    // new staged file, including the request stage introduced by V2.
    for (name, path, expected) in [
        (request_name.as_str(), request_path.as_path(), request_bytes),
        (sidecar_name.as_str(), sidecar_path.as_path(), receipt_bytes),
    ] {
        match read_cap_file_nofollow_bounded(&directory, name, path, expected.len() as u64) {
            Ok(existing) if existing == expected => {}
            Ok(_) => {
                return Err(replacement_error(
                    path,
                    "immutable pending material file differs from sealed bytes",
                ));
            }
            Err(ObserverError::Read { source, .. }) if source.kind() == io::ErrorKind::NotFound => {
            }
            Err(error) => return Err(error),
        }
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
    replacement_test_boundary(context.root, "before_request")?;
    super::usage::write_immutable_sidecar(&directory, &request_name, &request_path, request_bytes)?;
    #[cfg(test)]
    replacement_test_boundary(context.root, "after_request")?;
    #[cfg(test)]
    replacement_test_boundary(context.root, "before_sidecar")?;
    super::usage::write_immutable_sidecar(&directory, &sidecar_name, &sidecar_path, receipt_bytes)?;
    #[cfg(test)]
    replacement_test_boundary(context.root, "after_sidecar")?;
    #[cfg(test)]
    REPLACEMENT_BEFORE_COMMIT.with(|callback| {
        if let Some(callback) = callback.borrow_mut().take() {
            callback();
        }
    });
    // Immutable publication does not make concurrent staged-file mutations safe.
    // Verify all exact prepared bytes before the only Summary commit point.
    for (name, path, bytes, limit) in [
        (
            prepared_name.as_str(),
            prepared_path.as_path(),
            prepared_bytes.as_slice(),
            MAX_BOUNDED_GIT_OUTPUT_BYTES as u64,
        ),
        (
            request_name.as_str(),
            request_path.as_path(),
            request_bytes,
            super::material_recipe::ARTIFACT_LIMIT as u64,
        ),
        (
            sidecar_name.as_str(),
            sidecar_path.as_path(),
            receipt_bytes,
            MAX_BOUNDED_GIT_OUTPUT_BYTES as u64,
        ),
    ] {
        if read_cap_file_nofollow_bounded(&directory, name, path, limit)? != bytes {
            return Err(replacement_error(
                path,
                "staged material artifact bytes conflict before Summary commit",
            ));
        }
    }
    revalidate_material_replacement(&context, &envelope.prepared, &directory)?;
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
    revalidate_replacement_directories(&context, &directory)?;
    let canonical = read_cap_file_nofollow_bounded(
        context.active,
        &summary_name,
        &summary_path,
        MAX_BOUNDED_GIT_OUTPUT_BYTES as u64,
    )?;
    if canonical == target_bytes {
        for (name, path, bytes, limit) in [
            (
                prepared_name.as_str(),
                prepared_path.as_path(),
                prepared_bytes.as_slice(),
                MAX_BOUNDED_GIT_OUTPUT_BYTES as u64,
            ),
            (
                request_name.as_str(),
                request_path.as_path(),
                request_bytes,
                super::material_recipe::ARTIFACT_LIMIT as u64,
            ),
            (
                sidecar_name.as_str(),
                sidecar_path.as_path(),
                receipt_bytes,
                MAX_BOUNDED_GIT_OUTPUT_BYTES as u64,
            ),
        ] {
            if read_cap_file_nofollow_bounded(&directory, name, path, limit)? != bytes {
                return Err(replacement_error(
                    path,
                    "committed material artifact bytes conflict",
                ));
            }
        }
        match read_material_review_receipt(
            context.root,
            context.contract,
            context.contract_digest,
            context.request,
            &summary_path,
        )? {
            MaterialReviewReceiptState::Current(receipt)
                if *receipt == envelope.prepared.receipt =>
            {
                return Ok(*receipt);
            }
            _ => {
                return Err(replacement_error(
                    &summary_path,
                    "committed material pointer failed strict canonical readback",
                ));
            }
        }
    }
    if Digest::sha256_bytes(&canonical) != envelope.prepared.previous_summary_digest {
        return Err(replacement_error(
            &summary_path,
            "material commit conflict; inspect canonical state",
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
    use super::record_work_item_material_review_business as record_work_item_material_review;
    use super::*;
    use crate::review_authority::pipeline_tests::TestAuthority;
    // These tests exercise the native transaction primitive, including retries
    // across its own durable prepare/commit cuts. The protected public wrapper
    // has separate no-reexecution tests in review_authority::pipeline_tests.

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
        authority: &TestAuthority,
        legacy_projection: bool,
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
        if legacy_projection {
            authority.approve_preflight_with_synthetic_legacy_projection(root, ITEM);
        } else {
            authority.approve(
                root,
                ITEM,
                cockpit_protocol::review_authority::HumanReviewOperation::PreflightReview,
                &serde_json::json!({"decision":"confirm_review"}),
            );
        }
    }

    type InitialTransactionFixture = (
        tempfile::TempDir,
        RuntimeContext,
        MaterialInspectionReviewDecisionInput,
        TestAuthority,
    );
    fn initial_transaction_fixture() -> InitialTransactionFixture {
        initial_transaction_fixture_with_legacy(true)
    }
    fn initial_transaction_fixture_with_legacy(
        legacy_projection: bool,
    ) -> InitialTransactionFixture {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let authority = TestAuthority::install();
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
            &authority,
            legacy_projection,
            serde_json::json!([{"path":"/governanceProfile","operation":"set","value":{"materialInspectionReview":{
                "schemaVersion":1,"permittedUnknown":"repository_material_inspection_unavailable","permittedCause":"readable_committed_rust_syntax_unknown",
                "assurance":"self_declared","reviewerActor":"agent:transaction-fixture","authoritySource":"fixture:old-risk","acceptResidualRisk":true
            }}}]),
        );
        let request = crate::plan_work_item_material_review(root, ITEM).unwrap();
        let input: MaterialInspectionReviewDecisionInput = serde_json::from_value(serde_json::json!({
            "schemaVersion":2,"decision":"accept_permitted_unknowns","requestDigest":request.request_digest,
            "requestArtifactDigest":request.request_artifact.as_ref().unwrap().digest, "requestArtifactByteLength":request.request_artifact.as_ref().unwrap().byte_length,
            "reviewerActor":"agent:transaction-fixture","authoritySource":"fixture:old-risk","assurance":"self_declared",
            "evidenceRefs":[{"path":"synthetic-review","digest":Digest::sha256_bytes(b"synthetic review")}],
            "rationale":"Synthetic fixture reviews its exact material.","residualRisk":"Synthetic readable syntax scanner unknown."
        })).unwrap();
        (directory, runtime, input, authority)
    }

    type ReplacementTransactionFixture = (
        tempfile::TempDir,
        RuntimeContext,
        MaterialInspectionReviewDecisionInput,
        PathBuf,
        Vec<u8>,
        TestAuthority,
    );
    fn transaction_fixture() -> ReplacementTransactionFixture {
        transaction_fixture_with_legacy(true)
    }
    fn transaction_fixture_with_legacy(legacy_projection: bool) -> ReplacementTransactionFixture {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        let (directory, runtime, mut input, authority) =
            initial_transaction_fixture_with_legacy(legacy_projection);
        let root = directory.path();
        if legacy_projection {
            record_work_item_material_review(root, ITEM, &input, &runtime).unwrap();
        } else {
            authority.approve(
                root,
                ITEM,
                cockpit_protocol::review_authority::HumanReviewOperation::MaterialReview,
                &serde_json::json!({"input":input,"runtime":runtime}),
            );
            crate::record_work_item_material_review(root, ITEM, &input, &runtime).unwrap();
        }
        let summary: serde_json::Value = serde_json::from_slice(
            &fs::read(root.join(format!(".ai/work-items/active/{ITEM}.summary.json"))).unwrap(),
        )
        .unwrap();
        let old_sidecar = root.join(summary["materialReviewReceipt"]["path"].as_str().unwrap());
        let old_bytes = fs::read(&old_sidecar).unwrap();
        transaction_fixture_amend(
            root,
            &runtime,
            &authority,
            legacy_projection,
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
            &authority,
            legacy_projection,
            serde_json::json!([{"path":"/governanceProfile/materialInspectionReview/authoritySource","operation":"set","value":"fixture:new-risk"}]),
        );
        let request = crate::plan_work_item_material_review(root, ITEM).unwrap();
        input.request_digest = request.request_digest;
        input.request_artifact_digest =
            Some(request.request_artifact.as_ref().unwrap().digest.clone());
        input.request_artifact_byte_length =
            Some(request.request_artifact.as_ref().unwrap().byte_length);
        input.authority_source = "fixture:new-risk".into();
        (directory, runtime, input, old_sidecar, old_bytes, authority)
    }

    /// Optional explicit diagnostic export; the ordinary test always runs its
    /// complete assertions. No old Runtime, human or production writer is impersonated.
    fn export_pinned_runtime_material_probe(
        root: &Path,
        request: &MaterialReviewRequest,
        input: &MaterialInspectionReviewDecisionInput,
        receipt: &MaterialInspectionReviewDecisionReceipt,
    ) {
        let Some(destination) = std::env::var_os("AI_COCKPIT_MATERIAL_PROBE_EXPORT") else {
            return;
        };
        let destination = PathBuf::from(destination);
        assert!(
            !destination.exists(),
            "diagnostic export never overwrites evidence"
        );
        let binary = PathBuf::from(
            std::env::var_os("AI_COCKPIT_INTERFACE_DESCRIPTION_BIN")
                .expect("explicit pinned Runtime is required"),
        );
        let binary_digest = Digest::sha256_bytes(&fs::read(&binary).unwrap());
        assert_eq!(
            binary_digest.as_str(),
            "sha256:e967d02a49806d0481272625bdc9813dc913a7aca1773aa734d4a095367b0a98"
        );
        fn copy_fixture(from: &Path, to: &Path) {
            fs::create_dir(to).unwrap();
            for entry in fs::read_dir(from).unwrap() {
                let entry = entry.unwrap();
                let meta = fs::symlink_metadata(entry.path()).unwrap();
                assert!(!meta.file_type().is_symlink());
                if meta.is_dir() {
                    copy_fixture(&entry.path(), &to.join(entry.file_name()));
                } else {
                    fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
                }
            }
        }
        fs::create_dir_all(&destination).unwrap();
        let control = destination.join("v1-control");
        let v2 = destination.join("v2-original");
        let alias = destination.join("v2-legacy-path");
        copy_fixture(root, &control);
        copy_fixture(root, &v2);
        copy_fixture(root, &alias);
        let run = |label: &str, repo: &Path, tail: &[&str]| -> serde_json::Value {
            let mut args = vec!["work-item".to_owned()];
            args.extend(tail.iter().map(|s| s.to_string()));
            args.extend([
                "--repo".into(),
                repo.display().to_string(),
                "--id".into(),
                request.work_item_id.clone(),
            ]);
            let started = chrono::Utc::now().to_rfc3339();
            let output = std::process::Command::new(&binary)
                .args(&args)
                .output()
                .unwrap();
            fs::write(
                destination.join(format!("{label}.stdout.actual.json")),
                &output.stdout,
            )
            .unwrap();
            fs::write(
                destination.join(format!("{label}.stderr.actual.txt")),
                &output.stderr,
            )
            .unwrap();
            fs::write(destination.join(format!("{label}.execution.actual.agent-recorded.json")),serde_json::to_vec_pretty(&serde_json::json!({"phase":"synthetic_read_only_pinned_runtime_probe_not_canonical_verification","binary":binary,"binaryDigest":binary_digest,"args":args,"startedAt":started,"finishedAt":chrono::Utc::now().to_rfc3339(),"exitCode":output.status.code()})).unwrap()).unwrap();
            assert!(
                output.status.success(),
                "{label}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            serde_json::from_slice(&output.stdout).unwrap()
        };
        let legacy_plan = run("legacy-v1-plan", &control, &["material-review", "plan"]);
        let fixed: super::super::material_recipe::RequestV1 =
            serde_json::from_value(legacy_plan).unwrap();
        let legacy: MaterialReviewRequest = fixed.into();
        assert_eq!(legacy.schema_version, 1);
        assert_eq!(legacy.repository_id, request.repository_id);
        assert_eq!(legacy.contract_digest, request.contract_digest);
        assert_eq!(
            legacy.source_snapshot_digest,
            request.source_snapshot_digest
        );
        assert_eq!(
            legacy.material_manifest_digest,
            request.material_manifest_digest
        );
        let mut legacy_input = input.clone();
        legacy_input.schema_version = 1;
        legacy_input.request_digest = legacy.request_digest.clone();
        legacy_input.request_artifact_digest = None;
        legacy_input.request_artifact_byte_length = None;
        legacy_input.rationale.push_str(" Synthetic V1 control is encoded by the current pure V1 validator; it is not an old Runtime writer receipt or a real person's review.");
        let contract = read_active_contract_document(&control, &request.work_item_id).unwrap();
        let legacy_receipt = validate_material_review_decision_with_contract_digest(
            &contract.contract,
            &legacy.contract_digest,
            &legacy,
            &legacy_input,
            "test-fixture:current-pure-V1-validator",
            &receipt.recorded_at,
        )
        .unwrap();
        let summary_path = control.join(format!(
            ".ai/work-items/active/{}.summary.json",
            request.work_item_id
        ));
        let mut summary: serde_json::Value =
            serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
        let legacy_name = receipt_file_name(&legacy_receipt).unwrap();
        let legacy_path = format!(
            ".ai/evidence/{MATERIAL_REVIEW_EVIDENCE_DIRECTORY}/{}/{legacy_name}",
            request.work_item_id
        );
        let legacy_bytes = serde_json::to_vec_pretty(&legacy_receipt).unwrap();
        fs::write(control.join(&legacy_path), &legacy_bytes).unwrap();
        summary["materialReviewReceipt"] = serde_json::json!({"path":legacy_path,"digest":Digest::sha256_bytes(&legacy_bytes),"requestDigest":legacy_receipt.request_digest,"receiptDigest":legacy_receipt.receipt_digest});
        fs::write(&summary_path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
        let positive = run("legacy-v1-positive-status", &control, &["status", "--json"]);
        assert_eq!(
            positive["reviewReceiptDigest"].as_str(),
            Some(legacy_receipt.receipt_digest.as_str()),
            "old process must actually consume its V1 control"
        );
        let negative = run("legacy-v2-original-status", &v2, &["status", "--json"]);
        assert!(negative["reviewReceiptDigest"].is_null());
        assert!(
            negative["effectiveUnknowns"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "material_review_receipt_invalid"),
            "V2 rejection must be receipt invalid, not an unrelated failure: {negative}"
        );
        assert!(
            !negative["effectiveUnknowns"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "material_review_receipt_stale")
        );
        // Keep V2 bytes and valid raw pointer binding, but pass the old path
        // grammar. Decode/integrity rejection precedes semantic-stale handling.
        let alias_name = format!(
            "{}.json",
            receipt
                .request_digest
                .as_str()
                .strip_prefix("sha256:")
                .unwrap()
        );
        let alias_path = format!(
            ".ai/evidence/{MATERIAL_REVIEW_EVIDENCE_DIRECTORY}/{}/{alias_name}",
            request.work_item_id
        );
        let v2_bytes = serde_json::to_vec_pretty(receipt).unwrap();
        fs::write(alias.join(&alias_path), &v2_bytes).unwrap();
        summary["materialReviewReceipt"] = serde_json::json!({"path":alias_path,"digest":Digest::sha256_bytes(&v2_bytes),"requestDigest":receipt.request_digest,"receiptDigest":receipt.receipt_digest});
        fs::write(
            alias.join(format!(
                ".ai/work-items/active/{}.summary.json",
                request.work_item_id
            )),
            serde_json::to_vec_pretty(&summary).unwrap(),
        )
        .unwrap();
        let decoder = run(
            "legacy-v2-canonical-v1-path-status",
            &alias,
            &["status", "--json"],
        );
        assert!(decoder["reviewReceiptDigest"].is_null());
        assert!(
            decoder["effectiveUnknowns"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "material_review_receipt_invalid")
        );
        assert!(
            !decoder["effectiveUnknowns"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "material_review_receipt_stale")
        );
        fs::write(destination.join("probe-assertions.actual.agent-recorded.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "phase":"synthetic_read_only_pinned_runtime_probe_not_authority_or_acceptance","binaryDigest":binary_digest,
            "v1PositiveReceiptDigest":legacy_receipt.receipt_digest,"v2ReceiptDigest":receipt.receipt_digest,
            "v1ControlConsumed":true,"v2OriginalPathRejectedInvalid":true,"v2PayloadWithValidLegacyPathRejectedInvalidNotStale":true,
            "innerDecoderMessageEmittedByStatus":false,"reasonBoundary":"old process classifies invalid before stale; independent frozen old decoder test separately proves exact unknown-field/schema rejection"
        })).unwrap()).unwrap();
    }

    #[test]
    fn pure_v2_protected_reviews_replace_material_without_legacy_decision_evidence() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        let (directory, runtime, input, old_sidecar, old_bytes, authority) =
            transaction_fixture_with_legacy(false);
        let root = directory.path();
        let summary_path = root.join(format!(".ai/work-items/active/{ITEM}.summary.json"));
        assert!(crate::read_json(&summary_path).unwrap()["decisionEvidence"].is_null());
        assert!(
            fs::read_dir(root.join(".ai/decisions"))
                .unwrap()
                .all(|entry| !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .contains(".preflight-review"))
        );
        authority.approve(
            root,
            ITEM,
            cockpit_protocol::review_authority::HumanReviewOperation::MaterialReview,
            &serde_json::json!({"input":input,"runtime":runtime}),
        );
        let receipt = crate::record_work_item_material_review(root, ITEM, &input, &runtime)
            .expect("authenticated v2-only current/history proofs must support native replacement");
        assert_eq!(receipt.authority_source, "fixture:new-risk");
        assert_eq!(receipt.request_digest, input.request_digest);
        assert_eq!(
            fs::read(old_sidecar).unwrap(),
            old_bytes,
            "old immutable evidence is preserved"
        );
        let summary = crate::read_json(&summary_path).unwrap();
        assert!(
            summary["decisionEvidence"].is_null(),
            "bridge must not invent a legacy approval projection"
        );
        assert_eq!(
            summary["materialReviewReceiptHistory"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            summary["materialReviewReceipt"]["receiptDigest"],
            serde_json::json!(receipt.receipt_digest)
        );
    }

    #[test]
    fn public_initial_material_review_uses_explicit_private_approval_and_exact_replay() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        let (directory, runtime, input, authority) = initial_transaction_fixture();
        let root = directory.path();
        authority.approve(
            root,
            ITEM,
            cockpit_protocol::review_authority::HumanReviewOperation::MaterialReview,
            &serde_json::json!({"input":input,"runtime":runtime}),
        );
        // Call the public protected wrapper explicitly; the alias used by
        // transaction-cut tests below deliberately does not apply here.
        let receipt = crate::record_work_item_material_review(root, ITEM, &input, &runtime)
            .expect("explicit test source approves this exact material decision");
        let summary = root.join(format!(".ai/work-items/active/{ITEM}.summary.json"));
        let summary_before_replay = fs::read(&summary).unwrap();
        let artifacts_before_replay = transaction_artifacts(root);
        assert_eq!(
            crate::record_work_item_material_review(root, ITEM, &input, &runtime).unwrap(),
            receipt,
            "completed replay returns the exact original receipt"
        );
        assert_eq!(fs::read(summary).unwrap(), summary_before_replay);
        assert_eq!(transaction_artifacts(root), artifacts_before_replay);
        assert_eq!(receipt.request_digest, input.request_digest);
    }

    #[test]
    fn initial_material_transaction_seals_before_publishing_and_reuses_exact_pending_bytes() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        let (directory, runtime, input, _authority) = initial_transaction_fixture();
        let root = directory.path();
        let summary_path = root.join(format!(".ai/work-items/active/{ITEM}.summary.json"));
        let before = fs::read(&summary_path).unwrap();
        REPLACEMENT_FAILURE.with(|failure| failure.set(Some("after_prepare")));
        record_work_item_material_review(root, ITEM, &input, &runtime)
            .expect_err("initial prepare is sealed before publishing");
        assert_eq!(fs::read(&summary_path).unwrap(), before);
        let pending = transaction_artifacts(root);
        assert_eq!(pending.len(), 1, "only the sealed prepare exists");
        let preparation = pending.iter().next().unwrap();
        assert!(preparation.0.ends_with(".prepare.json"));
        let value: serde_json::Value = serde_json::from_slice(preparation.1).unwrap();
        assert_eq!(value["prepared"]["schemaVersion"], 2);
        assert_eq!(value["prepared"]["kind"], "initial");
        let receipt = record_work_item_material_review(root, ITEM, &input, &runtime).unwrap();
        let committed = transaction_artifacts(root);
        assert_eq!(
            committed.len(),
            3,
            "prepare, exact request and exact receipt are retained"
        );
        assert_eq!(committed.get(preparation.0), Some(preparation.1));
        assert_eq!(
            receipt,
            serde_json::from_value::<MaterialInspectionReviewDecisionReceipt>(
                value["prepared"]["receipt"].clone()
            )
            .unwrap()
        );
        let document = read_active_contract_document(root, ITEM).unwrap();
        let request = plan_work_item_material_review(root, ITEM).unwrap();
        assert!(matches!(
            read_material_review_receipt(
                root,
                &document.contract,
                &document.digest,
                &request,
                &summary_path
            )
            .unwrap(),
            MaterialReviewReceiptState::Current(_)
        ));
        export_pinned_runtime_material_probe(root, &request, &input, &receipt);
    }

    #[test]
    fn material_transaction_rechecks_staged_raw_bytes_before_summary() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        for suffix in ["request.json", "receipt.json"] {
            let (directory, runtime, input, _old_sidecar, _old_bytes, _authority) =
                transaction_fixture();
            let root = directory.path();
            let summary = root.join(format!(".ai/work-items/active/{ITEM}.summary.json"));
            let before = fs::read(&summary).unwrap();
            let semantic = input
                .request_digest
                .as_str()
                .strip_prefix("sha256:")
                .unwrap();
            let raw = input
                .request_artifact_digest
                .as_ref()
                .unwrap()
                .as_str()
                .strip_prefix("sha256:")
                .unwrap();
            let staged = root.join(format!(
                ".ai/evidence/material-inspection-review/{ITEM}/{semantic}.{raw}.{suffix}"
            ));
            let changed = staged.clone();
            REPLACEMENT_BEFORE_COMMIT.with(|callback| {
                *callback.borrow_mut() =
                    Some(Box::new(move || fs::write(changed, b"{}\n").unwrap()))
            });
            record_work_item_material_review(root, ITEM, &input, &runtime)
                .expect_err("staged raw mutation must reject before Summary");
            assert!(
                fs::read(&summary).unwrap() == before,
                "{suffix}: no pointer may publish altered staged evidence"
            );
            assert_eq!(
                fs::read(staged).unwrap(),
                b"{}\n",
                "concurrent bytes are preserved"
            );
        }
    }

    #[test]
    fn replacement_pending_retry_reuses_immutable_receipt() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        let (directory, runtime, input, old_sidecar, old_bytes, _authority) = transaction_fixture();
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
            ".ai/evidence/material-inspection-review/{ITEM}/{}.receipt.json",
            transaction_stem(&input)
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

    fn transaction_stem(input: &MaterialInspectionReviewDecisionInput) -> String {
        format!(
            "{}.{}",
            input
                .request_digest
                .as_str()
                .strip_prefix("sha256:")
                .unwrap(),
            input
                .request_artifact_digest
                .as_ref()
                .unwrap()
                .as_str()
                .strip_prefix("sha256:")
                .unwrap()
        )
    }

    fn artifacts_at(directory: &Path) -> BTreeMap<String, Vec<u8>> {
        fs::read_dir(directory)
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

    /// Independent expectations for exact input/provenance, target Summary and
    /// all three newly staged files. The original evidence map is checked separately.
    fn expected_transaction_files(
        directory: &Path,
        request: &MaterialReviewRequest,
        input: &MaterialInspectionReviewDecisionInput,
        runtime: &RuntimeContext,
        contract: &Contract,
        summary_before: &[u8],
        proof_digest: &Option<Digest>,
    ) -> BTreeMap<String, Vec<u8>> {
        let stem = transaction_stem(input);
        let prepare_name = format!("{stem}.prepare.json");
        let prepare_bytes = fs::read(directory.join(&prepare_name)).unwrap();
        let envelope: PreparedMaterialReplacement = serde_json::from_slice(&prepare_bytes).unwrap();
        let prepared = &envelope.prepared;
        assert_eq!(prepared.schema_version, 2);
        assert_eq!(
            prepared.kind,
            if proof_digest.is_some() {
                MaterialPreparationKind::AuthoritySourceReplacement
            } else {
                MaterialPreparationKind::Initial
            }
        );
        assert_eq!(prepared.repository_id, contract.repository_id);
        assert_eq!(prepared.work_item_id, contract.work_item_id);
        assert_eq!(prepared.contract_digest, request.contract_digest);
        assert_eq!(prepared.request_digest, request.request_digest);
        assert_eq!(prepared.proof_digest, *proof_digest);
        assert_eq!(prepared.input_digest, json_digest(input).unwrap());
        assert_eq!(prepared.runtime, *runtime);
        assert_eq!(
            prepared.previous_summary_digest,
            Digest::sha256_bytes(summary_before)
        );
        assert_eq!(
            prepared.canonical_request_json,
            request.request_artifact.as_ref().unwrap().canonical_json
        );
        assert_eq!(
            prepared.request_artifact,
            v2_artifact_reference(request).unwrap()
        );
        assert_eq!(
            envelope.prepared_digest,
            replacement_prepared_digest(prepared).unwrap()
        );
        let recorded_by = format!(
            "runtime:{}:{}",
            runtime.runtime_version, runtime.runtime_digest
        );
        let receipt = validate_material_review_decision_with_contract_digest(
            contract,
            &request.contract_digest,
            request,
            input,
            &recorded_by,
            &prepared.receipt.recorded_at,
        )
        .unwrap();
        assert_eq!(prepared.receipt, receipt);
        assert_eq!(
            prepared.receipt_bytes.as_bytes(),
            serde_json::to_vec_pretty(&receipt).unwrap()
        );
        let mut target: serde_json::Value = serde_json::from_slice(summary_before).unwrap();
        let prior = target.get("materialReviewReceipt").cloned();
        assert_eq!(prepared.prior_pointer, prior);
        if let Some(prior) = prior {
            let history = target
                .as_object_mut()
                .unwrap()
                .entry("materialReviewReceiptHistory")
                .or_insert_with(|| serde_json::json!([]))
                .as_array_mut()
                .unwrap();
            assert!(!history.contains(&prior));
            history.push(prior);
        }
        target["materialReviewReceipt"] = serde_json::json!({
            "path":format!(".ai/evidence/{MATERIAL_REVIEW_EVIDENCE_DIRECTORY}/{}/{stem}.receipt.json",contract.work_item_id),
            "digest":Digest::sha256_bytes(prepared.receipt_bytes.as_bytes()),"requestDigest":request.request_digest,"receiptDigest":receipt.receipt_digest,
        });
        assert_eq!(prepared.target_summary, target);
        BTreeMap::from([
            (prepare_name, prepare_bytes),
            (
                format!("{stem}.request.json"),
                request
                    .request_artifact
                    .as_ref()
                    .unwrap()
                    .canonical_json
                    .as_bytes()
                    .to_vec(),
            ),
            (
                format!("{stem}.receipt.json"),
                serde_json::to_vec_pretty(&receipt).unwrap(),
            ),
        ])
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

    fn exercise_transaction_failure_matrix(initial: bool) {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        for stage in [
            "before_prepare",
            "after_prepare",
            "before_request",
            "after_request",
            "before_sidecar",
            "after_sidecar",
            "before_summary",
            "after_summary",
        ] {
            let (directory, runtime, input, old_receipt, _authority) = if initial {
                let (directory, runtime, input, _authority) = initial_transaction_fixture();
                (directory, runtime, input, None, _authority)
            } else {
                let (directory, runtime, input, path, bytes, _authority) = transaction_fixture();
                (directory, runtime, input, Some((path, bytes)), _authority)
            };
            let root = directory.path();
            let summary_path = root.join(format!(".ai/work-items/active/{ITEM}.summary.json"));
            let before = fs::read(&summary_path).unwrap();
            let old_pointer = serde_json::from_slice::<serde_json::Value>(&before)
                .unwrap()
                .get("materialReviewReceipt")
                .cloned();
            let old = if initial {
                BTreeMap::new()
            } else {
                transaction_artifacts(root)
            };
            let request = plan_work_item_material_review(root, ITEM).unwrap();
            let document = read_active_contract_document(root, ITEM).unwrap();
            let proof = finite_authority_source_replacement_proof(
                root,
                &document.contract,
                &document.digest,
                &request,
                &summary_path,
            )
            .unwrap();
            assert_eq!(proof.is_none(), initial);
            let proof_digest = proof.as_ref().map(|value| json_digest(value).unwrap());
            REPLACEMENT_FAILURE.with(|failure| failure.set(Some(stage)));
            let result = record_work_item_material_review(root, ITEM, &input, &runtime);
            let material = root.join(format!(".ai/evidence/material-inspection-review/{ITEM}"));
            let receipt = if stage == "after_summary" {
                result.expect("post-rename uncertainty resolves from exact canonical target")
            } else {
                let error = result.expect_err("actual publication boundary interrupts");
                assert!(
                    error.to_string().contains("injected interruption"),
                    "{stage}: {error}"
                );
                assert!(
                    fs::read(&summary_path).unwrap() == before,
                    "{stage}: Summary unchanged"
                );
                let status =
                    crate::work_item_status_snapshot_with_runtime(root, ITEM, &runtime).unwrap();
                assert!(status.review_receipt_digest.is_none());
                assert!(
                    status
                        .effective_unknowns
                        .contains(&"repository_material_inspection_unavailable".into())
                );
                let mut expected = old.clone();
                if stage != "before_prepare" {
                    let new = expected_transaction_files(
                        &material,
                        &request,
                        &input,
                        &runtime,
                        &document.contract,
                        &before,
                        &proof_digest,
                    );
                    for (name, bytes) in new {
                        let include = name.ends_with(".prepare.json")
                            || (name.ends_with(".request.json")
                                && !["after_prepare", "before_request"].contains(&stage))
                            || (name.ends_with(".receipt.json")
                                && ["after_sidecar", "before_summary"].contains(&stage));
                        if include {
                            assert!(expected.insert(name, bytes).is_none());
                        }
                    }
                }
                assert_eq!(
                    transaction_artifacts(root),
                    expected,
                    "{stage}: exact immutable pending set"
                );
                record_work_item_material_review(root, ITEM, &input, &runtime)
                    .expect("exact pending continuation commits")
            };
            if let Some((path, bytes)) = &old_receipt {
                assert_eq!(&fs::read(path).unwrap(), bytes);
            }
            let committed = fs::read(&summary_path).unwrap();
            let summary: serde_json::Value = serde_json::from_slice(&committed).unwrap();
            if let Some(pointer) = old_pointer {
                assert_eq!(
                    summary["materialReviewReceiptHistory"],
                    serde_json::json!([pointer])
                );
            } else {
                assert!(summary.get("materialReviewReceiptHistory").is_none());
            }
            assert_eq!(
                fs::read(root.join(summary["materialReviewReceipt"]["path"].as_str().unwrap()))
                    .unwrap(),
                serde_json::to_vec_pretty(&receipt).unwrap()
            );
            let mut expected = old.clone();
            for (name, bytes) in expected_transaction_files(
                &material,
                &request,
                &input,
                &runtime,
                &document.contract,
                &before,
                &proof_digest,
            ) {
                assert!(expected.insert(name, bytes).is_none());
            }
            let actual = transaction_artifacts(root);
            assert_eq!(actual, expected);
            record_work_item_material_review(root, ITEM, &input, &runtime)
                .expect_err("committed replay rejects without writes");
            assert_eq!(transaction_artifacts(root), actual);
            assert_eq!(fs::read(summary_path).unwrap(), committed);
        }
    }

    #[test]
    fn replacement_transaction_failure_matrix() {
        exercise_transaction_failure_matrix(false);
    }

    #[test]
    fn initial_material_transaction_failure_matrix() {
        exercise_transaction_failure_matrix(true);
    }

    #[test]
    fn committed_material_artifact_corruption_never_discharges_or_repairs_evidence() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        for case in [
            "missing",
            "append",
            "same_semantics_other_provenance",
            "oversized_valid_request",
            "false_declared_length",
        ] {
            let (directory, runtime, input, _authority) = initial_transaction_fixture();
            let root = directory.path();
            let mut receipt =
                record_work_item_material_review(root, ITEM, &input, &runtime).unwrap();
            let document = read_active_contract_document(root, ITEM).unwrap();
            let request = plan_work_item_material_review(root, ITEM).unwrap();
            let summary_path = root.join(format!(".ai/work-items/active/{ITEM}.summary.json"));
            assert!(matches!(
                read_material_review_receipt(
                    root,
                    &document.contract,
                    &document.digest,
                    &request,
                    &summary_path
                )
                .unwrap(),
                MaterialReviewReceiptState::Current(_)
            ));
            let reference = receipt.request_artifact.as_ref().unwrap();
            let path = root.join(&reference.path);
            let original = fs::read(&path).unwrap();
            match case {
                "missing" => fs::remove_file(&path).unwrap(),
                "append" => {
                    let mut changed = original.clone();
                    changed.push(b' ');
                    fs::write(&path, changed).unwrap();
                }
                "same_semantics_other_provenance" => {
                    let mut fixed = super::super::material_recipe::RequestV2::from_v1(
                        public_request_to_v1(&request),
                    );
                    fixed.analysis_implementation_digest =
                        Digest::sha256_bytes(b"another consumer provenance");
                    let other = super::super::material_recipe::seal_v2(fixed).unwrap();
                    assert_eq!(other.request.request_digest, request.request_digest);
                    assert_ne!(other.raw_digest, reference.raw_digest);
                    super::super::material_recipe::decode_v2(&other.canonical_bytes).unwrap();
                    fs::write(&path, other.canonical_bytes).unwrap();
                }
                "oversized_valid_request" => {
                    let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
                    value["reviewDiagnostic"] = serde_json::json!("");
                    let base = serde_json::to_vec(&value).unwrap().len();
                    value["reviewDiagnostic"] = serde_json::json!(
                        "x".repeat(super::super::material_recipe::ARTIFACT_LIMIT + 1 - base)
                    );
                    let changed = serde_json::to_vec(&value).unwrap();
                    assert_eq!(
                        changed.len(),
                        super::super::material_recipe::ARTIFACT_LIMIT + 1
                    );
                    assert_eq!(
                        serde_json::from_slice::<serde_json::Value>(&changed).unwrap()["schemaVersion"],
                        2
                    );
                    fs::write(&path, changed).unwrap();
                }
                "false_declared_length" => {
                    receipt.request_artifact.as_mut().unwrap().byte_length += 1;
                    receipt.receipt_digest = receipt.canonical_digest().unwrap();
                    receipt.validate_integrity().unwrap();
                    let mut summary: serde_json::Value =
                        serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
                    let receipt_path =
                        root.join(summary["materialReviewReceipt"]["path"].as_str().unwrap());
                    let changed = serde_json::to_vec_pretty(&receipt).unwrap();
                    fs::write(receipt_path, &changed).unwrap();
                    summary["materialReviewReceipt"]["digest"] =
                        serde_json::json!(Digest::sha256_bytes(&changed));
                    summary["materialReviewReceipt"]["receiptDigest"] =
                        serde_json::json!(receipt.receipt_digest);
                    fs::write(&summary_path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
                }
                _ => unreachable!(),
            }
            let before = fs::read(&summary_path).unwrap();
            let files = transaction_artifacts(root);
            let error = read_material_review_receipt(
                root,
                &document.contract,
                &document.digest,
                &request,
                &summary_path,
            )
            .unwrap_err();
            if case == "oversized_valid_request" {
                assert!(error.to_string().contains("bounded read limit"), "{error}");
            }
            let projection = material_review_gate_projection_with_contract_digest(
                root,
                &document.contract,
                &document.digest,
                &summary_path,
            )
            .unwrap();
            assert!(projection.review_receipt_digest.is_none());
            assert!(projection.discharged_unknowns.is_empty());
            assert!(
                projection
                    .effective_unknowns
                    .contains(&"material_review_receipt_invalid".into())
            );
            assert!(
                projection
                    .effective_unknowns
                    .contains(&"repository_material_inspection_unavailable".into())
            );
            assert!(
                fs::read(&summary_path).unwrap() == before,
                "{case}: reader cannot repair Summary"
            );
            assert!(
                transaction_artifacts(root) == files,
                "{case}: reader preserves all observed files"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn oversized_canonical_material_request_publishes_no_evidence_or_summary() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        let (directory, runtime, input, _authority) = initial_transaction_fixture();
        let root = directory.path();
        let initial = plan_work_item_material_review(root, ITEM).unwrap();
        let mut prefix = PathBuf::from("src");
        for index in 0..15 {
            prefix.push(format!("{index:02}{}", "\u{1}".repeat(238)));
        }
        fs::create_dir_all(root.join(&prefix)).unwrap();
        for index in initial.entries.len()..MAX_MATERIAL_CHANGED_FILES {
            fs::write(root.join(&prefix).join(format!("member-{index:03}.rs")),"fn material() -> String { let mut text=String::from(\"token\"); text.push_str(\"ization\"); text }\n").unwrap();
        }
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
                "bounded source with expanding canonical path escapes",
            ],
        );
        let summary = root.join(format!(".ai/work-items/active/{ITEM}.summary.json"));
        let before = fs::read(&summary).unwrap();
        let evidence = root.join(format!(".ai/evidence/material-inspection-review/{ITEM}"));
        assert!(!evidence.exists());
        let error = record_work_item_material_review(root, ITEM, &input, &runtime).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("canonical material request exceeds 4194304 bytes"),
            "{error}"
        );
        assert!(
            !evidence.exists(),
            "oversize refusal precedes evidence directory/preparation publication"
        );
        assert!(fs::read(summary).unwrap() == before);
    }

    #[test]
    fn replacement_pending_rejects_wrong_input_runtime_and_rehashed_target() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        let (directory, runtime, input, _old_sidecar, _old_bytes, _authority) =
            transaction_fixture();
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
        let name = format!("{}.prepare.json", transaction_stem(&input));
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
    fn replacement_pending_summary_conflict_preserves_concurrent_state() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        let (directory, runtime, input, _sidecar, _bytes, _authority) = transaction_fixture();
        let root = directory.path();
        REPLACEMENT_FAILURE.with(|failure| failure.set(Some("after_prepare")));
        record_work_item_material_review(root, ITEM, &input, &runtime)
            .expect_err("actual pending preparation");
        let path = root.join(format!(".ai/work-items/active/{ITEM}.summary.json"));
        let mut summary: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        summary["concurrentPendingNote"] = serde_json::json!(true);
        let concurrent = serde_json::to_vec_pretty(&summary).unwrap();
        fs::write(&path, &concurrent).unwrap();
        let artifacts = transaction_artifacts(root);
        record_work_item_material_review(root, ITEM, &input, &runtime)
            .expect_err("changed Summary cannot consume original pending");
        assert_eq!(fs::read(path).unwrap(), concurrent);
        assert_eq!(transaction_artifacts(root), artifacts);
    }

    #[test]
    fn replacement_orphan_sidecar_without_prepare_is_not_adopted() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        let (directory, runtime, input, _sidecar, _bytes, _authority) = transaction_fixture();
        let root = directory.path();
        REPLACEMENT_FAILURE.with(|failure| failure.set(Some("after_sidecar")));
        record_work_item_material_review(root, ITEM, &input, &runtime)
            .expect_err("actual staged sidecar");
        let name = format!("{}.prepare.json", transaction_stem(&input));
        fs::remove_file(
            root.join(".ai/evidence/material-inspection-review")
                .join(ITEM)
                .join(name),
        )
        .unwrap();
        let summary = root.join(format!(".ai/work-items/active/{ITEM}.summary.json"));
        let before = fs::read(&summary).unwrap();
        let artifacts = transaction_artifacts(root);
        let error = record_work_item_material_review(root, ITEM, &input, &runtime)
            .expect_err("unproven orphan cannot be normalized into a fresh transaction");
        assert!(error.to_string().contains("without sealed prepare"));
        assert_eq!(fs::read(summary).unwrap(), before);
        assert_eq!(transaction_artifacts(root), artifacts);
    }

    #[cfg(unix)]
    #[test]
    fn replacement_pending_symlinks_reject_without_following_or_overwriting() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        for link_prepare in [true, false] {
            let (directory, runtime, input, old_sidecar, old_bytes, _authority) =
                transaction_fixture();
            let root = directory.path();
            REPLACEMENT_FAILURE.with(|failure| failure.set(Some("after_prepare")));
            record_work_item_material_review(root, ITEM, &input, &runtime)
                .expect_err("actual sealed preparation");
            let hex = transaction_stem(&input);
            let parent = root
                .join(".ai/evidence/material-inspection-review")
                .join(ITEM);
            let target = if link_prepare {
                let path = parent.join(format!("{hex}.prepare.json"));
                let retained = root.join(".ai/evidence/retained-prepare.json");
                fs::copy(&path, &retained).unwrap();
                fs::remove_file(&path).unwrap();
                std::os::unix::fs::symlink(retained, &path).unwrap();
                path
            } else {
                let path = parent.join(format!("{hex}.receipt.json"));
                std::os::unix::fs::symlink(&old_sidecar, &path).unwrap();
                path
            };
            let summary = root.join(format!(".ai/work-items/active/{ITEM}.summary.json"));
            let before = fs::read(&summary).unwrap();
            let artifacts = transaction_artifacts(root);
            record_work_item_material_review(root, ITEM, &input, &runtime)
                .expect_err("symlinked pending files cannot be consumed");
            assert!(
                fs::symlink_metadata(target)
                    .unwrap()
                    .file_type()
                    .is_symlink()
            );
            assert_eq!(fs::read(summary).unwrap(), before);
            assert_eq!(fs::read(old_sidecar).unwrap(), old_bytes);
            assert_eq!(transaction_artifacts(root), artifacts);
        }
    }

    #[test]
    fn replacement_rechecks_source_review_and_summary_before_commit() {
        const ITEM: &str = "WI-REPLACEMENT-TRANSACTION";
        for mutation in [
            "summary",
            "source",
            "review",
            "active-directory",
            "evidence-directory",
        ] {
            let (directory, runtime, input, old_sidecar, old_bytes, _authority) =
                transaction_fixture();
            let root = directory.path();
            let summary_path = root.join(format!(".ai/work-items/active/{ITEM}.summary.json"));
            let before = fs::read(&summary_path).unwrap();
            let mut changed: serde_json::Value = serde_json::from_slice(&before).unwrap();
            let old_pointer = changed["materialReviewReceipt"].clone();
            let old_history = changed.get("materialReviewReceiptHistory").cloned();
            let old_artifacts = transaction_artifacts(root);
            let request = plan_work_item_material_review(root, ITEM).unwrap();
            let document = read_active_contract_document(root, ITEM).unwrap();
            let proof = finite_authority_source_replacement_proof(
                root,
                &document.contract,
                &document.digest,
                &request,
                &summary_path,
            )
            .unwrap()
            .unwrap();
            let proof_digest = Some(json_digest(&proof).unwrap());
            let mutation_root = root.to_path_buf();
            let mutation_summary = summary_path.clone();
            #[cfg(not(windows))]
            let copied_old = old_artifacts.clone();
            match mutation {
                "summary" => changed["concurrentFixtureNote"] = serde_json::json!(true),
                "review" => {
                    changed["decisionEvidence"]["recordedBy"] =
                        serde_json::json!("human:unbound-fixture")
                }
                _ => {}
            }
            let expected =
                if ["source", "active-directory", "evidence-directory"].contains(&mutation) {
                    before.clone()
                } else {
                    serde_json::to_vec_pretty(&changed).unwrap()
                };
            let callback_bytes = expected.clone();
            #[cfg(windows)]
            let rename_error = std::rc::Rc::new(std::cell::RefCell::new(None));
            #[cfg(windows)]
            let callback_rename_error = rename_error.clone();
            #[cfg(windows)]
            let original_identity =
                if ["active-directory", "evidence-directory"].contains(&mutation) {
                    let path = if mutation == "active-directory" {
                        root.join(".ai/work-items/active")
                    } else {
                        root.join(format!(".ai/evidence/material-inspection-review/{ITEM}"))
                    };
                    // This temporary observation handle is dropped before the producer/callback.
                    let handle = Dir::open_ambient_dir(&path, ambient_authority()).unwrap();
                    Some(replacement_directory_identity(&handle, &path).unwrap())
                } else {
                    None
                };
            REPLACEMENT_BEFORE_COMMIT.with(|callback| *callback.borrow_mut()=Some(Box::new(move || {
                if mutation=="source"{
                    fs::write(mutation_root.join("README.md"),"source changed before commit\n").unwrap();
                }else if mutation=="active-directory"{
                    let active=mutation_root.join(".ai/work-items/active");
                    let retained=mutation_root.join(".ai/work-items/retained-active-before-swap");
                    let result=fs::rename(&active,&retained);
                    #[cfg(windows)]{
                        let error=result.expect_err("actual Windows rename must be denied by producer's existing directory handle");
                        assert_eq!(error.raw_os_error(),Some(32),"unexpected Windows active rename error");
                        *callback_rename_error.borrow_mut()=error.raw_os_error();
                    }
                    #[cfg(not(windows))]{
                        result.unwrap();fs::create_dir(&active).unwrap();
                        for entry in fs::read_dir(&retained).unwrap(){let entry=entry.unwrap();fs::copy(entry.path(),active.join(entry.file_name())).unwrap();}
                    }
                }else if mutation=="evidence-directory"{
                    let material=mutation_root.join(format!(".ai/evidence/material-inspection-review/{ITEM}"));
                    let retained=mutation_root.join(".ai/evidence/retained-material-before-swap");
                    let result=fs::rename(&material,&retained);
                    #[cfg(windows)]{
                        let error=result.expect_err("actual Windows rename must be denied by producer's existing directory handle");
                        assert_eq!(error.raw_os_error(),Some(32),"unexpected Windows evidence rename error");
                        *callback_rename_error.borrow_mut()=error.raw_os_error();
                    }
                    #[cfg(not(windows))]{
                        result.unwrap();fs::create_dir(&material).unwrap();
                        for (name,bytes) in copied_old{fs::write(material.join(name),bytes).unwrap();}
                    }
                }else{fs::write(mutation_summary,callback_bytes).unwrap();}
            })));
            let result = record_work_item_material_review(root, ITEM, &input, &runtime);
            let current_directory =
                root.join(format!(".ai/evidence/material-inspection-review/{ITEM}"));
            #[cfg(windows)]
            if ["active-directory", "evidence-directory"].contains(&mutation) {
                assert_eq!(*rename_error.borrow(), Some(32));
                let retained = if mutation == "active-directory" {
                    root.join(".ai/work-items/retained-active-before-swap")
                } else {
                    root.join(".ai/evidence/retained-material-before-swap")
                };
                assert!(!retained.exists(), "no directory swap happened");
                let observed_path = if mutation == "active-directory" {
                    root.join(".ai/work-items/active")
                } else {
                    current_directory.clone()
                };
                let observed = {
                    let handle =
                        Dir::open_ambient_dir(&observed_path, ambient_authority()).unwrap();
                    replacement_directory_identity(&handle, &observed_path).unwrap()
                };
                assert_eq!(Some(observed), original_identity);
                let receipt = result.expect(
                    "blocked directory attack leaves the legal exact producer transaction valid",
                );
                let new = expected_transaction_files(
                    &current_directory,
                    &request,
                    &input,
                    &runtime,
                    &document.contract,
                    &before,
                    &proof_digest,
                );
                let mut all = old_artifacts.clone();
                for (name, bytes) in new {
                    assert!(all.insert(name, bytes).is_none());
                }
                assert_eq!(transaction_artifacts(root), all);
                let envelope: PreparedMaterialReplacement = serde_json::from_slice(
                    &all[&format!("{}.prepare.json", transaction_stem(&input))],
                )
                .unwrap();
                assert_eq!(
                    fs::read(&summary_path).unwrap(),
                    serde_json::to_vec_pretty(&envelope.prepared.target_summary).unwrap()
                );
                assert_eq!(receipt, envelope.prepared.receipt);
                let canonical: serde_json::Value =
                    serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
                let mut history = old_history.unwrap_or_else(|| serde_json::json!([]));
                history.as_array_mut().unwrap().push(old_pointer.clone());
                assert_eq!(canonical["materialReviewReceiptHistory"], history);
                assert_eq!(
                    history
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter(|p| **p == old_pointer)
                        .count(),
                    1
                );
                match read_material_review_receipt(
                    root,
                    &document.contract,
                    &document.digest,
                    &request,
                    &summary_path,
                )
                .unwrap()
                {
                    MaterialReviewReceiptState::Current(actual) => assert_eq!(*actual, receipt),
                    _ => panic!("strict reader must consume the newly committed exact artifact"),
                }
                assert_eq!(fs::read(&old_sidecar).unwrap(), old_bytes);
                continue;
            }
            let error = result.expect_err("actual precommit material mutation must reject");
            if mutation == "summary" {
                assert!(error.to_string().contains("compare-and-swap"), "{error}");
            }
            if ["active-directory", "evidence-directory"].contains(&mutation) {
                assert!(
                    error.to_string().contains("directory identity changed"),
                    "{error}"
                );
            }
            assert!(
                fs::read(&summary_path).unwrap() == expected,
                "{mutation}: concurrent Summary bytes preserved"
            );
            let canonical: serde_json::Value =
                serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
            assert_eq!(canonical["materialReviewReceipt"], old_pointer);
            assert_eq!(
                canonical.get("materialReviewReceiptHistory").cloned(),
                old_history
            );
            assert_eq!(fs::read(&old_sidecar).unwrap(), old_bytes);
            if mutation == "source" {
                assert_eq!(
                    fs::read(root.join("README.md")).unwrap(),
                    b"source changed before commit\n"
                );
            }
            let staged_directory = if mutation == "evidence-directory" {
                root.join(".ai/evidence/retained-material-before-swap")
            } else {
                current_directory.clone()
            };
            let new = expected_transaction_files(
                &staged_directory,
                &request,
                &input,
                &runtime,
                &document.contract,
                &before,
                &proof_digest,
            );
            let mut all = old_artifacts.clone();
            for (name, bytes) in new {
                assert!(all.insert(name, bytes).is_none());
            }
            assert_eq!(artifacts_at(&staged_directory), all);
            if mutation == "evidence-directory" {
                assert_eq!(transaction_artifacts(root), old_artifacts);
            }
            if mutation == "active-directory" {
                assert_eq!(
                    fs::read(root.join(format!(
                        ".ai/work-items/retained-active-before-swap/{ITEM}.summary.json"
                    )))
                    .unwrap(),
                    before
                );
            }
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
            request_artifact: None,
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
