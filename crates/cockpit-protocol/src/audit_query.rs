use cockpit_core::Digest;
use serde::{Deserialize, Serialize};

use crate::{UsageAssurance, UsageCoverage, UsageSourceKind, UsageSubtotal, UsageTokenCounts};

pub const AUDIT_QUERY_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditQueryFilters {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_item_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reported_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_timezone: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditEvidenceRef {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<Digest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditQueryItem {
    pub event_id: String,
    pub event_type: String,
    pub work_item_id: Option<String>,
    pub occurred_at: Option<String>,
    pub recorded_at: Option<String>,
    pub source_observed_at: Option<String>,
    pub received_at: Option<String>,
    pub reported_model: Option<String>,
    pub configured_model: Option<String>,
    pub actor: Option<String>,
    pub actor_provenance: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_runtime_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_runtime_digest: Option<Digest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_kind: Option<UsageSourceKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_assurance: Option<UsageAssurance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_assurance: Option<UsageAssurance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_elapsed_ms: Option<u64>,
    pub token_counts: UsageTokenCounts,
    pub evidence_refs: Vec<AuditEvidenceRef>,
    pub display_timezone: String,
    pub display_time: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditQueryCoverage {
    pub state: UsageCoverage,
    pub known_count: u64,
    pub unknown_count: Option<u64>,
    pub unknown_sources: Vec<String>,
    pub unknown_reasons: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditQueryPage {
    pub schema_version: u32,
    pub repository_id: String,
    pub runtime_version: String,
    pub runtime_digest: Digest,
    pub as_of: String,
    pub source_snapshot_digest: Digest,
    pub filters: AuditQueryFilters,
    pub items: Vec<AuditQueryItem>,
    pub coverage: AuditQueryCoverage,
    pub page_usage_totals: UsageTokenCounts,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub page_usage_subtotals: Vec<UsageSubtotal>,
    pub returned_count: usize,
    pub truncated: bool,
    pub next_cursor: Option<String>,
}
