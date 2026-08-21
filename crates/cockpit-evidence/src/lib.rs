use cockpit_core::Digest;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

pub const REUSABLE_RECEIPT_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiffIdentity {
    pub base_commit: String,
    pub head_commit: String,
    pub changed_paths_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceContext {
    pub content_digest: String,
    pub diff: DiffIdentity,
    pub environment_digest: String,
    pub command_digest: String,
    pub scope_digest: String,
    pub governance_digest: String,
    pub toolchain_digest: String,
    pub policy_digest: String,
    pub profile_digest: String,
    pub stage: String,
    pub runner: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReusableReceipt {
    pub schema_version: u32,
    pub receipt_id: String,
    pub node_id: String,
    pub passed: bool,
    pub context: EvidenceContext,
    pub output_digest: String,
    pub created_at_epoch_seconds: i64,
    pub expires_at_epoch_seconds: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReceiptBody<'a> {
    schema_version: u32,
    node_id: &'a str,
    passed: bool,
    context: &'a EvidenceContext,
    output_digest: &'a str,
    created_at_epoch_seconds: i64,
    expires_at_epoch_seconds: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReuseState {
    Fresh,
    Stale,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReuseAction {
    Reuse,
    Execute,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReuseReason {
    FreshExactBinding,
    ProtectedNode,
    EvidenceMissing,
    ReceiptInvalid,
    ReceiptFailed,
    ReceiptFromFuture,
    EvidenceExpired,
    BindingMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReuseDecision {
    pub state: ReuseState,
    pub action: ReuseAction,
    pub reason: ReuseReason,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReceiptError {
    #[error("unsupported reusable receipt schema version")]
    UnsupportedSchema,
    #[error("{0} must not be empty")]
    EmptyIdentity(&'static str),
    #[error("{0} must be a valid SHA-256 digest")]
    InvalidDigest(&'static str),
    #[error("{0} must be a full lowercase hexadecimal Git object id")]
    InvalidObjectId(&'static str),
    #[error("receipt expiry must be later than creation")]
    InvalidTimeWindow,
    #[error("could not serialize reusable receipt identity")]
    Serialization,
    #[error("reusable receipt identity does not match its body")]
    IdentityMismatch,
}

impl EvidenceContext {
    pub fn validate(&self) -> Result<(), ReceiptError> {
        validate_digest("content digest", &self.content_digest)?;
        validate_object_id("base commit", &self.diff.base_commit)?;
        validate_object_id("head commit", &self.diff.head_commit)?;
        validate_digest("changed paths digest", &self.diff.changed_paths_digest)?;
        validate_digest("environment digest", &self.environment_digest)?;
        validate_digest("command digest", &self.command_digest)?;
        validate_digest("scope digest", &self.scope_digest)?;
        validate_digest("governance digest", &self.governance_digest)?;
        validate_digest("toolchain digest", &self.toolchain_digest)?;
        validate_digest("policy digest", &self.policy_digest)?;
        validate_digest("profile digest", &self.profile_digest)?;
        validate_nonempty("stage", &self.stage)?;
        validate_nonempty("runner", &self.runner)
    }
}

impl ReusableReceipt {
    pub fn new(
        node_id: &str,
        passed: bool,
        context: EvidenceContext,
        output_digest: &str,
        created_at_epoch_seconds: i64,
        expires_at_epoch_seconds: i64,
    ) -> Result<Self, ReceiptError> {
        let mut receipt = Self {
            schema_version: REUSABLE_RECEIPT_SCHEMA_VERSION,
            receipt_id: String::new(),
            node_id: node_id.into(),
            passed,
            context,
            output_digest: output_digest.into(),
            created_at_epoch_seconds,
            expires_at_epoch_seconds,
        };
        receipt.validate_body()?;
        receipt.receipt_id = receipt.recompute_id()?;
        Ok(receipt)
    }

    pub fn recompute_id(&self) -> Result<String, ReceiptError> {
        let body = ReceiptBody {
            schema_version: self.schema_version,
            node_id: &self.node_id,
            passed: self.passed,
            context: &self.context,
            output_digest: &self.output_digest,
            created_at_epoch_seconds: self.created_at_epoch_seconds,
            expires_at_epoch_seconds: self.expires_at_epoch_seconds,
        };
        let bytes = serde_json::to_vec(&body).map_err(|_| ReceiptError::Serialization)?;
        Ok(Digest::sha256_bytes(&bytes).to_string())
    }

    pub fn validate(&self) -> Result<(), ReceiptError> {
        self.validate_body()?;
        validate_digest("receipt id", &self.receipt_id)?;
        if self.recompute_id()? != self.receipt_id {
            return Err(ReceiptError::IdentityMismatch);
        }
        Ok(())
    }

    fn validate_body(&self) -> Result<(), ReceiptError> {
        if self.schema_version != REUSABLE_RECEIPT_SCHEMA_VERSION {
            return Err(ReceiptError::UnsupportedSchema);
        }
        validate_nonempty("node id", &self.node_id)?;
        self.context.validate()?;
        validate_digest("output digest", &self.output_digest)?;
        if self.expires_at_epoch_seconds <= self.created_at_epoch_seconds {
            return Err(ReceiptError::InvalidTimeWindow);
        }
        Ok(())
    }
}

pub fn decide_reuse(
    receipt: Option<&ReusableReceipt>,
    current: &EvidenceContext,
    node_id: &str,
    now_epoch_seconds: i64,
    protected: bool,
) -> ReuseDecision {
    if protected {
        return decision(
            ReuseState::Unknown,
            ReuseAction::Execute,
            ReuseReason::ProtectedNode,
        );
    }
    if current.validate().is_err() || node_id.is_empty() {
        return decision(
            ReuseState::Unknown,
            ReuseAction::Execute,
            ReuseReason::ReceiptInvalid,
        );
    }
    let Some(receipt) = receipt else {
        return decision(
            ReuseState::Unknown,
            ReuseAction::Execute,
            ReuseReason::EvidenceMissing,
        );
    };
    if receipt.validate().is_err() {
        return decision(
            ReuseState::Unknown,
            ReuseAction::Execute,
            ReuseReason::ReceiptInvalid,
        );
    }
    if !receipt.passed {
        return decision(
            ReuseState::Unknown,
            ReuseAction::Execute,
            ReuseReason::ReceiptFailed,
        );
    }
    if receipt.created_at_epoch_seconds > now_epoch_seconds {
        return decision(
            ReuseState::Unknown,
            ReuseAction::Execute,
            ReuseReason::ReceiptFromFuture,
        );
    }
    if receipt.expires_at_epoch_seconds <= now_epoch_seconds {
        return decision(
            ReuseState::Stale,
            ReuseAction::Execute,
            ReuseReason::EvidenceExpired,
        );
    }
    if receipt.node_id != node_id {
        return decision(
            ReuseState::Unknown,
            ReuseAction::Execute,
            ReuseReason::ReceiptInvalid,
        );
    }
    if receipt.context != *current {
        return decision(
            ReuseState::Stale,
            ReuseAction::Execute,
            ReuseReason::BindingMismatch,
        );
    }
    decision(
        ReuseState::Fresh,
        ReuseAction::Reuse,
        ReuseReason::FreshExactBinding,
    )
}

fn validate_digest(field: &'static str, value: &str) -> Result<(), ReceiptError> {
    Digest::from_str(value)
        .map(|_| ())
        .map_err(|_| ReceiptError::InvalidDigest(field))
}

fn validate_nonempty(field: &'static str, value: &str) -> Result<(), ReceiptError> {
    if value.trim().is_empty() {
        Err(ReceiptError::EmptyIdentity(field))
    } else {
        Ok(())
    }
}

fn validate_object_id(field: &'static str, value: &str) -> Result<(), ReceiptError> {
    if matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(ReceiptError::InvalidObjectId(field))
    }
}

fn decision(state: ReuseState, action: ReuseAction, reason: ReuseReason) -> ReuseDecision {
    ReuseDecision {
        state,
        action,
        reason,
    }
}
