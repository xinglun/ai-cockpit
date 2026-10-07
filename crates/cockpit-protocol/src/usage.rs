use cockpit_core::Digest;
use serde::{Deserialize, Serialize};

pub const USAGE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UsageSourceKind {
    ProviderReported,
    HostReported,
    AgentDeclared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageUnit {
    Invocation,
    Turn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageAssurance {
    Unknown,
    CallerClaim,
    VerifiedAdapter,
}

impl Default for UsageAssurance {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageCoverage {
    Unknown,
    Partial,
    Complete,
}

/// A caller claim. A provider/host label is not a verified measurement.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageRecordRequest {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub source_event_id: String,
    pub source_kind: UsageSourceKind,
    pub actor: Option<String>,
    pub configured_model: Option<String>,
    pub reported_model: Option<String>,
    pub role: String,
    pub phase: String,
    pub unit: UsageUnit,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cached_input_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
    pub source_observed_at: Option<String>,
    pub evidence_ref: String,
    pub evidence_digest: Digest,
}

impl UsageRecordRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != USAGE_SCHEMA_VERSION {
            return Err("unsupported usage schema version");
        }
        if !valid_sha256(&self.repository_id) || !valid_sha256(&self.evidence_digest.to_string()) {
            return Err("usage digest or repository identity is invalid");
        }
        if !valid_identifier(&self.work_item_id)
            || !valid_identifier(&self.source_event_id)
            || !valid_identifier(&self.role)
            || !valid_identifier(&self.phase)
        {
            return Err("usage identity, role, or phase is invalid");
        }
        if self
            .actor
            .as_deref()
            .is_some_and(|value| !valid_text(value))
            || self
                .configured_model
                .as_deref()
                .is_some_and(|value| !valid_text(value))
            || self
                .reported_model
                .as_deref()
                .is_some_and(|value| !valid_text(value))
        {
            return Err("usage actor or model is invalid");
        }
        if self.cached_input_tokens.is_some()
            && self
                .cached_input_tokens
                .zip(self.input_tokens)
                .is_none_or(|(subset, total)| subset > total)
            || self.reasoning_tokens.is_some()
                && self
                    .reasoning_tokens
                    .zip(self.output_tokens)
                    .is_none_or(|(subset, total)| subset > total)
        {
            return Err("usage subset exceeds or lacks its parent total");
        }
        if self.evidence_ref.is_empty()
            || self.evidence_ref.contains('\\')
            || self
                .evidence_ref
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == "..")
        {
            return Err("usage evidence reference must be a safe relative path");
        }
        Ok(())
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn valid_text(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageReceipt {
    pub schema_version: u32,
    pub receipt_id: Digest,
    pub request: UsageRecordRequest,
    pub received_at: String,
    pub source_observed_at: Option<String>,
    pub model_assurance: UsageAssurance,
    pub token_assurance: UsageAssurance,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageReceiptRef {
    pub path: String,
    pub digest: Digest,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageTokenCounts {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cached_input_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageSubtotal {
    pub reported_model: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub configured_models: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_kinds: Vec<UsageSourceKind>,
    #[serde(default)]
    pub model_assurance: UsageAssurance,
    #[serde(default)]
    pub token_assurance: UsageAssurance,
    pub role: String,
    pub phase: String,
    pub record_count: u64,
    pub counts: UsageTokenCounts,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageSummary {
    pub schema_version: u32,
    pub work_item_id: String,
    pub cutoff: String,
    pub coverage: UsageCoverage,
    pub unknown_reasons: Vec<String>,
    pub receipt_refs: Vec<UsageReceiptRef>,
    pub totals: UsageTokenCounts,
    pub subtotals: Vec<UsageSubtotal>,
}

impl UsageSummary {
    pub fn unknown(work_item_id: &str, cutoff: String, reason: &str) -> Self {
        Self {
            schema_version: USAGE_SCHEMA_VERSION,
            work_item_id: work_item_id.into(),
            cutoff,
            coverage: UsageCoverage::Unknown,
            unknown_reasons: vec![reason.into()],
            receipt_refs: Vec::new(),
            totals: UsageTokenCounts::default(),
            subtotals: Vec::new(),
        }
    }
}

/// The MCP request schema projects the same strict Rust request parsed by CLI.
/// Optional nullable measurements are never filled with zero by the adapter.
pub fn usage_record_request_schema() -> serde_json::Value {
    let identifier = serde_json::json!({"type":"string","minLength":1,"maxLength":160});
    let optional_text = serde_json::json!({"type":["string","null"],"maxLength":256});
    let nullable_count = serde_json::json!({"type":["integer","null"],"minimum":0});
    serde_json::json!({
        "type": "object",
        "description": crate::WORK_ITEM_USAGE_RECORD_REQUEST_DESCRIPTION,
        "additionalProperties": false,
        "properties": {
            "schemaVersion": {"type":"integer","const":USAGE_SCHEMA_VERSION},
            "repositoryId": {"type":"string","pattern":"^sha256:[0-9a-f]{64}$"},
            "workItemId": identifier,
            "sourceEventId": identifier,
            "sourceKind": {"type":"string","enum":["provider-reported","host-reported","agent-declared"],"description":"Caller label; not authentication or independently verified provenance."},
            "actor": optional_text,
            "configuredModel": optional_text,
            "reportedModel": optional_text,
            "role": identifier,
            "phase": identifier,
            "unit": {"type":"string","enum":["invocation","turn"]},
            "inputTokens": nullable_count,
            "outputTokens": nullable_count,
            "cachedInputTokens": nullable_count,
            "reasoningTokens": nullable_count,
            "sourceObservedAt": {"type":["string","null"],"description":"Accepted only through an independently trusted adapter; caller requests must leave it null."},
            "evidenceRef": {"type":"string","minLength":1},
            "evidenceDigest": {"type":"string","pattern":"^sha256:[0-9a-f]{64}$"}
        },
        "required": ["schemaVersion","repositoryId","workItemId","sourceEventId","sourceKind","role","phase","unit","evidenceRef","evidenceDigest"]
    })
}
