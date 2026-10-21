//! Wire data is a claim, never an authenticated human principal.
//! Only the separately protected host authority can supply that principal.
use cockpit_core::Digest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanReviewKind {
    HumanReview,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanReviewOperation {
    PreflightReview,
    Close,
    Recovery,
    RestrictedWrite,
    Retire,
    ProfileConfirmation,
    CrossCheckoutRecovery,
    ActivationRecovery,
    SelectedLineageRecovery,
    ArchivedVerificationRecovery,
    LegacySourceRegistration,
    LegacyFinalization,
    HistoricalFinalizationRecovery,
    MaterialReview,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumanReviewBinding {
    pub repository_id: String,
    pub work_item_id: String,
    pub checkout_path: String,
    pub contract_digest: Digest,
    pub preflight_decision_digest: Option<Digest>,
    pub repository_snapshot_digest: Digest,
    pub operation: HumanReviewOperation,
    pub scope: Vec<String>,
    /// Exact native arguments/plan, never just the method name.
    pub operation_digest: Digest,
    pub operation_payload: serde_json::Value,
    /// Fresh native plan/state; excluded from the stable execution replay key.
    pub operation_context: serde_json::Value,
    /// Current denial-only hold head. Resolving it requires the issuer's
    /// independently authenticated resolution, not deleting local records.
    pub local_hold_digest: Option<Digest>,
    pub activation_recovery_digest: Option<Digest>,
}

/// The issuer mints request ID, nonce, epoch and time window outside the agent
/// domain. Their presence here does not prove that issuance occurred.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumanReviewRequest {
    pub schema_version: u32,
    pub kind: HumanReviewKind,
    pub issuer: String,
    pub request_id: String,
    pub nonce: String,
    pub authority_epoch: u64,
    pub issued_at_millis: i64,
    pub expires_at_millis: i64,
    pub binding: HumanReviewBinding,
}

impl HumanReviewRequest {
    pub fn digest(&self) -> Result<Digest, serde_json::Error> {
        crate::digest_json(&serde_json::json!({
            "domain": "ai-cockpit/human-review-request/v2",
            "request": self,
        }))
    }

    pub fn validate_at(&self, now_millis: i64) -> Result<(), &'static str> {
        if self.schema_version != 2 {
            return Err("review_request_schema_unsupported");
        }
        if self.binding.operation_digest
            != review_operation_digest(&self.binding.operation, &self.binding.operation_payload)
                .map_err(|_| "review_operation_digest_invalid")?
        {
            return Err("review_operation_digest_mismatch");
        }
        if [
            &self.issuer,
            &self.request_id,
            &self.nonce,
            &self.binding.repository_id,
            &self.binding.work_item_id,
            &self.binding.checkout_path,
        ]
        .iter()
        .any(|s| s.trim().is_empty())
            || self.binding.scope.is_empty()
            || self.binding.scope.iter().any(|s| s.trim().is_empty())
            || self.authority_epoch == 0
            || self.issued_at_millis < 0
            || self.expires_at_millis <= self.issued_at_millis
        {
            return Err("review_request_invalid");
        }
        if self.binding.operation == HumanReviewOperation::PreflightReview
            && self.binding.preflight_decision_digest.is_none()
        {
            return Err("review_preflight_provenance_missing");
        }
        if now_millis < self.issued_at_millis {
            return Err("review_request_not_yet_valid");
        }
        if now_millis >= self.expires_at_millis {
            return Err("review_request_expired");
        }
        Ok(())
    }
}

pub fn review_operation_digest(
    operation: &HumanReviewOperation,
    payload: &serde_json::Value,
) -> Result<Digest, serde_json::Error> {
    crate::digest_json(
        &serde_json::json!({"domain":"ai-cockpit/human-operation/v1","operation":operation,"payload":payload}),
    )
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanApprovalSubmissionKind {
    HumanApprovalSubmission,
}

/// A transport may submit a reference. It cannot supply the authenticated
/// actor, decision time, source bytes, assurance or authority configuration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumanApprovalSubmission {
    pub schema_version: u32,
    pub kind: HumanApprovalSubmissionKind,
    pub request_digest: Digest,
    pub issuer: String,
    pub approval_id: String,
    pub source_event_id: String,
    pub proof_handle: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnicalReviewKind {
    TechnicalReview,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnicalReviewChoice {
    Accepted,
    ChangesRequired,
}

/// This type deliberately has no conversion into authenticated human approval.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TechnicalReviewDecision {
    pub schema_version: u32,
    pub kind: TechnicalReviewKind,
    pub reviewer_actor: String,
    pub request_digest: Digest,
    pub decision: TechnicalReviewChoice,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurrentReviewAuthorityState {
    AuthorityUnavailable,
    Missing,
    UnknownProvenance,
    Invalid,
    Revoked,
    LocalHold,
    Consumed,
    PublicationIncomplete,
    Active,
    Denied,
}

/// Public mirror bytes are evidence, not a constructible verified capability.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumanApprovalReceipt {
    pub schema_version: u32,
    pub request: HumanReviewRequest,
    pub request_digest: Digest,
    pub approval_id: String,
    pub source_event_id: String,
    pub authenticated_subject: String,
    pub authority_epoch: u64,
    pub resolved_local_hold: Option<Digest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumanRevocationReceipt {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub approval_id: String,
    pub revocation_id: String,
    pub source_event_id: String,
    pub authenticated_subject: String,
    pub authority_epoch: u64,
}

/// A denial-only correction is an attributable recording claim, not a human
/// revocation or a replacement approval. It may never grant authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewCorrectionRequest {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub target_receipt_raw_digest: Digest,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewCorrectionEvent {
    pub schema_version: u32,
    pub sequence: u64,
    pub parent_digest: Option<Digest>,
    pub request_digest: Digest,
    pub request: ReviewCorrectionRequest,
    pub recorded_at: String,
    pub recorded_by: String,
    pub assurance: String,
    pub human_authority_granted: bool,
}

/// Revoking an authenticated grant requires the protected authority. A raw
/// correction claim is intentionally not convertible into this operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewRevocationRequest {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub approval_id: String,
    pub proof_handle: String,
}

/// Exact operation selection only; cannot select an authority implementation.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumanOperationSelection {
    pub operation: HumanReviewOperation,
    pub payload: serde_json::Value,
}
