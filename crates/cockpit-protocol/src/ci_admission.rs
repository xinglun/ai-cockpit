//! Untrusted evidence data and read-only inspection results.
//! Parsing a packet never authenticates its author or grants execution authority.

use cockpit_core::Digest;
use serde::{Deserialize, Serialize};

/// Untrusted archive inventory. Parsing this wire type proves no provider,
/// workflow, Runtime, uploader, consumer, or execution authority.
pub const EVIDENCE_ARCHIVE_MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UntrustedEvidenceArchiveManifest {
    pub schema_version: u32,
    pub entries: Vec<UntrustedEvidenceArchiveEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UntrustedEvidenceArchiveEntry {
    pub path: String,
    pub role: UntrustedEvidenceArchiveRole,
    pub bytes: u64,
    pub digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UntrustedEvidenceArchiveRole {
    NativeVerificationEvidence,
}

/// Strict *raw* schema2 native envelope. T is the existing typed native
/// VerificationReceipt at the repository boundary, never an authenticated
/// provider capability. Semantic validation remains mandatory after decoding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeExecutionEnvelope<T> {
    pub protocol_version: u32,
    pub evidence_schema_version: u32,
    pub work_item_id: String,
    pub repository_id: String,
    pub runtime_version: String,
    pub runtime_digest: Digest,
    pub contract_digest: Digest,
    pub repository_snapshot_digest: Digest,
    pub passed: bool,
    pub receipt_digest: Digest,
    pub capture_mode: UntrustedNativeCaptureMode,
    pub created_at: String,
    pub receipt: T,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UntrustedNativeCaptureMode {
    FullCapture,
}

// Portable execution evidence is data. Deserialization never authenticates an author.
pub const PORTABLE_EVIDENCE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortableExecutionEvidence<T> {
    pub schema_version: u32,
    pub native_input: serde_json::Value,
    pub native_evidence: T,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortableSourceVerification {
    Unverified,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortableEvidencePlan {
    pub schema_version: u32,
    pub input_digest: Digest,
    pub plan_digest: Digest,
    pub work_item_id: Option<String>,
    pub consumer_contract_digest: Option<Digest>,
    pub consumer_snapshot_digest: Option<Digest>,
    pub source_verification: PortableSourceVerification,
    pub may_replace_execution: bool,
    pub integrity_valid: bool,
    /// Explicitly unverified source/result claims, never raw command output or authority.
    pub claims: serde_json::Value,
    pub gaps: Vec<String>,
    pub verification_commands_executed: usize,
    pub writes_performed: usize,
}
