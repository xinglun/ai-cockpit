//! Read-only GitHub verification planning. No plan is an admission receipt.

use cockpit_core::Digest;
use serde::{Deserialize, Serialize};

pub const GITHUB_PLAN_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GithubEvidenceSelector {
    pub run_id: u64,
    pub attempt: u32,
    pub job_id: u64,
    pub artifact_id: u64,
}

impl GithubEvidenceSelector {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.run_id == 0 || self.attempt == 0 || self.job_id == 0 || self.artifact_id == 0 {
            Err("GitHub run, attempt, job and artifact identities must be positive")
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GithubPlanState {
    Gaps,
    ProviderUnavailable,
    MetadataRejected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GithubPlanAdmission {
    NotAttempted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GithubReadObservation {
    /// Fixed-origin request path, never a caller-supplied or redirect URL.
    pub path: String,
    pub status: u16,
    pub body_digest: Digest,
    pub body_bytes: u64,
    pub link_digest: Option<Digest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GithubCollectionReport {
    pub origin: String,
    pub assurance: String,
    pub elapsed_ms: u64,
    pub maximum_seconds: u64,
    pub requests: Vec<GithubReadObservation>,
    pub complete: bool,
    /// Stable failure class; raw response/error bodies and credentials are not echoed.
    pub failure: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GithubAdmissionPlan {
    pub schema_version: u32,
    pub work_item_id: String,
    pub repository_id: String,
    pub contract_digest: Digest,
    pub consumer_snapshot_digest: Digest,
    pub consumer_runtime_digest: Digest,
    pub selector: GithubEvidenceSelector,
    pub state: GithubPlanState,
    pub collection: GithubCollectionReport,
    /// Non-authoritative bounded metadata inspection, never producer node facts.
    pub metadata: Option<serde_json::Value>,
    pub gaps: Vec<String>,
    pub admission: GithubPlanAdmission,
    pub verification_commands_executed: usize,
    pub writes_performed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_requires_each_positive_identity_and_rejects_authority_fields() {
        let value = serde_json::json!({"runId":1,"attempt":1,"jobId":2,"artifactId":3});
        let selector: GithubEvidenceSelector = serde_json::from_value(value.clone()).unwrap();
        assert!(selector.validate().is_ok());
        for field in ["runId", "attempt", "jobId", "artifactId"] {
            let mut changed = value.clone();
            changed[field] = 0.into();
            let changed: GithubEvidenceSelector = serde_json::from_value(changed).unwrap();
            assert!(changed.validate().is_err());
        }
        for field in ["providerAuthenticated", "trusted", "endpoint", "command"] {
            let mut changed = value.clone();
            changed[field] = true.into();
            assert!(serde_json::from_value::<GithubEvidenceSelector>(changed).is_err());
        }
    }
}

/// Untrusted archive inventory. Parsing this wire type proves no provider,
/// workflow, Runtime, uploader, consumer, or execution authority.
pub const GITHUB_ARCHIVE_MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UntrustedGithubArchiveManifest {
    pub schema_version: u32,
    pub entries: Vec<UntrustedGithubArchiveEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UntrustedGithubArchiveEntry {
    pub path: String,
    pub role: UntrustedGithubArchiveRole,
    pub bytes: u64,
    pub digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UntrustedGithubArchiveRole {
    NativeVerificationEvidence,
}

/// Strict *raw* schema2 native envelope. T is the existing typed native
/// VerificationReceipt at the repository boundary, never an authenticated
/// provider capability. Semantic validation remains mandatory after decoding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UntrustedGithubNativeEnvelope<T> {
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
    pub capture_mode: UntrustedGithubCaptureMode,
    pub created_at: String,
    pub receipt: T,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UntrustedGithubCaptureMode {
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

/// Generic wire alias. The historic Github name remains source-compatible;
/// the envelope itself contains only native repository/execution identities.
pub type NativeExecutionEnvelope<T> = UntrustedGithubNativeEnvelope<T>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortableSourceVerification {
    Unverified,
    NativeInvocation,
    ControlledToolObservation,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortableEvidenceAcceptance {
    pub schema_version: u32,
    pub input_digest: Digest,
    pub plan_digest: Digest,
    pub accepted: bool,
    pub source_verification: PortableSourceVerification,
    pub admission_ref: Option<String>,
    pub gaps: Vec<String>,
    pub verification_commands_executed: usize,
    pub writes_performed: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortableNativePublication {
    pub schema_version: u32,
    pub input_digest: Digest,
    pub record_ref: String,
    pub source_verification: PortableSourceVerification,
    /// A persisted publication cannot recreate its process-local witness on import.
    pub may_replace_execution_after_import: bool,
}
