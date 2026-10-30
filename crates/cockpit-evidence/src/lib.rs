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

/// Exact fact comparison for cross-boundary CI evidence. This API does not
/// authenticate GitHub, authorize a caller, execute artifact-supplied commands,
/// or mint a reusable receipt. Only the provider admission service may do that
/// after independent provenance and current authorization checks.
pub const CI_NODE_FACTS_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CiCommandFacts {
    pub logical_program: String,
    pub logical_args: Vec<String>,
    pub execution_program: String,
    pub execution_args: Vec<String>,
    /// Exact platform-native encoded bytes; no path rewriting or root aliasing.
    pub cwd_bytes: Vec<u8>,
    /// Hash of the exact explicit environment vector, including order.
    pub explicit_environment_digest: Digest,
    /// Hash of every effective child environment entry, not the coverage
    /// helper's environment projection that deliberately omits transient keys.
    pub effective_environment_digest: Digest,
    pub clear_environment: bool,
    pub resource_weight: u64,
    pub timeout_seconds: u64,
    /// Only empty is supported by this first slice. Node IDs alone do not
    /// bind upstream receipts, outputs or generated inputs.
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CiSourceFacts {
    pub repository_id: Digest,
    pub snapshot_digest: Digest,
    /// PR source/head commit, never implicitly substituted for tested_commit.
    pub source_commit: String,
    /// Actual checked-out and tested commit (normally the PR merge commit).
    pub tested_commit: String,
    pub base_commit: String,
    pub source_tree: String,
    pub tested_tree: String,
    pub cargo_lock_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CiCoverageFacts {
    pub manifest_digest: Digest,
    pub metadata_digest: Digest,
    pub target_inventory_digest: Digest,
    pub selected_targets: Vec<String>,
    pub features: Vec<String>,
    pub complete: bool,
    pub warning_capture_complete: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CiNodeFacts {
    pub schema_version: u32,
    pub node_id: String,
    pub command: CiCommandFacts,
    pub source: CiSourceFacts,
    /// Runtime-defined identity of OS, architecture, runner, host and target.
    pub platform_digest: Digest,
    pub toolchain_digest: Digest,
    /// Identity of the Runtime that authored the execution receipt, not the
    /// local Runtime that later imported DelegatedEvidence bytes.
    pub producer_runtime_digest: Digest,
    pub scope_digest: Digest,
    pub policy_digest: Digest,
    pub governance_digest: Digest,
    pub profile_digest: Digest,
    pub stage: String,
    pub coverage: CiCoverageFacts,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CiNodeMismatch {
    InvalidCurrent,
    InvalidProducer,
    NodeId,
    LogicalProgram,
    LogicalArgs,
    ExecutionProgram,
    ExecutionArgs,
    Cwd,
    ExplicitEnvironment,
    EffectiveEnvironment,
    ClearEnvironment,
    ResourceWeight,
    Timeout,
    UnsupportedDependencies,
    Repository,
    Snapshot,
    SourceCommit,
    TestedCommit,
    BaseCommit,
    SourceTree,
    TestedTree,
    CargoLock,
    Platform,
    Toolchain,
    ProducerRuntime,
    Scope,
    Policy,
    Governance,
    Profile,
    Stage,
    CoverageManifest,
    Metadata,
    TargetInventory,
    TargetSelection,
    Features,
    CoverageIncomplete,
    WarningCaptureIncomplete,
}

impl CiNodeFacts {
    fn valid(&self) -> bool {
        fn nonempty_no_nul(value: &str) -> bool {
            !value.trim().is_empty() && !value.contains('\0')
        }
        fn distinct_nonempty(values: &[String]) -> bool {
            let mut seen = std::collections::BTreeSet::new();
            values.iter().all(|v| nonempty_no_nul(v) && seen.insert(v))
        }
        self.schema_version == CI_NODE_FACTS_SCHEMA_VERSION
            && nonempty_no_nul(&self.node_id)
            && nonempty_no_nul(&self.command.logical_program)
            && nonempty_no_nul(&self.command.execution_program)
            && self.command.logical_args.iter().all(|v| !v.contains('\0'))
            && self
                .command
                .execution_args
                .iter()
                .all(|v| !v.contains('\0'))
            && !self.command.cwd_bytes.is_empty()
            && !self.command.cwd_bytes.contains(&0)
            && self.command.resource_weight > 0
            && (1..=900).contains(&self.command.timeout_seconds)
            && matches!(
                self.stage.as_str(),
                "task" | "pre_ci" | "pr" | "merge" | "release"
            )
            && !self.coverage.selected_targets.is_empty()
            && distinct_nonempty(&self.command.dependencies)
            && distinct_nonempty(&self.coverage.selected_targets)
            && distinct_nonempty(&self.coverage.features)
            && [
                &self.source.source_commit,
                &self.source.tested_commit,
                &self.source.base_commit,
                &self.source.source_tree,
                &self.source.tested_tree,
            ]
            .iter()
            .all(|v| validate_object_id("CI Git object id", v).is_ok())
            && [
                &self.command.explicit_environment_digest,
                &self.command.effective_environment_digest,
                &self.source.repository_id,
                &self.source.snapshot_digest,
                &self.source.cargo_lock_digest,
                &self.platform_digest,
                &self.toolchain_digest,
                &self.producer_runtime_digest,
                &self.scope_digest,
                &self.policy_digest,
                &self.governance_digest,
                &self.profile_digest,
                &self.coverage.manifest_digest,
                &self.coverage.metadata_digest,
                &self.coverage.target_inventory_digest,
            ]
            .iter()
            .all(|v| Digest::from_str(v.as_str()).is_ok())
    }
}

/// Return unequal binding dimensions in a stable order, or a fail-closed
/// malformed/unsupported-input reason. An empty result
/// means only that the supplied complete fact objects are equal. It is NOT
/// provider authentication, evidence acceptance, or permission to skip work.
/// The current object must be derived by Runtime from current local policy;
/// callers must never construct it from the candidate artifact.
pub fn compare_ci_node_facts(current: &CiNodeFacts, producer: &CiNodeFacts) -> Vec<CiNodeMismatch> {
    // Conservative local TDD baseline: matching facts are not implemented yet.
    // This baseline cannot authorize admission or skip execution.
    let mut result = Vec::new();
    if !current.valid() {
        result.push(CiNodeMismatch::InvalidCurrent);
    }
    if !producer.valid() {
        result.push(CiNodeMismatch::InvalidProducer);
    }
    if result.is_empty() {
        result.push(CiNodeMismatch::InvalidCurrent);
    }
    result
}

#[cfg(test)]
mod ci_node_fact_tests {
    use super::*;

    fn digest(label: &str) -> Digest {
        Digest::sha256_bytes(label.as_bytes())
    }

    fn fixture() -> CiNodeFacts {
        CiNodeFacts {
            schema_version: CI_NODE_FACTS_SCHEMA_VERSION,
            node_id: "project-command-1".into(),
            command: CiCommandFacts {
                logical_program: "cargo".into(),
                logical_args: vec![
                    "test".into(),
                    "--locked".into(),
                    "-p".into(),
                    "cockpit-core".into(),
                    "--all-targets".into(),
                    "--".into(),
                    "--test-threads=1".into(),
                ],
                execution_program: "/opt/pinned/cargo".into(),
                execution_args: vec![
                    "test".into(),
                    "--locked".into(),
                    "-p".into(),
                    "cockpit-core".into(),
                    "--all-targets".into(),
                    "--".into(),
                    "--test-threads=1".into(),
                ],
                cwd_bytes: b"/fixed/checkout".to_vec(),
                explicit_environment_digest: digest("explicit env bytes"),
                effective_environment_digest: digest("all actual child env bytes"),
                clear_environment: true,
                resource_weight: 1,
                timeout_seconds: 900,
                dependencies: vec![],
            },
            source: CiSourceFacts {
                repository_id: digest("attached repository id"),
                snapshot_digest: digest("repository snapshot"),
                source_commit: "a".repeat(40),
                tested_commit: "b".repeat(40),
                base_commit: "c".repeat(40),
                source_tree: "d".repeat(40),
                tested_tree: "e".repeat(40),
                cargo_lock_digest: digest("lock"),
            },
            platform_digest: digest("linux x86_64 host target runner"),
            toolchain_digest: digest("exact tools"),
            producer_runtime_digest: digest("receipt authoring runtime"),
            scope_digest: digest("scope"),
            policy_digest: digest("policy"),
            governance_digest: digest("governance"),
            profile_digest: digest("profile"),
            stage: "task".into(),
            coverage: CiCoverageFacts {
                manifest_digest: digest("manifest"),
                metadata_digest: digest("metadata"),
                target_inventory_digest: digest("metadata-derived target inventory"),
                selected_targets: vec!["cockpit-core:lib".into()],
                features: vec!["default".into()],
                complete: true,
                warning_capture_complete: true,
            },
        }
    }

    #[test]
    fn identical_complete_facts_match_without_becoming_admission() {
        let facts = fixture();
        assert!(compare_ci_node_facts(&facts, &facts).is_empty());
        // There is intentionally no receipt writer, executor, provider-status
        // input or conversion from this result to ReuseAction in this API.
    }

    #[test]
    fn every_identity_dimension_is_compared_without_normalization() {
        type Mutation = (CiNodeMismatch, fn(&mut CiNodeFacts));
        let mutations: Vec<Mutation> = vec![
            (CiNodeMismatch::NodeId, |v| v.node_id.push('x')),
            (CiNodeMismatch::LogicalProgram, |v| {
                v.command.logical_program.push('x')
            }),
            (CiNodeMismatch::LogicalArgs, |v| {
                v.command.logical_args.push("--lib".into())
            }),
            (CiNodeMismatch::ExecutionProgram, |v| {
                v.command.execution_program.push('x')
            }),
            (CiNodeMismatch::ExecutionArgs, |v| {
                v.command.execution_args.reverse()
            }),
            (CiNodeMismatch::Cwd, |v| v.command.cwd_bytes.push(b'/')),
            (CiNodeMismatch::ExplicitEnvironment, |v| {
                v.command.explicit_environment_digest = digest("other explicit env")
            }),
            (CiNodeMismatch::EffectiveEnvironment, |v| {
                v.command.effective_environment_digest = digest("other inherited env")
            }),
            (CiNodeMismatch::ClearEnvironment, |v| {
                v.command.clear_environment = false
            }),
            (CiNodeMismatch::ResourceWeight, |v| {
                v.command.resource_weight = 2
            }),
            (CiNodeMismatch::Timeout, |v| v.command.timeout_seconds = 899),
            (CiNodeMismatch::UnsupportedDependencies, |v| {
                v.command.dependencies.push("prior".into())
            }),
            (CiNodeMismatch::Repository, |v| {
                v.source.repository_id = digest("other repo")
            }),
            (CiNodeMismatch::Snapshot, |v| {
                v.source.snapshot_digest = digest("other snapshot")
            }),
            (CiNodeMismatch::SourceCommit, |v| {
                v.source.source_commit = "f".repeat(40)
            }),
            (CiNodeMismatch::TestedCommit, |v| {
                v.source.tested_commit = "f".repeat(40)
            }),
            (CiNodeMismatch::BaseCommit, |v| {
                v.source.base_commit = "f".repeat(40)
            }),
            (CiNodeMismatch::SourceTree, |v| {
                v.source.source_tree = "f".repeat(40)
            }),
            (CiNodeMismatch::TestedTree, |v| {
                v.source.tested_tree = "f".repeat(40)
            }),
            (CiNodeMismatch::CargoLock, |v| {
                v.source.cargo_lock_digest = digest("other lock")
            }),
            (CiNodeMismatch::Platform, |v| {
                v.platform_digest = digest("mac arm64")
            }),
            (CiNodeMismatch::Toolchain, |v| {
                v.toolchain_digest = digest("other tools")
            }),
            (CiNodeMismatch::ProducerRuntime, |v| {
                v.producer_runtime_digest = digest("other runtime")
            }),
            (CiNodeMismatch::Scope, |v| {
                v.scope_digest = digest("other scope")
            }),
            (CiNodeMismatch::Policy, |v| {
                v.policy_digest = digest("other policy")
            }),
            (CiNodeMismatch::Governance, |v| {
                v.governance_digest = digest("other governance")
            }),
            (CiNodeMismatch::Profile, |v| {
                v.profile_digest = digest("other profile")
            }),
            (CiNodeMismatch::Stage, |v| v.stage = "release".into()),
            (CiNodeMismatch::CoverageManifest, |v| {
                v.coverage.manifest_digest = digest("other manifest")
            }),
            (CiNodeMismatch::Metadata, |v| {
                v.coverage.metadata_digest = digest("other metadata")
            }),
            (CiNodeMismatch::TargetInventory, |v| {
                v.coverage.target_inventory_digest = digest("other inventory")
            }),
            (CiNodeMismatch::TargetSelection, |v| {
                v.coverage
                    .selected_targets
                    .push("cockpit-core:example:demo".into())
            }),
            (CiNodeMismatch::Features, |v| {
                v.coverage.features.push("test-support".into())
            }),
        ];
        let current = fixture();
        for (expected, mutate) in mutations {
            let mut claimed = current.clone();
            mutate(&mut claimed);
            assert_eq!(compare_ci_node_facts(&current, &claimed), vec![expected]);
        }
    }

    #[test]
    fn same_incomplete_or_malformed_claim_never_matches() {
        let mut facts = fixture();
        facts.coverage.complete = false;
        assert_eq!(
            compare_ci_node_facts(&facts, &facts),
            vec![CiNodeMismatch::CoverageIncomplete]
        );
        facts.coverage.complete = true;
        facts.coverage.warning_capture_complete = false;
        assert_eq!(
            compare_ci_node_facts(&facts, &facts),
            vec![CiNodeMismatch::WarningCaptureIncomplete]
        );
        facts.coverage.warning_capture_complete = true;
        facts.command.timeout_seconds = 0;
        assert_eq!(
            compare_ci_node_facts(&facts, &facts),
            vec![
                CiNodeMismatch::InvalidCurrent,
                CiNodeMismatch::InvalidProducer
            ]
        );
    }

    #[test]
    fn duplicate_or_empty_target_inventory_is_invalid() {
        let current = fixture();
        for targets in [vec![], vec!["".into()], vec!["same".into(), "same".into()]] {
            let mut claimed = current.clone();
            claimed.coverage.selected_targets = targets;
            assert_eq!(
                compare_ci_node_facts(&current, &claimed),
                vec![CiNodeMismatch::InvalidProducer]
            );
        }
    }

    #[test]
    fn malformed_identity_fields_are_invalid() {
        let mutations: [fn(&mut CiNodeFacts); 8] = [
            |v| v.schema_version = 0,
            |v| v.node_id.push('\0'),
            |v| v.command.logical_program.clear(),
            |v| v.command.execution_args.push("bad\0arg".into()),
            |v| v.source.source_commit = "A".repeat(40),
            |v| v.command.timeout_seconds = 0,
            |v| v.command.timeout_seconds = 901,
            |v| v.stage = "future".into(),
        ];
        let current = fixture();
        for mutate in mutations {
            let mut claimed = current.clone();
            mutate(&mut claimed);
            assert_eq!(
                compare_ci_node_facts(&current, &claimed),
                vec![CiNodeMismatch::InvalidProducer]
            );
        }
    }

    #[test]
    fn equal_unknown_stage_and_dependency_claims_stay_unsupported() {
        let mut facts = fixture();
        for invalid_stage in ["future", "pull_request", ""] {
            facts.stage = invalid_stage.into();
            assert_eq!(
                compare_ci_node_facts(&facts, &facts),
                vec![
                    CiNodeMismatch::InvalidCurrent,
                    CiNodeMismatch::InvalidProducer
                ]
            );
        }
        for valid_stage in ["task", "pre_ci", "pr", "merge", "release"] {
            facts.stage = valid_stage.into();
            assert!(compare_ci_node_facts(&facts, &facts).is_empty());
        }
        facts.command.dependencies = vec!["same-upstream-node".into()];
        assert_eq!(
            compare_ci_node_facts(&facts, &facts),
            vec![CiNodeMismatch::UnsupportedDependencies]
        );
    }

    #[test]
    fn strict_schema_rejects_missing_unknown_and_untyped_legacy_fields() {
        let value = serde_json::to_value(fixture()).expect("serialize fixture");
        for missing in ["producerRuntimeDigest", "coverage", "platformDigest"] {
            let mut v = value.clone();
            v.as_object_mut().expect("object").remove(missing);
            assert!(serde_json::from_value::<CiNodeFacts>(v).is_err());
        }
        let mut v = value;
        v["providerAuthenticated"] = serde_json::json!(true);
        assert!(serde_json::from_value::<CiNodeFacts>(v).is_err());
    }
}
