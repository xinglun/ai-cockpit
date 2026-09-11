use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use chrono::{DateTime, Utc};
use cockpit_core::{
    ActionKind, AuthorityState, DecisionState, Digest, EvidenceState, GovernanceDecision,
    GovernanceInput, evaluate,
};
use cockpit_git::{ChangeContentState, ChangeKind, RepositorySnapshot};
use cockpit_protocol::{
    AdopterCapabilityState, AdopterCapabilityTruth, AgentAdapterCompatibility,
    AgentInterfaceAvailability, AgentInterfaceManifest, AgentInterfaces, AgentRootBinding,
    ApprovalMode, AuditEvent, AuditExportManifest, CapabilityConfidence, CapabilityExclusion,
    CapabilityOwnership, CapabilityTruth, CapabilityTruthRegistry, CheckpointEvidence,
    ConcurrencyBoundary, Contract, DataClassification, DelegatedEvidence, DelegatedEvidenceReceipt,
    DiagnosisState, EvidenceAssurance, EvidenceDisposition, EvidenceDispositionItem,
    EvidencePersistence, EvidenceRetention, EvidenceRetentionPolicy, EvidenceValidity, FactOrigin,
    FinalizationErrorCode, GovernanceCost, GovernancePolicy, GovernancePolicyDocument,
    HistoricalFinalizationKind, HistoricalFinalizationRecoveryReceipt, HumanBenefitReport,
    HumanDecision, ImplementationApproach, OutcomeClaim, OutcomeReportBindings,
    OutcomeReportSections, OutcomeState, OutcomeV2, PARALLEL_SLOT_LEASE_SCHEMA_VERSION,
    ParallelSlotLease, PerformanceCounters, PerformanceDiagnosis, PerformancePhase, PolicyLayer,
    ProjectGovernanceProjection, QualityCommand, RecoveryDecisionReceipt, RepositoryConfig,
    ResourceFinalizationContext, ResourceFinalizationDisposition, ResourceFinalizationReceipt,
    ResourceFinalizationTransitionReceipt, RuntimeContext, SchemaMigrationStep, TaskOutcomeEvent,
    TaskOutcomeReport, TruthState, VerificationStage, VerificationTier, WorkItemCompatibility,
    WorkItemEvidenceFreshness, WorkItemIntelligence, WorkItemStatusIndex, WorkItemStatusIndexEntry,
    WorkItemStatusSnapshot, default_repository_schema_version, merge_policy_layers,
    repository_schema_migration_chain, validate_evidence_retention,
    validate_historical_finalization_recovery, validate_protocol_version,
    validate_resource_finalization_receipt_for, validate_resource_finalization_replay,
    validate_resource_finalization_transition,
};
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer as _, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
#[cfg(unix)]
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;

mod evidence_store;
mod execution_context;
mod governance_controls;
mod lifecycle;
mod observation_ledger;
mod outcome_render;
mod project_governance;
mod status_projection;

pub use evidence_store::{
    ReceiptStoreBinding, ReceiptStoreLoad, ReceiptStoreWrite, load_reusable_receipt,
    persist_reusable_receipt,
};
use evidence_store::{
    create_and_open_cap_directory, open_cap_directory_nofollow_strict, open_or_create_cap_nofollow,
    read_cap_file_nofollow_bounded, valid_sha256_digest,
};
#[cfg(test)]
use execution_context::execution_environment_digest_from_values;
pub use execution_context::{
    ObservationConsistency, ObservationContext, ObservationPhase, RepositoryExecutionContext,
    RuntimeSession, VerificationContextInput, VerificationReuseAssessment,
    VerificationReuseAuthorization, assess_verification_reuse,
};
use execution_context::{
    VerificationIdentityCost, assess_verification_reuse_measured,
    build_repository_verification_command, refresh_verification_context,
    resolved_executable_identity, valid_git_object_id,
};
pub use governance_controls::*;
pub use lifecycle::*;
use lifecycle::{
    RECOVERY_DECISION_INVALID, contract_digest, contract_digest_for_evidence, decision_state_name,
    recovery_decision_error, validate_archived_revalidation_evidence,
    validate_recovery_predecessor_bindings, validate_recovery_successor_binding,
    work_item_artifact_path,
};
pub use outcome_render::{
    FinalizationProjection, HumanDecisionProjection, OutcomeAssemblyMetadata, OutcomeRenderInput,
    OutcomeRenderView, outcome_render_input, outcome_render_input_from_outcome,
    outcome_render_input_with_runtime, render_full_human_outcome, render_human_outcome,
    render_human_outcome_with_view,
};
pub use project_governance::*;
use status_projection::{
    discover_worktree_layout, historical_finalization_inventory,
    recovery_successor_resolves_pending_close, repository_readiness, unclosed_archived_work_items,
};
pub use status_projection::{status, status_with_runtime};

static NEXT_ATOMIC_WRITE_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);
const MAX_RECEIPT_INDEX_BYTES: u64 = 8 * 1024 * 1024;
const MAX_REUSABLE_RECEIPT_BYTES: u64 = 1024 * 1024;
const MAX_VERIFICATION_IDENTITY_FILE_BYTES: u64 = 1024 * 1024;
const MAX_EXTERNAL_EVIDENCE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguageSignal {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildSystem {
    Cargo,
    Npm,
    Poetry,
    Go,
    Make,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryObservation {
    pub snapshot_digest: Digest,
    pub dependency_fingerprint: Digest,
    pub languages: Vec<LanguageSignal>,
    pub build_systems: Vec<BuildSystem>,
    pub test_roots: Vec<String>,
    pub quality_commands: Vec<QualityCommand>,
    pub ci_surfaces: Vec<String>,
    pub critical_domains: Vec<String>,
    pub files_read: usize,
    #[serde(default)]
    pub cache_hit: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvolutionClass {
    L0,
    L1,
    L2,
    L3,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvolutionEvent {
    pub class: EvolutionClass,
    pub event_type: String,
    pub path: String,
    pub action: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUpdateProposal {
    pub from_profile_version: u64,
    pub candidate: String,
    pub reason: String,
    pub requires_human_confirmation: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttachedProfile {
    pub profile_version: u64,
    pub repository_id: String,
    #[serde(default = "default_repository_schema_version")]
    pub repository_schema_version: u32,
    pub state: String,
    #[serde(default)]
    pub profile_digest: Option<Digest>,
    pub tests: Vec<QualityCommand>,
    pub build_systems: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepositoryVerificationPolicy {
    ProfileAuthorized,
    NeverReuse,
    Protected(cockpit_verification::ProtectedGateClass),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryVerificationRequest {
    pub node_id: String,
    pub program: String,
    pub args: Vec<String>,
    pub scope: Vec<String>,
    pub stage: String,
    pub runner: String,
    pub runtime_digest: String,
    pub base_commit: Option<String>,
    pub workers: usize,
    pub policy: RepositoryVerificationPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryVerificationRun {
    pub receipt: cockpit_verification::VerificationReceipt,
    pub final_snapshot: RepositorySnapshot,
}

/// The request-scoped route selected for a Work Item.  Policy is optional for
/// protocol-v1/no-policy repositories; when present, the requirement and its
/// traceability facts are carried into the execution receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationRoute {
    pub work_item_id: String,
    pub operation: String,
    pub stage: VerificationStage,
    pub policy_plan: Option<cockpit_verification::PolicyVerificationPlan>,
    pub actual_tier: VerificationTier,
    pub actual_assurance: EvidenceAssurance,
    pub base_revision: Option<String>,
    pub affected_paths: Vec<String>,
    pub dependency_confidence: cockpit_verification::DependencyConfidence,
}

/// Read-only CI authority produced from the same Contract, repository
/// snapshot, and policy route used by lifecycle verification.  It is a
/// projection receipt only: the gate never writes `.ai/` state and never
/// creates a human decision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractQualityGateReport {
    pub schema_version: u32,
    pub kind: String,
    pub state: String,
    pub repository_id: Digest,
    pub work_item_id: String,
    pub contract_digest: Digest,
    pub contract_file_digest: Digest,
    pub repository_snapshot_digest: Digest,
    pub base_revision: String,
    pub head_revision: Option<String>,
    pub changed_paths: Vec<String>,
    pub stage: String,
    pub runner: String,
    pub operation: String,
    pub verification_tier: VerificationTier,
    pub evidence_assurance: EvidenceAssurance,
    pub dependency_confidence: cockpit_verification::DependencyConfidence,
    pub decision_state: String,
    pub blockers: Vec<String>,
    pub unknowns: Vec<String>,
    pub required_checks: Vec<String>,
    pub runtime_version: String,
    pub runtime_digest: Digest,
    pub receipt_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryReadiness {
    pub state: String,
    pub ready_on_base: bool,
    pub blockers: Vec<String>,
    pub unknowns: Vec<String>,
    pub current_branch: Option<String>,
    pub default_remote: Option<String>,
    pub default_branch: Option<String>,
    pub current_revision: Option<String>,
    pub default_revision: Option<String>,
    pub dirty_paths: Vec<String>,
    pub unclosed_archived_work_items: Vec<String>,
    /// Stable classification of repository-wide historical lifecycle debt.
    /// This guides recovery without weakening the pending-close gate.
    #[serde(default)]
    pub historical_debt: Vec<HistoricalDebtItem>,
    /// Recognized failed-attempt artifacts that have no active Contract.
    /// These are retained audit bytes, not counted as active Work Items.
    #[serde(default)]
    pub orphaned_active_artifacts: Vec<String>,
    /// Immutable finalization heads produced by an older Runtime.  These are
    /// discovery projections and never authorize a new lifecycle transition.
    #[serde(default)]
    pub historical_finalization: Vec<HistoricalFinalizationInventoryItem>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HistoricalDebtItem {
    pub work_item_id: String,
    pub category: String,
    pub assurance: String,
    pub recovery_action: String,
}

/// Read-only inventory entry for an immutable finalization receipt produced by
/// an older Runtime.  This is discovery data, not an authorization receipt;
/// no predecessor bytes are rewritten by producing it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HistoricalFinalizationInventoryItem {
    pub work_item_id: String,
    pub state: String,
    pub assurance: String,
    pub predecessor_path: String,
    pub predecessor_digest: Option<Digest>,
    pub sequence: u64,
    pub runtime_version: Option<String>,
    pub runtime_digest: Option<Digest>,
    pub historical_kind: Option<String>,
    pub safe_actions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryStatus {
    pub protocol_version: u32,
    pub repository_schema_version: u32,
    pub repository_id: String,
    pub state: String,
    pub profile_version: u64,
    pub active_work_items: usize,
    pub archived_work_items: usize,
    /// Recognized Work Item artifact variants currently present in active.
    /// Canonical Contract/Summary files are represented by active_work_items;
    /// this list is for failed-attempt projections such as outcome.*.json.
    #[serde(default)]
    pub active_artifacts: Vec<String>,
    #[serde(default)]
    pub orphaned_active_artifacts: Vec<String>,
    pub readiness: RepositoryReadiness,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActiveArtifactReconciliationArtifact {
    pub source_path: String,
    pub target_path: String,
    pub digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActiveArtifactReconciliationReceipt {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub state: String,
    pub archive_manifest_path: String,
    pub moved_artifacts: Vec<ActiveArtifactReconciliationArtifact>,
    pub recorded_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepositoryCompatibility {
    pub runtime_version: String,
    pub runtime_digest: Digest,
    pub protocol_version: u32,
    pub repository_schema_version: u32,
    pub required_repository_schema_version: u32,
    pub state: String,
    pub safe_actions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MigrationPlan {
    pub state: String,
    pub current_schema: u32,
    pub target_schema: u32,
    pub migration_type: String,
    pub planned_changes: Vec<String>,
    pub unchanged: Vec<String>,
    pub human_approval_required: bool,
    pub steps: Vec<SchemaMigrationStep>,
    /// Historical finalization records that need explicit compatibility
    /// handling when this Runtime is newer than their producer.
    #[serde(default)]
    pub historical_finalization: Vec<HistoricalFinalizationInventoryItem>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MigrationReceipt {
    pub schema_version: u32,
    pub migration_id: String,
    pub from_schema: u32,
    pub to_schema: u32,
    pub runtime_version: String,
    pub runtime_digest: Digest,
    pub before_digest: Digest,
    pub after_digest: Digest,
    pub changes: Vec<String>,
    pub result: String,
    pub created_at: String,
    pub step: SchemaMigrationStep,
    pub chain_length: usize,
    pub preserved_evidence_digest: Digest,
    pub preserved_paths: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GovernanceSignalAssessment {
    pub untrusted_material: bool,
    pub test_weakening: bool,
    pub coverage_weakening: bool,
    pub unknowns: Vec<String>,
    pub findings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleReceipt {
    pub work_item_id: String,
    pub state: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkItemStartOptions {
    pub out_of_scope: Vec<String>,
    pub risk: String,
    pub authority: String,
    pub acceptance_criteria: Vec<String>,
    pub required_evidence_classes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItemScaffoldFacts {
    pub repository_id: String,
    pub base_revision: String,
    pub project_profile_digest: Digest,
    pub repository_snapshot_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItemScaffoldReceipt {
    pub work_item_id: String,
    pub mode: String,
    pub contract_path: String,
    pub state: String,
    pub known_facts: WorkItemScaffoldFacts,
    pub human_input_required: Vec<String>,
}

/// The persisted Work Item verification envelope.  This is deliberately
/// stricter than the JSON produced by a one-shot execution: every field is
/// required, unknown envelope fields are rejected, and a captured receipt is
/// deserialized through `VerificationReceipt` (which has the same strict
/// policy for its nested result/candidate records).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationCaptureMode {
    FullCapture,
    RedactedCapture,
    DigestOnly,
    /// Compatibility lane for the pre-v2 public Rust API, whose unit tests
    /// and callers supplied a small arbitrary JSON value instead of the
    /// Runtime's typed execution receipt.  It is never accepted by a
    /// Runtime-bound lifecycle operation and is not emitted by the CLI.
    LegacyUntyped,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VerificationEvidenceV2 {
    pub protocol_version: u32,
    pub evidence_schema_version: u32,
    pub work_item_id: String,
    pub repository_id: String,
    pub runtime_version: String,
    pub runtime_digest: Digest,
    pub repository_snapshot_digest: Digest,
    #[serde(default)]
    pub contract_digest: Option<Digest>,
    pub passed: bool,
    pub receipt_digest: Digest,
    pub capture_mode: VerificationCaptureMode,
    pub created_at: String,
    #[serde(default)]
    pub receipt: Option<cockpit_verification::VerificationReceipt>,
    #[serde(default)]
    pub retention: Option<EvidenceRetentionPolicy>,
}

/// Strict envelope parser used before nested receipt validation.  Keeping the
/// raw receipt as a `Value` here lets the compatibility lane read old
/// untyped payloads while the v2 capture modes below always deserialize it as
/// `VerificationReceipt` with `deny_unknown_fields`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VerificationEvidenceEnvelope {
    protocol_version: u32,
    evidence_schema_version: u32,
    work_item_id: String,
    repository_id: String,
    runtime_version: String,
    runtime_digest: Digest,
    repository_snapshot_digest: Digest,
    #[serde(default)]
    contract_digest: Option<Digest>,
    passed: bool,
    receipt_digest: Digest,
    capture_mode: VerificationCaptureMode,
    created_at: String,
    #[serde(default)]
    receipt: Option<serde_json::Value>,
    #[serde(default)]
    retention: Option<EvidenceRetentionPolicy>,
}

#[derive(Debug, Error)]
pub enum ObserverError {
    #[error("failed to read repository entry {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("repository snapshot root does not match observer root")]
    SnapshotRootMismatch,
    #[error("repository protocol state error at {path}: {message}")]
    State { path: PathBuf, message: String },
    #[error("resource finalization observation error at {path} ({code:?}): {diagnostic}")]
    FinalizationObservation {
        path: PathBuf,
        code: FinalizationErrorCode,
        diagnostic: String,
    },
}

fn finalization_observation_error(
    path: impl Into<PathBuf>,
    code: FinalizationErrorCode,
    error: impl std::fmt::Display,
) -> ObserverError {
    ObserverError::FinalizationObservation {
        path: path.into(),
        code,
        diagnostic: error.to_string(),
    }
}

fn path_derived_repository_id(root: &Path) -> Digest {
    Digest::sha256_bytes(root.to_string_lossy().as_bytes())
}

fn stored_repository_id(root: &Path) -> Option<Digest> {
    let config = fs::read_to_string(root.join(".ai/cockpit.toml")).ok()?;
    let config: RepositoryConfig = toml::from_str(&config).ok()?;
    config.repository_id.parse().ok()
}

/// Return the repository identity bound to an attached repository.
///
/// An attached repository owns the value in `.ai/cockpit.toml`; the path hash
/// is only a compatibility fallback for un-attached test fixtures and cannot
/// authorize a repository-local receipt on its own.
pub fn repository_id(root: &Path) -> Digest {
    stored_repository_id(root).unwrap_or_else(|| path_derived_repository_id(root))
}

fn new_repository_id() -> Digest {
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    Digest::sha256_bytes(
        format!(
            "ai-cockpit:repository:{}:{}:{}",
            std::process::id(),
            timestamp,
            sequence
        )
        .as_bytes(),
    )
}

pub fn run_repository_verification(
    root: &Path,
    request: &RepositoryVerificationRequest,
) -> Result<RepositoryVerificationRun, ObserverError> {
    let service_started = Instant::now();
    let stage = VerificationStage::parse(&request.stage).map_err(|error| ObserverError::State {
        path: root.to_path_buf(),
        message: error,
    })?;
    if stage.requires_base_revision()
        && request
            .base_commit
            .as_deref()
            .is_none_or(|value| !valid_git_object_id(value))
    {
        return Err(ObserverError::State {
            path: root.to_path_buf(),
            message: format!(
                "verification stage {} requires a valid base revision",
                stage.as_str()
            ),
        });
    }
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let mut store_files_read = 0;
    let mut store_unavailable_reason = None;
    let execution_identity = resolved_executable_identity(&root, &request.program);
    let mut executable_files_read = execution_identity
        .as_ref()
        .map_or(0, |identity| identity.components.len());
    let mut executable_files_hashed = executable_files_read;
    let base_command = build_repository_verification_command(
        &root,
        request,
        None,
        match &request.policy {
            RepositoryVerificationPolicy::ProfileAuthorized => {
                cockpit_verification::VerificationReusePolicy::Reusable
            }
            RepositoryVerificationPolicy::NeverReuse => {
                cockpit_verification::VerificationReusePolicy::NeverReuse
            }
            RepositoryVerificationPolicy::Protected(class) => {
                cockpit_verification::VerificationReusePolicy::Protected(class.clone())
            }
        },
    );
    let context_input = VerificationContextInput {
        program: request.program.clone(),
        args: request.args.clone(),
        command_digest: base_command.command_digest(),
        scope: request.scope.clone(),
        stage: request.stage.clone(),
        runner: request.runner.clone(),
        runtime_digest: request.runtime_digest.clone(),
        base_commit: request.base_commit.clone(),
    };
    let mut authorized_binding = None;
    let command = if request.policy == RepositoryVerificationPolicy::ProfileAuthorized {
        let mut assessment_cost = VerificationIdentityCost::default();
        let assessment = assess_verification_reuse_measured(
            &root,
            &snapshot,
            &context_input,
            execution_identity.as_ref(),
            &mut assessment_cost,
        );
        executable_files_read = executable_files_read.saturating_add(assessment_cost.files_read);
        executable_files_hashed =
            executable_files_hashed.saturating_add(assessment_cost.files_hashed);
        match assessment {
            Ok(VerificationReuseAssessment::Authorized(authorization)) => {
                let authorization = *authorization;
                let context = authorization.context.clone();
                let binding = ReceiptStoreBinding {
                    repository_id: repository_id(&root).to_string(),
                    profile_digest: context.profile_digest.clone(),
                    node_id: request.node_id.clone(),
                };
                let candidate = match load_reusable_receipt(&root, &binding)? {
                    ReceiptStoreLoad::Candidate {
                        receipt,
                        files_read,
                    } => {
                        store_files_read += files_read;
                        Some(*receipt)
                    }
                    ReceiptStoreLoad::Unavailable { reason, files_read } => {
                        store_files_read += files_read;
                        store_unavailable_reason = Some(reason);
                        None
                    }
                };
                authorized_binding = Some((binding, authorization));
                build_repository_verification_command(
                    &root,
                    request,
                    execution_identity.as_ref(),
                    cockpit_verification::VerificationReusePolicy::Reusable,
                )
                .with_reuse_candidate(candidate, context)
            }
            Ok(VerificationReuseAssessment::Denied { .. }) | Err(_) => {
                build_repository_verification_command(
                    &root,
                    request,
                    None,
                    cockpit_verification::VerificationReusePolicy::NeverReuse,
                )
            }
        }
    } else {
        base_command
    };
    let mut receipt = cockpit_verification::execute_bounded(vec![command], request.workers)
        .map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    if let Some(reason) = store_unavailable_reason.take()
        && let Some(result) = receipt
            .results
            .iter_mut()
            .find(|result| result.reason == "evidence_missing")
    {
        result.reason = reason;
    }
    let mut final_snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let mut snapshot_git_calls = snapshot.git_calls.saturating_add(final_snapshot.git_calls);
    let mut snapshot_files_read = snapshot
        .files_read
        .saturating_add(final_snapshot.files_read);
    let mut snapshot_files_hashed = snapshot
        .files_hashed
        .saturating_add(final_snapshot.files_hashed);

    let mut post_execution_identity = None;
    let mut post_context = if let Some((_, authorization)) = &authorized_binding {
        post_execution_identity = resolved_executable_identity(&root, &request.program);
        if let Some(identity) = &post_execution_identity {
            executable_files_read = executable_files_read.saturating_add(identity.components.len());
            executable_files_hashed =
                executable_files_hashed.saturating_add(identity.components.len());
        }
        let mut refresh_cost = VerificationIdentityCost::default();
        let refreshed = refresh_verification_context(
            &root,
            &final_snapshot,
            &context_input,
            post_execution_identity.as_ref(),
            authorization,
            &mut refresh_cost,
        )?;
        executable_files_read = executable_files_read.saturating_add(refresh_cost.files_read);
        executable_files_hashed = executable_files_hashed.saturating_add(refresh_cost.files_hashed);
        refreshed
    } else {
        None
    };
    let post_stable = authorized_binding
        .as_ref()
        .zip(post_context.as_ref())
        .is_some_and(|((_, before), after)| before.context == *after);
    if receipt.nodes_reused > 0 && !post_stable {
        let command = if let Some(context) = post_context.clone() {
            let mut authorization = authorized_binding
                .as_ref()
                .expect("post context requires authorization")
                .1
                .clone();
            authorization.context = context.clone();
            authorized_binding = Some((
                ReceiptStoreBinding {
                    repository_id: repository_id(&root).to_string(),
                    profile_digest: context.profile_digest.clone(),
                    node_id: request.node_id.clone(),
                },
                authorization,
            ));
            build_repository_verification_command(
                &root,
                request,
                post_execution_identity.as_ref(),
                cockpit_verification::VerificationReusePolicy::Reusable,
            )
            .with_reuse_candidate(None, context)
        } else {
            authorized_binding = None;
            build_repository_verification_command(
                &root,
                request,
                None,
                cockpit_verification::VerificationReusePolicy::NeverReuse,
            )
        };
        receipt = cockpit_verification::execute_bounded(vec![command], request.workers).map_err(
            |error| ObserverError::State {
                path: root.clone(),
                message: error.to_string(),
            },
        )?;
        if let Some(result) = receipt.results.first_mut() {
            result.reason = "post_planning_binding_drift".into();
        }
        final_snapshot = git.snapshot().map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
        snapshot_git_calls = snapshot_git_calls.saturating_add(final_snapshot.git_calls);
        snapshot_files_read = snapshot_files_read.saturating_add(final_snapshot.files_read);
        snapshot_files_hashed = snapshot_files_hashed.saturating_add(final_snapshot.files_hashed);
        let final_execution_identity = if authorized_binding.is_some() {
            resolved_executable_identity(&root, &request.program)
        } else {
            None
        };
        if let Some(identity) = &final_execution_identity {
            executable_files_read = executable_files_read.saturating_add(identity.components.len());
            executable_files_hashed =
                executable_files_hashed.saturating_add(identity.components.len());
        }
        post_context = if let Some((_, authorization)) = &authorized_binding {
            let mut refresh_cost = VerificationIdentityCost::default();
            let refreshed = refresh_verification_context(
                &root,
                &final_snapshot,
                &context_input,
                final_execution_identity.as_ref(),
                authorization,
                &mut refresh_cost,
            )?;
            executable_files_read = executable_files_read.saturating_add(refresh_cost.files_read);
            executable_files_hashed =
                executable_files_hashed.saturating_add(refresh_cost.files_hashed);
            refreshed
        } else {
            None
        };
    }

    if !receipt.receipt_candidates.is_empty() {
        let stable = authorized_binding
            .as_ref()
            .zip(post_context.as_ref())
            .is_some_and(|((_, before), after)| before.context == *after);
        if stable {
            let (binding, _) = authorized_binding.as_ref().expect("stable binding exists");
            for candidate in &receipt.receipt_candidates {
                store_files_read += persist_reusable_receipt(&root, binding, candidate)?.files_read;
            }
        } else {
            receipt.receipt_candidates.clear();
            for result in &mut receipt.results {
                if !result.reused {
                    result.receipt_id = None;
                    result.reason = "post_execution_binding_drift".into();
                }
            }
        }
    }
    receipt.git_calls = snapshot_git_calls;
    receipt.files_read = receipt
        .files_read
        .saturating_add(snapshot_files_read)
        .saturating_add(store_files_read)
        .saturating_add(executable_files_read);
    receipt.files_hashed = receipt
        .files_hashed
        .saturating_add(snapshot_files_hashed)
        .saturating_add(executable_files_hashed);
    receipt.elapsed_ms = service_started.elapsed().as_millis();
    receipt.repository_id = Some(repository_id(&root).to_string());
    {
        let mut plan_receipt = cockpit_verification::VerificationPlanReceipt::new(
            stage,
            cockpit_protocol::VerificationTier::T0,
            cockpit_protocol::VerificationTier::T0,
            cockpit_protocol::EvidenceAssurance::SelfDeclared,
            vec!["repository_route_stage_explicit".into()],
            Vec::new(),
        )
        .map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error,
        })?;
        plan_receipt.executed_nodes = receipt
            .results
            .iter()
            .filter(|result| !result.reused)
            .map(|result| result.node_id.clone())
            .collect();
        plan_receipt.reused_nodes = receipt
            .results
            .iter()
            .filter(|result| result.reused)
            .map(|result| result.node_id.clone())
            .collect();
        plan_receipt.planning_elapsed_ms = receipt.planning_elapsed_ms;
        plan_receipt.execution_elapsed_ms = receipt.execution_elapsed_ms;
        plan_receipt.saved_executions = receipt.nodes_reused;
        receipt.plan_receipt = Some(plan_receipt);
    }
    Ok(RepositoryVerificationRun {
        receipt,
        final_snapshot,
    })
}

pub fn attach(root: &Path) -> Result<AttachedProfile, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let observation = observe(&root, &snapshot)?;
    let ai = root.join(".ai");
    let config_path = ai.join("cockpit.toml");
    let legacy_id = path_derived_repository_id(&root).to_string();
    let mut id = new_repository_id().to_string();
    if config_path.is_file() {
        let existing = fs::read_to_string(&config_path).map_err(|source| ObserverError::Read {
            path: config_path.clone(),
            source,
        })?;
        let config: RepositoryConfig =
            toml::from_str(&existing).map_err(|error| ObserverError::State {
                path: config_path.clone(),
                message: error.to_string(),
            })?;
        validate_protocol_version(config.protocol_version).map_err(|error| {
            ObserverError::State {
                path: config_path.clone(),
                message: error.to_string(),
            }
        })?;
        if config.repository_schema_version != cockpit_protocol::REPOSITORY_SCHEMA_VERSION {
            return Err(ObserverError::State {
                path: config_path.clone(),
                message: format!(
                    "repository schema {} requires explicit migration to {}",
                    config.repository_schema_version,
                    cockpit_protocol::REPOSITORY_SCHEMA_VERSION
                ),
            });
        }
        if config.repository_id == legacy_id {
            // Rebind repositories created by the pre-attach path-derived
            // implementation to a durable identity on the next explicit
            // attach. A current attached identity remains idempotent.
        } else if config.repository_id.parse::<Digest>().is_ok() {
            id = config.repository_id;
        } else {
            return Err(ObserverError::State {
                path: config_path.clone(),
                message: "repository identity is not a valid stable digest".into(),
            });
        }
    }
    for directory in [
        ai.join("work-items/active"),
        ai.join("work-items/archive"),
        ai.join("decisions"),
        ai.join("evidence"),
        ai.join("knowledge"),
    ] {
        fs::create_dir_all(&directory).map_err(|source| ObserverError::Read {
            path: directory,
            source,
        })?;
    }
    let config = format!(
        "protocol_version = 1\nrepository_schema_version = {}\nrepository_id = \"{id}\"\n",
        cockpit_protocol::REPOSITORY_SCHEMA_VERSION
    );
    atomic_write(&ai.join("cockpit.toml"), config.as_bytes())?;
    let profile_digest = cockpit_protocol::digest_json(&cockpit_protocol::ProjectProfile {
        profile_version: 1,
        repository_id: id.clone(),
        tests: observation.quality_commands.clone(),
        build_systems: observation
            .build_systems
            .iter()
            .map(|value| format!("{value:?}"))
            .collect(),
    })
    .map_err(|error| ObserverError::State {
        path: ai.join("project.json"),
        message: error.to_string(),
    })?;
    let profile = AttachedProfile {
        profile_version: 1,
        repository_id: id,
        repository_schema_version: cockpit_protocol::REPOSITORY_SCHEMA_VERSION,
        state: "calibration_required".into(),
        profile_digest: Some(profile_digest.clone()),
        tests: observation.quality_commands,
        build_systems: observation
            .build_systems
            .iter()
            .map(|value| format!("{value:?}"))
            .collect(),
    };
    let encoded = serde_json::to_vec_pretty(&profile).map_err(|error| ObserverError::State {
        path: ai.join("project.json"),
        message: error.to_string(),
    })?;
    atomic_write(&ai.join("project.json"), &encoded)?;
    let manifest = AgentInterfaceManifest {
        schema_version: 1,
        protocol_version: 1,
        repository_schema_version: cockpit_protocol::REPOSITORY_SCHEMA_VERSION,
        interface_version: 1,
        repository_id: profile.repository_id.clone(),
        root_binding: AgentRootBinding {
            binding_type: "manifest-parent".into(),
        },
        capabilities: cockpit_protocol::AGENT_INTERFACE_CAPABILITIES
            .iter()
            .map(|capability| (*capability).into())
            .collect(),
        interfaces: AgentInterfaces {
            cli: AgentInterfaceAvailability {
                available: true,
                transport: None,
            },
            mcp: AgentInterfaceAvailability {
                available: true,
                transport: Some("stdio".into()),
            },
        },
        adapter: AgentAdapterCompatibility { required: false },
        adapter_state: "unconfigured".into(),
    };
    let manifest_value = serde_json::to_value(&manifest).map_err(|error| ObserverError::State {
        path: ai.join("agent-interface.json"),
        message: error.to_string(),
    })?;
    atomic_json(&ai.join("agent-interface.json"), &manifest_value)?;
    let proposal = serde_json::json!({
        "kind": "project_profile_initialization",
        "profileVersion": 1,
        "profileDigest": profile_digest,
        "state": "calibration_required",
    });
    atomic_json(&ai.join("decisions/profile-v1.json"), &proposal)?;
    Ok(profile)
}

fn migration_inputs(
    root: &Path,
) -> Result<
    (
        RepositoryConfig,
        AttachedProfile,
        AgentInterfaceManifest,
        Vec<u8>,
    ),
    ObserverError,
> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let ai = root.join(".ai");
    let config_path = ai.join("cockpit.toml");
    let project_path = ai.join("project.json");
    let manifest_path = ai.join("agent-interface.json");
    let config_bytes = fs::read(&config_path).map_err(|source| ObserverError::Read {
        path: config_path.clone(),
        source,
    })?;
    let config: RepositoryConfig =
        toml::from_slice(&config_bytes).map_err(|error| ObserverError::State {
            path: config_path.clone(),
            message: error.to_string(),
        })?;
    validate_protocol_version(config.protocol_version).map_err(|error| ObserverError::State {
        path: config_path.clone(),
        message: error.to_string(),
    })?;
    let project_bytes = fs::read(&project_path).map_err(|source| ObserverError::Read {
        path: project_path.clone(),
        source,
    })?;
    let profile: AttachedProfile =
        serde_json::from_slice(&project_bytes).map_err(|error| ObserverError::State {
            path: project_path.clone(),
            message: error.to_string(),
        })?;
    let manifest_bytes = fs::read(&manifest_path).map_err(|source| ObserverError::Read {
        path: manifest_path.clone(),
        source,
    })?;
    let manifest: AgentInterfaceManifest =
        serde_json::from_slice(&manifest_bytes).map_err(|error| ObserverError::State {
            path: manifest_path.clone(),
            message: error.to_string(),
        })?;
    if config.repository_id != profile.repository_id
        || config.repository_id != manifest.repository_id
        || config.repository_schema_version != profile.repository_schema_version
        || config.repository_schema_version != manifest.repository_schema_version
    {
        return Err(ObserverError::State {
            path: config_path,
            message: "repository identity or schema versions disagree across protocol files".into(),
        });
    }
    let mut before = config_bytes;
    before.push(0);
    before.extend_from_slice(&project_bytes);
    before.push(0);
    before.extend_from_slice(&manifest_bytes);
    Ok((config, profile, manifest, before))
}

const MIGRATION_PRESERVED_PATHS: [&str; 4] = [
    ".ai/evidence",
    ".ai/decisions",
    ".ai/knowledge",
    ".ai/work-items/archive",
];

fn collect_preserved_files(
    directory: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), ObserverError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(ObserverError::Read {
                path: directory.to_path_buf(),
                source,
            });
        }
    };
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: directory.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        if metadata.file_type().is_symlink() {
            return Err(ObserverError::State {
                path,
                message: "migration preservation refuses symlinked historical evidence".into(),
            });
        }
        if metadata.is_dir() {
            collect_preserved_files(&path, files)?;
        } else if metadata.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn preserved_evidence_digest(root: &Path) -> Result<Digest, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let mut files = Vec::new();
    for relative in MIGRATION_PRESERVED_PATHS {
        collect_preserved_files(&root.join(relative), &mut files)?;
    }
    files.sort();
    let mut bytes = Vec::new();
    for path in files {
        let relative = path.strip_prefix(&root).map_err(|_| ObserverError::State {
            path: path.clone(),
            message: "historical evidence path escaped repository root".into(),
        })?;
        bytes.extend_from_slice(relative.to_string_lossy().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&fs::read(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?);
        bytes.push(0);
    }
    Ok(Digest::sha256_bytes(&bytes))
}

pub fn compatibility_report(
    root: &Path,
    runtime: &RuntimeContext,
) -> Result<RepositoryCompatibility, ObserverError> {
    let (config, _, _, _) = migration_inputs(root)?;
    let (state, safe_actions) =
        if config.repository_schema_version == cockpit_protocol::REPOSITORY_SCHEMA_VERSION {
            ("COMPATIBLE", Vec::new())
        } else if config.repository_schema_version < cockpit_protocol::REPOSITORY_SCHEMA_VERSION {
            (
                "MIGRATION_REQUIRED",
                vec![
                    "ai-cockpit migrate plan --repo <repository>".into(),
                    "ai-cockpit migrate apply --repo <repository> --approved".into(),
                ],
            )
        } else {
            (
                "INCOMPATIBLE",
                vec!["install a Runtime that supports this repository schema".into()],
            )
        };
    Ok(RepositoryCompatibility {
        runtime_version: runtime.runtime_version.clone(),
        runtime_digest: runtime.runtime_digest.clone(),
        protocol_version: config.protocol_version,
        repository_schema_version: config.repository_schema_version,
        required_repository_schema_version: cockpit_protocol::REPOSITORY_SCHEMA_VERSION,
        state: state.into(),
        safe_actions,
    })
}

pub fn migration_plan(root: &Path) -> Result<MigrationPlan, ObserverError> {
    migration_plan_with_runtime(root, None)
}

/// Build a migration plan with an optional executing Runtime identity.  The
/// schema migration decision remains unchanged; the additional inventory
/// makes immutable older finalization records discoverable without treating
/// them as current evidence.
pub fn migration_plan_with_runtime(
    root: &Path,
    runtime: Option<&RuntimeContext>,
) -> Result<MigrationPlan, ObserverError> {
    let (config, _, _, _) = migration_inputs(root)?;
    let target = cockpit_protocol::REPOSITORY_SCHEMA_VERSION;
    let (state, migration_type, planned_changes, steps) =
        if config.repository_schema_version == target {
            ("COMPATIBLE", "none", Vec::new(), Vec::new())
        } else if config.repository_schema_version < target {
            let steps = repository_schema_migration_chain(config.repository_schema_version, target)
                .map_err(|error| ObserverError::State {
                    path: root.join(".ai/cockpit.toml"),
                    message: error.to_string(),
                })?;
            let mut planned_changes = vec![
                ".ai/cockpit.toml".into(),
                ".ai/project.json".into(),
                ".ai/agent-interface.json".into(),
            ];
            planned_changes.extend(steps.iter().map(|step| {
                format!(
                    ".ai/migrations/<timestamp>-schema-{}-to-{}.json",
                    step.from_schema, step.to_schema
                )
            }));
            (
                "MIGRATION_REQUIRED",
                "adjacent_chain",
                planned_changes,
                steps,
            )
        } else {
            ("INCOMPATIBLE", "unsupported", Vec::new(), Vec::new())
        };
    let historical_finalization = historical_finalization_inventory(root, runtime)?;
    Ok(MigrationPlan {
        state: state.into(),
        current_schema: config.repository_schema_version,
        target_schema: target,
        migration_type: migration_type.into(),
        planned_changes,
        unchanged: vec![
            ".ai/work-items/archive".into(),
            ".ai/evidence".into(),
            ".ai/decisions".into(),
            ".ai/knowledge".into(),
            "historical Work Item records".into(),
        ],
        human_approval_required: state == "MIGRATION_REQUIRED",
        steps,
        historical_finalization,
    })
}

pub fn apply_migration(
    root: &Path,
    runtime: &RuntimeContext,
) -> Result<MigrationReceipt, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let (config, mut profile, mut manifest, before) = migration_inputs(&root)?;
    let target = cockpit_protocol::REPOSITORY_SCHEMA_VERSION;
    if config.repository_schema_version >= target {
        return Err(ObserverError::State {
            path: root.join(".ai/cockpit.toml"),
            message: "repository is already migrated or uses an unsupported future schema".into(),
        });
    }
    let from_schema = config.repository_schema_version;
    let chain = repository_schema_migration_chain(from_schema, target).map_err(|error| {
        ObserverError::State {
            path: root.join(".ai/cockpit.toml"),
            message: error.to_string(),
        }
    })?;
    let step = chain.first().cloned().ok_or_else(|| ObserverError::State {
        path: root.join(".ai/cockpit.toml"),
        message: "migration chain has no next step".into(),
    })?;
    if step.from_schema != from_schema || step.to_schema > target {
        return Err(ObserverError::State {
            path: root.join(".ai/cockpit.toml"),
            message: "migration step is not an adjacent reviewed edge".into(),
        });
    }
    let preserved_before = preserved_evidence_digest(&root)?;
    let migration_id = format!(
        "schema-{}-to-{}-{}",
        step.from_schema,
        step.to_schema,
        now().replace([':', '.'], "-")
    );
    profile.repository_schema_version = step.to_schema;
    manifest.repository_schema_version = step.to_schema;
    let ai = root.join(".ai");
    let config_text = format!(
        "protocol_version = {}\nrepository_schema_version = {}\nrepository_id = \"{}\"\n",
        config.protocol_version, step.to_schema, config.repository_id
    );
    let project_value = serde_json::to_value(&profile).map_err(|error| ObserverError::State {
        path: ai.join("project.json"),
        message: error.to_string(),
    })?;
    let manifest_value = serde_json::to_value(&manifest).map_err(|error| ObserverError::State {
        path: ai.join("agent-interface.json"),
        message: error.to_string(),
    })?;
    atomic_write(&ai.join("cockpit.toml"), config_text.as_bytes())?;
    atomic_json(&ai.join("project.json"), &project_value)?;
    atomic_json(&ai.join("agent-interface.json"), &manifest_value)?;
    let mut after = config_text.into_bytes();
    after.push(0);
    after.extend_from_slice(&serde_json::to_vec_pretty(&project_value).map_err(|error| {
        ObserverError::State {
            path: ai.join("project.json"),
            message: error.to_string(),
        }
    })?);
    after.push(0);
    after.extend_from_slice(
        &serde_json::to_vec_pretty(&manifest_value).map_err(|error| ObserverError::State {
            path: ai.join("agent-interface.json"),
            message: error.to_string(),
        })?,
    );
    let preserved_after = preserved_evidence_digest(&root)?;
    if preserved_before != preserved_after {
        return Err(ObserverError::State {
            path: root.join(".ai"),
            message: "historical evidence changed while applying migration".into(),
        });
    }
    let receipt = MigrationReceipt {
        schema_version: 1,
        migration_id: migration_id.clone(),
        from_schema,
        to_schema: step.to_schema,
        runtime_version: runtime.runtime_version.clone(),
        runtime_digest: runtime.runtime_digest.clone(),
        before_digest: Digest::sha256_bytes(&before),
        after_digest: Digest::sha256_bytes(&after),
        changes: vec![
            ".ai/cockpit.toml".into(),
            ".ai/project.json".into(),
            ".ai/agent-interface.json".into(),
        ],
        result: "completed".into(),
        created_at: now(),
        step,
        chain_length: chain.len(),
        preserved_evidence_digest: preserved_after,
        preserved_paths: MIGRATION_PRESERVED_PATHS
            .iter()
            .map(|path| (*path).into())
            .collect(),
    };
    let migrations = ai.join("migrations");
    fs::create_dir_all(&migrations).map_err(|source| ObserverError::Read {
        path: migrations.clone(),
        source,
    })?;
    let receipt_value = serde_json::to_value(&receipt).map_err(|error| ObserverError::State {
        path: migrations.clone(),
        message: error.to_string(),
    })?;
    atomic_json(
        &migrations.join(format!("{migration_id}.json")),
        &receipt_value,
    )?;
    Ok(receipt)
}

fn validate_start_entry(
    root: &Path,
    reject_unclosed_archives: bool,
    allow_recovery_base_drift: bool,
) -> Result<(), ObserverError> {
    let readiness = repository_readiness(root)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let mut failures = Vec::new();
    if reject_unclosed_archives && !readiness.unclosed_archived_work_items.is_empty() {
        failures.push(format!(
            "archived Work Items pending close: {}",
            readiness.unclosed_archived_work_items.join(", ")
        ));
    }
    if !readiness.dirty_paths.is_empty() {
        failures.push(format!(
            "non-governance changes were present before start: {}",
            readiness.dirty_paths.join(", ")
        ));
    }
    if readiness.current_branch.is_none() {
        failures.push("start requires a named branch; HEAD is detached".into());
    }
    if !allow_recovery_base_drift
        && readiness
            .blockers
            .iter()
            .any(|blocker| blocker == "base_revision_not_synchronized")
    {
        let default = readiness
            .default_remote
            .as_deref()
            .zip(readiness.default_branch.as_deref())
            .map(|(remote, branch)| format!("{remote}/{branch}"))
            .unwrap_or_else(|| "the discovered remote default".into());
        failures.push(format!(
            "branch HEAD does not equal the discovered base {default}; create a fresh branch from that base before start"
        ));
    }

    // A Work Item must never bind the repository's primary checkout or its
    // default branch.  The primary checkout is where the synchronized base
    // lives; using it for implementation makes finalization unable to prove
    // that the Work Item worktree was removed.  Requiring a discovered
    // default base for linked worktrees also prevents an ambiguous topology
    // from being treated as a green dedicated checkout.  Repositories used
    // only for local calibration (no remote and no linked worktree) retain the
    // existing `status=unknown` behavior until they are attached to a known
    // provider base.
    let layout = discover_worktree_layout(&root)?;
    let is_primary = layout.primary == root;
    if is_primary
        && (layout.paths.len() > 1
            || readiness.current_branch.as_deref() == readiness.default_branch.as_deref())
    {
        failures.push(format!(
            "work item must use a dedicated linked worktree; primary repository worktree is reserved for the synchronized default branch ({})",
            layout.primary.display()
        ));
    } else if !is_primary
        && (readiness.default_remote.is_none()
            || readiness.default_branch.is_none()
            || readiness.default_revision.is_none())
    {
        failures.push(
            "cannot authorize a linked worktree without an unambiguous discovered remote default base"
                .into(),
        );
    }
    if failures.is_empty() {
        return Ok(());
    }
    failures.sort();
    Err(ObserverError::State {
        path: root.clone(),
        message: format!(
            "lifecycle entry rejected before start: {}",
            failures.join("; ")
        ),
    })
}

fn recovery_scaffold_exists(root: &Path, work_item_id: &str) -> bool {
    let Some(root) = fs::canonicalize(root).ok() else {
        return false;
    };
    let active = root.join(".ai/work-items/active");
    let contract = read_json(&active.join(format!("{work_item_id}.contract.json"))).ok();
    let summary = read_json(&active.join(format!("{work_item_id}.summary.json"))).ok();
    contract
        .as_ref()
        .is_some_and(|value| value["state"] == serde_json::json!("not_ready"))
        && summary
            .as_ref()
            .is_some_and(|value| value["state"] == serde_json::json!("not_ready"))
        && contract
            .as_ref()
            .is_some_and(|value| value["predecessorWorkItemId"].is_string())
}

fn ensure_no_unclosed_archived_work_items(root: &Path) -> Result<(), ObserverError> {
    let pending = unclosed_archived_work_items(root)?;
    if pending.is_empty() {
        return Ok(());
    }
    Err(ObserverError::State {
        path: PathBuf::from(root).join(".ai/work-items/archive"),
        message: format!(
            "lifecycle entry rejected before start: archived Work Items pending close: {}",
            pending.join(", ")
        ),
    })
}

fn count_suffix(path: &Path, suffix: &str) -> usize {
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(suffix))
        .count()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ActiveArtifactVariant {
    name: String,
}

/// Recognize only Runtime-owned failed-attempt projections.  The canonical
/// `outcome.json`/`events.jsonl` artifacts have their own lifecycle path; a
/// dotted suffix denotes an immutable attempt or historical projection.
fn active_artifact_variant_name(name: &str) -> Option<(&str, &'static str)> {
    for (marker, extension, kind) in [
        (".outcome.", ".json", "outcome"),
        (".events.", ".jsonl", "events"),
    ] {
        let Some(index) = name.find(marker) else {
            continue;
        };
        let work_item_id = &name[..index];
        let variant = &name[index + marker.len()..];
        if work_item_id.is_empty()
            || variant.is_empty()
            || !variant.ends_with(extension)
            || validate_work_item_id(work_item_id).is_err()
        {
            continue;
        }
        return Some((work_item_id, kind));
    }
    None
}

fn active_artifact_variants(active: &Path) -> Result<Vec<ActiveArtifactVariant>, ObserverError> {
    let mut variants = Vec::new();
    let entries = match fs::read_dir(active) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(variants),
        Err(source) => {
            return Err(ObserverError::Read {
                path: active.to_path_buf(),
                source,
            });
        }
    };
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: active.to_path_buf(),
            source,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some((_, _)) = active_artifact_variant_name(&name) else {
            continue;
        };
        // Keep symlinks visible for status, but lifecycle moves must reject
        // them through `optional_regular_artifact` rather than following a
        // potentially foreign target.
        variants.push(ActiveArtifactVariant { name });
    }
    variants.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(variants)
}

fn orphaned_active_artifact_names(root: &Path) -> Result<Vec<String>, ObserverError> {
    let active = root.join(".ai/work-items/active");
    let variants = active_artifact_variants(&active)?;
    let mut orphaned = variants
        .into_iter()
        .filter(|variant| {
            let Some((work_item_id, _)) = active_artifact_variant_name(&variant.name) else {
                return false;
            };
            !is_regular_non_symlink(&active.join(format!("{work_item_id}.contract.json")))
                .unwrap_or(false)
        })
        .map(|variant| variant.name)
        .collect::<Vec<_>>();
    orphaned.sort();
    orphaned.dedup();
    Ok(orphaned)
}

pub fn set_evidence_retention_policy(
    root: &Path,
    work_item_id: &str,
    retention: EvidenceRetention,
    runtime: &RuntimeContext,
) -> Result<EvidenceRetentionPolicy, ObserverError> {
    validate_work_item_id(work_item_id)?;
    validate_evidence_retention(&retention).map_err(|error| ObserverError::State {
        path: root
            .join(".ai/evidence")
            .join(format!("{work_item_id}.retention.json")),
        message: error.to_string(),
    })?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    let policy = EvidenceRetentionPolicy {
        schema_version: 1,
        repository_id: contract.repository_id,
        work_item_id: work_item_id.into(),
        retention,
        created_at: now(),
    };
    let path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.retention.json"));
    if path.exists() {
        let existing = read_evidence_retention_policy(&root, work_item_id)?.ok_or_else(|| {
            ObserverError::State {
                path: path.clone(),
                message: "retention policy disappeared while reading".into(),
            }
        })?;
        if existing.repository_id != policy.repository_id
            || existing.work_item_id != policy.work_item_id
            || existing.retention != policy.retention
        {
            return Err(ObserverError::State {
                path,
                message: "conflicting retention policy already exists".into(),
            });
        }
        return Ok(existing);
    }
    let policy_value = serde_json::to_value(&policy).map_err(|error| ObserverError::State {
        path: path.clone(),
        message: error.to_string(),
    })?;
    atomic_json(&path, &policy_value)?;
    let _ = runtime;
    Ok(policy)
}

pub fn read_evidence_retention_policy(
    root: &Path,
    work_item_id: &str,
) -> Result<Option<EvidenceRetentionPolicy>, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.retention.json"));
    let value = match read_json(&path) {
        Ok(value) => value,
        Err(ObserverError::Read { source, .. })
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    let policy: EvidenceRetentionPolicy =
        serde_json::from_value(value).map_err(|error| ObserverError::State {
            path: path.clone(),
            message: error.to_string(),
        })?;
    validate_retention_policy_binding(
        &policy,
        &repository_id(&root).to_string(),
        work_item_id,
        &path,
    )?;
    validate_evidence_retention(&policy.retention).map_err(|error| ObserverError::State {
        path: path.clone(),
        message: error.to_string(),
    })?;
    if DateTime::parse_from_rfc3339(&policy.created_at).is_err()
        || policy
            .retention
            .expires_at
            .as_deref()
            .is_some_and(|value| !valid_retention_expiry(value))
    {
        return Err(ObserverError::State {
            path,
            message: "retention policy timestamps must be RFC3339".into(),
        });
    }
    Ok(Some(policy))
}

fn validate_retention_policy_binding(
    policy: &EvidenceRetentionPolicy,
    expected_repository_id: &str,
    expected_work_item_id: &str,
    path: &Path,
) -> Result<(), ObserverError> {
    if policy.schema_version != 1 {
        return Err(ObserverError::State {
            path: path.to_path_buf(),
            message: format!(
                "unsupported retention policy schemaVersion {}; expected 1",
                policy.schema_version
            ),
        });
    }
    if policy.repository_id != expected_repository_id
        || policy.work_item_id != expected_work_item_id
    {
        return Err(ObserverError::State {
            path: path.to_path_buf(),
            message: "retention policy repository/work item binding mismatch".into(),
        });
    }
    Ok(())
}

fn valid_retention_expiry(value: &str) -> bool {
    DateTime::parse_from_rfc3339(value).is_ok() || parse_epoch_seconds(value).is_some()
}

pub fn evidence_purge_plan(
    root: &Path,
    now_epoch_seconds: u64,
) -> Result<Vec<EvidenceDispositionItem>, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let evidence_dir = root.join(".ai/evidence");
    let mut items = Vec::new();
    let entries = match fs::read_dir(&evidence_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(items),
        Err(source) => {
            return Err(ObserverError::Read {
                path: evidence_dir,
                source,
            });
        }
    };
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: root.join(".ai/evidence"),
            source,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(work_item_id) = name.strip_suffix(".verification.json") else {
            continue;
        };
        let path = entry.path();
        let bytes = fs::read(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        let digest = Digest::sha256_bytes(&bytes);
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
                path: path.clone(),
                message: error.to_string(),
            })?;
        let policy = read_evidence_retention_policy(&root, work_item_id)?;
        let Some(policy) = policy else {
            items.push(EvidenceDispositionItem {
                path: format!(".ai/evidence/{name}"),
                digest,
                classification: DataClassification::Internal,
                persistence: EvidencePersistence::FullCapture,
                disposition: EvidenceDisposition::Retain,
                reason: "no repository retention policy is bound; no automatic disposal".into(),
            });
            continue;
        };
        let expired = policy
            .retention
            .expires_at
            .as_deref()
            .and_then(parse_epoch_seconds)
            .is_some_and(|expiry| expiry <= now_epoch_seconds)
            || policy.retention.retention_days.is_some_and(|days| {
                value["createdAt"]
                    .as_str()
                    .and_then(parse_epoch_seconds)
                    .is_some_and(|created| {
                        created.saturating_add(days.saturating_mul(86_400)) <= now_epoch_seconds
                    })
            });
        items.push(EvidenceDispositionItem {
            path: format!(".ai/evidence/{name}"),
            digest,
            classification: policy.retention.classification,
            persistence: policy.retention.persistence,
            disposition: if expired {
                EvidenceDisposition::PurgePlanned
            } else {
                EvidenceDisposition::Retain
            },
            reason: if expired {
                format!(
                    "retention expired; explicit disposal action={}",
                    policy.retention.disposal_action
                )
            } else {
                "retention window is still active".into()
            },
        });
    }
    items.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(items)
}

/// Build a deterministic, repository-bound audit export. The export is a
/// handoff artifact: local Git/.ai storage is not claimed to be immutable
/// enterprise retention.
pub fn export_audit_events(
    root: &Path,
    runtime: &RuntimeContext,
) -> Result<AuditExportManifest, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let repository_id = repository_id(&root).to_string();
    let mut events = Vec::new();
    let evidence_dir = root.join(".ai/evidence");
    if let Ok(entries) = fs::read_dir(&evidence_dir) {
        for entry in entries {
            let entry = entry.map_err(|source| ObserverError::Read {
                path: evidence_dir.clone(),
                source,
            })?;
            if !entry
                .file_type()
                .map(|kind| kind.is_file())
                .unwrap_or(false)
            {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            let bytes = fs::read(&path).map_err(|source| ObserverError::Read {
                path: path.clone(),
                source,
            })?;
            if let Some(work_item_id) = name.strip_suffix(".verification.json") {
                let value: serde_json::Value =
                    serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
                        path: path.clone(),
                        message: error.to_string(),
                    })?;
                let timestamp = value["createdAt"].as_str().unwrap_or("unknown");
                events.push(stable_audit_event(
                    &repository_id,
                    runtime,
                    Some(work_item_id),
                    "verification_recorded",
                    timestamp,
                    Digest::sha256_bytes(&bytes),
                    vec![format!(".ai/evidence/{name}")],
                )?);
            }
        }
    }
    let external_dir = evidence_dir.join("external");
    if let Ok(entries) = fs::read_dir(&external_dir) {
        for entry in entries {
            let entry = entry.map_err(|source| ObserverError::Read {
                path: external_dir.clone(),
                source,
            })?;
            if !entry
                .file_type()
                .map(|kind| kind.is_file())
                .unwrap_or(false)
            {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(work_item_id) = name.strip_suffix(".delegated.json") else {
                continue;
            };
            let bytes = fs::read(entry.path()).map_err(|source| ObserverError::Read {
                path: entry.path(),
                source,
            })?;
            let receipt: DelegatedEvidenceReceipt =
                serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
                    path: external_dir.join(&name),
                    message: error.to_string(),
                })?;
            if receipt.repository_id != repository_id || receipt.work_item_id != work_item_id {
                return Err(ObserverError::State {
                    path: external_dir.join(&name),
                    message: "audit export found a cross-repository delegated receipt".into(),
                });
            }
            events.push(stable_audit_event(
                &repository_id,
                runtime,
                Some(work_item_id),
                "external_evidence_bound",
                &receipt.bound_at,
                receipt.evidence.digest.clone(),
                vec![
                    format!(".ai/evidence/external/{name}"),
                    receipt.evidence.raw_evidence_ref,
                ],
            )?);
        }
    }
    let decisions_dir = root.join(".ai/decisions");
    if let Ok(entries) = fs::read_dir(&decisions_dir) {
        for entry in entries {
            let entry = entry.map_err(|source| ObserverError::Read {
                path: decisions_dir.clone(),
                source,
            })?;
            if !entry
                .file_type()
                .map(|kind| kind.is_file())
                .unwrap_or(false)
            {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(work_item_id) = name.strip_suffix(".close.json") else {
                continue;
            };
            let path = entry.path();
            let bytes = fs::read(&path).map_err(|source| ObserverError::Read {
                path: path.clone(),
                source,
            })?;
            let value: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
                    path: path.clone(),
                    message: error.to_string(),
                })?;
            let timestamp = value["structuredDecision"]["decidedAt"]
                .as_str()
                .unwrap_or("unknown");
            let refs = value["structuredDecision"]["evidenceRefs"]
                .as_array()
                .map(|refs| {
                    refs.iter()
                        .filter_map(|reference| reference.as_str().map(str::to_owned))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            events.push(stable_audit_event(
                &repository_id,
                runtime,
                Some(work_item_id),
                "human_decision_recorded",
                timestamp,
                Digest::sha256_bytes(&bytes),
                refs,
            )?);
        }
    }
    events.sort_by(|left, right| left.event_id.cmp(&right.event_id));
    let export_digest = cockpit_protocol::digest_json(&serde_json::json!({
        "repositoryId": repository_id,
        "runtimeVersion": &runtime.runtime_version,
        "runtimeDigest": &runtime.runtime_digest,
        "events": events,
    }))
    .map_err(|error| ObserverError::State {
        path: root.join(".ai"),
        message: error.to_string(),
    })?;
    Ok(AuditExportManifest {
        schema_version: 1,
        repository_id,
        runtime_version: runtime.runtime_version.clone(),
        runtime_digest: runtime.runtime_digest.clone(),
        export_digest,
        external_retention_required: true,
        events,
    })
}

fn stable_audit_event(
    repository_id: &str,
    runtime: &RuntimeContext,
    work_item_id: Option<&str>,
    event_type: &str,
    timestamp: &str,
    digest: Digest,
    evidence_refs: Vec<String>,
) -> Result<AuditEvent, ObserverError> {
    let payload = serde_json::json!({
        "repositoryId": repository_id,
        "runtimeVersion": &runtime.runtime_version,
        "runtimeDigest": &runtime.runtime_digest,
        "workItemId": work_item_id,
        "eventType": event_type,
        "timestamp": timestamp,
        "digest": digest,
        "evidenceRefs": evidence_refs,
    });
    let event_digest =
        cockpit_protocol::digest_json(&payload).map_err(|error| ObserverError::State {
            path: PathBuf::from(".ai/audit"),
            message: error.to_string(),
        })?;
    Ok(AuditEvent {
        event_id: event_digest.to_string(),
        repository_id: repository_id.into(),
        work_item_id: work_item_id.map(str::to_owned),
        runtime_version: runtime.runtime_version.clone(),
        runtime_digest: runtime.runtime_digest.clone(),
        timestamp: timestamp.into(),
        event_type: event_type.into(),
        digest,
        evidence_refs,
    })
}

fn parse_epoch_seconds(value: &str) -> Option<u64> {
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(seconds);
    }
    DateTime::parse_from_rfc3339(value)
        .ok()
        .and_then(|timestamp| timestamp.with_timezone(&Utc).timestamp().try_into().ok())
}

fn external_evidence_directory(root: &Path) -> Result<PathBuf, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let ai = root.join(".ai");
    let evidence = ai.join("evidence");
    let external = evidence.join("external");
    for directory in [&ai, &evidence, &external] {
        match fs::symlink_metadata(directory) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(ObserverError::State {
                    path: directory.to_path_buf(),
                    message: "external evidence parent must not be a symlink".into(),
                });
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(ObserverError::State {
                    path: directory.to_path_buf(),
                    message: "external evidence parent is not a directory".into(),
                });
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(directory).map_err(|source| ObserverError::Read {
                    path: directory.to_path_buf(),
                    source,
                })?;
            }
            Err(source) => {
                return Err(ObserverError::Read {
                    path: directory.to_path_buf(),
                    source,
                });
            }
        }
    }
    Ok(external)
}

fn existing_external_evidence_directory(root: &Path) -> Result<Option<PathBuf>, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let parents = [
        root.join(".ai"),
        root.join(".ai/evidence"),
        root.join(".ai/evidence/external"),
    ];
    for parent in &parents {
        match fs::symlink_metadata(parent) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(ObserverError::State {
                    path: parent.clone(),
                    message: "external evidence parent must not be a symlink".into(),
                });
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(ObserverError::State {
                    path: parent.clone(),
                    message: "external evidence parent is not a directory".into(),
                });
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(ObserverError::Read {
                    path: parent.clone(),
                    source,
                });
            }
        }
    }
    Ok(Some(parents[2].clone()))
}

fn external_evidence_file(root: &Path, reference: &str) -> Result<PathBuf, ObserverError> {
    let Some(name) = reference.strip_prefix(".ai/evidence/external/") else {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence/external"),
            message: "rawEvidenceRef must stay under .ai/evidence/external".into(),
        });
    };
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
    {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence/external"),
            message: "rawEvidenceRef must be one safe repository-relative filename".into(),
        });
    }
    let external = external_evidence_directory(root)?;
    let path = external.join(name);
    if let Ok(metadata) = fs::symlink_metadata(&path)
        && metadata.file_type().is_symlink()
    {
        return Err(ObserverError::State {
            path,
            message: "raw evidence leaf must not be a symlink".into(),
        });
    }
    Ok(path)
}

fn delegated_receipt_path(
    root: &Path,
    work_item_id: &str,
    digest: &Digest,
) -> Result<PathBuf, ObserverError> {
    let external = external_evidence_directory(root)?;
    let digest_hex =
        digest
            .as_str()
            .strip_prefix("sha256:")
            .ok_or_else(|| ObserverError::State {
                path: external.clone(),
                message: "delegated evidence digest must be sha256".into(),
            })?;
    Ok(external.join(format!("{work_item_id}.{digest_hex}.delegated.json")))
}

fn validate_delegated_metadata(
    root: &Path,
    work_item_id: &str,
    evidence: &DelegatedEvidence,
    raw_bytes: &[u8],
) -> Result<PathBuf, ObserverError> {
    if raw_bytes.len() > MAX_EXTERNAL_EVIDENCE_BYTES {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence/external"),
            message: "delegated raw evidence exceeds the bounded size limit".into(),
        });
    }
    for (field, value) in [
        ("provider", evidence.provider.as_str()),
        ("subject", evidence.subject.as_str()),
        ("origin", evidence.origin.as_str()),
        ("collectedAt", evidence.collected_at.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(ObserverError::State {
                path: root.join(".ai/evidence/external"),
                message: format!("delegated evidence field {field} must not be empty"),
            });
        }
    }
    let raw_path = external_evidence_file(root, &evidence.raw_evidence_ref)?;
    let computed = Digest::sha256_bytes(raw_bytes);
    if computed != evidence.digest {
        return Err(ObserverError::State {
            path: raw_path,
            message: format!(
                "delegated evidence digest mismatch: declared {}, computed {}",
                evidence.digest, computed
            ),
        });
    }
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let contract_path = if contract_path.is_file() {
        contract_path
    } else {
        root.join(".ai/work-items/archive")
            .join(format!("{work_item_id}.contract.json"))
    };
    let contract = read_contract(&contract_path)?;
    if contract.work_item_id != work_item_id
        || contract.repository_id != repository_id(root).to_string()
    {
        return Err(ObserverError::State {
            path: contract_path,
            message: "delegated evidence Work Item or repository binding mismatch".into(),
        });
    }
    Ok(raw_path)
}

/// Import provider-produced bytes and bind their digest to a repository Work
/// Item. Existing identical bytes/receipts are idempotent; conflicting writes
/// fail closed.
pub fn import_delegated_evidence(
    root: &Path,
    work_item_id: &str,
    evidence: &DelegatedEvidence,
    raw_bytes: &[u8],
    runtime: &RuntimeContext,
) -> Result<DelegatedEvidenceReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let raw_path = validate_delegated_metadata(&root, work_item_id, evidence, raw_bytes)?;
    match fs::symlink_metadata(&raw_path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(ObserverError::State {
                path: raw_path,
                message: "raw evidence leaf must not be a symlink".into(),
            });
        }
        Ok(_) => {
            let metadata =
                fs::symlink_metadata(&raw_path).map_err(|source| ObserverError::Read {
                    path: raw_path.clone(),
                    source,
                })?;
            if metadata.len() > MAX_EXTERNAL_EVIDENCE_BYTES as u64 {
                return Err(ObserverError::State {
                    path: raw_path,
                    message: "existing delegated raw evidence exceeds the bounded size limit"
                        .into(),
                });
            }
            let existing = fs::read(&raw_path).map_err(|source| ObserverError::Read {
                path: raw_path.clone(),
                source,
            })?;
            if existing != raw_bytes {
                return Err(ObserverError::State {
                    path: raw_path,
                    message: "raw evidence already exists with different bytes".into(),
                });
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            atomic_write(&raw_path, raw_bytes)?;
        }
        Err(source) => {
            return Err(ObserverError::Read {
                path: raw_path,
                source,
            });
        }
    }
    let receipt = DelegatedEvidenceReceipt {
        schema_version: 1,
        repository_id: repository_id(&root).to_string(),
        work_item_id: work_item_id.into(),
        evidence: evidence.clone(),
        runtime_version: runtime.runtime_version.clone(),
        runtime_digest: runtime.runtime_digest.clone(),
        bound_at: now(),
    };
    let receipt_path = delegated_receipt_path(&root, work_item_id, &evidence.digest)?;
    if let Ok(metadata) = fs::symlink_metadata(&receipt_path) {
        if metadata.file_type().is_symlink() {
            return Err(ObserverError::State {
                path: receipt_path,
                message: "delegated receipt leaf must not be a symlink".into(),
            });
        }
        if metadata.len() > MAX_REUSABLE_RECEIPT_BYTES {
            return Err(ObserverError::State {
                path: receipt_path,
                message: "existing delegated receipt exceeds the bounded size limit".into(),
            });
        }
        let existing: DelegatedEvidenceReceipt =
            serde_json::from_slice(&fs::read(&receipt_path).map_err(|source| {
                ObserverError::Read {
                    path: receipt_path.clone(),
                    source,
                }
            })?)
            .map_err(|error| ObserverError::State {
                path: receipt_path.clone(),
                message: format!("invalid existing delegated receipt: {error}"),
            })?;
        if existing.repository_id != receipt.repository_id
            || existing.work_item_id != receipt.work_item_id
            || existing.evidence != receipt.evidence
        {
            return Err(ObserverError::State {
                path: receipt_path,
                message: "delegated receipt already exists with different binding".into(),
            });
        }
        if existing.schema_version != 1 {
            return Err(ObserverError::State {
                path: receipt_path,
                message: "unsupported delegated receipt schema".into(),
            });
        }
        return Ok(existing);
    }
    let value = serde_json::to_value(&receipt).map_err(|error| ObserverError::State {
        path: receipt_path.clone(),
        message: error.to_string(),
    })?;
    atomic_json(&receipt_path, &value)?;
    Ok(receipt)
}

/// Read and revalidate all delegated receipts for a Work Item. Invalid or
/// mismatched entries are errors rather than silently becoming authority.
pub fn list_delegated_evidence(
    root: &Path,
    work_item_id: &str,
) -> Result<Vec<DelegatedEvidenceReceipt>, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let Some(external) = existing_external_evidence_directory(root)? else {
        return Ok(Vec::new());
    };
    let mut receipts = Vec::new();
    let entries = fs::read_dir(&external).map_err(|source| ObserverError::Read {
        path: external.clone(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: external.clone(),
            source,
        })?;
        let path = entry.path();
        if !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".delegated.json"))
        {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        if metadata.file_type().is_symlink() || metadata.len() > MAX_REUSABLE_RECEIPT_BYTES {
            return Err(ObserverError::State {
                path,
                message: "delegated receipt is symlinked or exceeds the bounded size limit".into(),
            });
        }
        let receipt: DelegatedEvidenceReceipt =
            serde_json::from_slice(&fs::read(&path).map_err(|source| ObserverError::Read {
                path: path.clone(),
                source,
            })?)
            .map_err(|error| ObserverError::State {
                path: path.clone(),
                message: format!("invalid delegated receipt: {error}"),
            })?;
        if receipt.schema_version != 1 {
            return Err(ObserverError::State {
                path,
                message: "unsupported delegated receipt schema".into(),
            });
        }
        if receipt.repository_id != repository_id(root).to_string() {
            return Err(ObserverError::State {
                path,
                message: "delegated receipt repository binding mismatch".into(),
            });
        }
        if receipt.work_item_id != work_item_id {
            continue;
        }
        let raw_path = external_evidence_file(root, &receipt.evidence.raw_evidence_ref)?;
        let raw_metadata =
            fs::symlink_metadata(&raw_path).map_err(|source| ObserverError::Read {
                path: raw_path.clone(),
                source,
            })?;
        if raw_metadata.len() > MAX_EXTERNAL_EVIDENCE_BYTES as u64 {
            return Err(ObserverError::State {
                path: raw_path,
                message: "delegated raw evidence exceeds the bounded size limit".into(),
            });
        }
        let raw = fs::read(&raw_path).map_err(|source| ObserverError::Read {
            path: raw_path.clone(),
            source,
        })?;
        validate_delegated_metadata(root, work_item_id, &receipt.evidence, &raw)?;
        receipts.push(receipt);
    }
    receipts.sort_by(|left, right| {
        left.evidence
            .provider
            .cmp(&right.evidence.provider)
            .then(left.evidence.subject.cmp(&right.evidence.subject))
            .then(
                left.evidence
                    .digest
                    .as_str()
                    .cmp(right.evidence.digest.as_str()),
            )
    });
    Ok(receipts)
}

fn delegated_evidence_satisfies(
    root: &Path,
    work_item_id: &str,
    requirement: &str,
) -> Result<bool, ObserverError> {
    let provider = requirement.strip_prefix("delegated:");
    if provider.is_none() && !matches!(requirement, "delegated_evidence" | "external_evidence") {
        return Ok(false);
    }
    Ok(list_delegated_evidence(root, work_item_id)?
        .into_iter()
        .any(|receipt| {
            receipt.evidence.validity == EvidenceValidity::Valid
                && provider.is_none_or(|provider| receipt.evidence.provider == provider)
        }))
}

fn source_tree_digest(snapshot: &RepositorySnapshot) -> Result<Digest, ObserverError> {
    // A governance-only edit does not change the source tree, so the captured
    // index digest is already the stable post-commit value.  When source paths
    // are dirty, rebuild the index representation with the current worktree
    // blobs; this makes the digest equal before and after committing the same
    // source change while preserving the existing digest wire shape.
    let source_changed = snapshot
        .changed_paths
        .iter()
        .any(|path| path != ".ai" && !path.starts_with(".ai/"));
    if !source_changed && let Some(captured) = &snapshot.source_tree_digest {
        return captured.parse().map_err(|_| ObserverError::State {
            path: snapshot.git_root.clone(),
            message: "captured source tree digest is invalid".into(),
        });
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(&snapshot.git_root)
        .args(["ls-files", "-s"])
        .output()
        .map_err(|source| ObserverError::Read {
            path: snapshot.git_root.clone(),
            source,
        })?;
    if !output.status.success() {
        return Err(ObserverError::State {
            path: snapshot.git_root.clone(),
            message: "cannot enumerate the repository source tree".into(),
        });
    }
    let mut entries = BTreeMap::<String, (String, String)>::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let Some((metadata, path)) = line.split_once('\t') else {
            continue;
        };
        let Some(path) = stable_source_path(path) else {
            continue;
        };
        let mut fields = metadata.split_whitespace();
        let (Some(mode), Some(object_id)) = (fields.next(), fields.next()) else {
            continue;
        };
        entries.insert(path, (mode.to_owned(), object_id.to_owned()));
    }

    if source_changed {
        let status = Command::new("git")
            .arg("-C")
            .arg(&snapshot.git_root)
            .args(["status", "--short", "--untracked-files=all"])
            .output()
            .map_err(|source| ObserverError::Read {
                path: snapshot.git_root.clone(),
                source,
            })?;
        if !status.status.success() {
            return Err(ObserverError::State {
                path: snapshot.git_root.clone(),
                message: "cannot inspect the source working tree".into(),
            });
        }
        let mut changes = BTreeMap::<String, bool>::new();
        for line in String::from_utf8_lossy(&status.stdout).lines() {
            let Some(raw_path) = line.get(3..) else {
                continue;
            };
            let code = line.get(..2).unwrap_or("  ");
            if code.contains('R')
                && let Some((source, target)) = raw_path.rsplit_once(" -> ")
            {
                if let Some(source) = stable_source_path(source) {
                    changes.insert(source, false);
                }
                if let Some(target) = stable_source_path(target) {
                    changes.insert(target, true);
                }
                continue;
            }
            if let Some(path) = stable_source_path(raw_path) {
                changes.insert(path, !code.contains('D'));
            }
        }
        let paths = changes
            .iter()
            .filter_map(|(path, present)| {
                if !present {
                    return None;
                }
                let candidate = snapshot.git_root.join(path);
                let metadata = fs::symlink_metadata(candidate).ok()?;
                (metadata.file_type().is_file() || metadata.file_type().is_symlink())
                    .then_some(path.clone())
            })
            .collect::<Vec<_>>();
        let hashes = git_hash_object_paths(&snapshot.git_root, &paths)?;
        for (path, present) in changes {
            if !present {
                entries.remove(&path);
                continue;
            }
            let Some(object_id) = hashes.get(&path) else {
                entries.remove(&path);
                continue;
            };
            entries.insert(
                path.clone(),
                (
                    worktree_mode(&snapshot.git_root.join(path)),
                    object_id.clone(),
                ),
            );
        }
    }

    let mut hasher = Sha256::new();
    for (path, (mode, object_id)) in entries {
        hasher.update(path.as_bytes());
        hasher.update([0]);
        hasher.update(format!("{mode} {object_id} 0\t{path}").as_bytes());
        hasher.update([0]);
    }
    Ok(Digest::sha256_bytes(&hasher.finalize()))
}

fn stable_source_path(raw: &str) -> Option<String> {
    let path = raw.trim().trim_matches('"').replace('\\', "/");
    if path == ".ai" || path.starts_with(".ai/") || path.is_empty() {
        return None;
    }
    let candidate = Path::new(&path);
    if candidate.is_absolute()
        || candidate
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return None;
    }
    Some(path)
}

fn git_hash_object_paths(
    root: &Path,
    paths: &[String],
) -> Result<BTreeMap<String, String>, ObserverError> {
    if paths.is_empty() {
        return Ok(BTreeMap::new());
    }
    let mut child = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["hash-object", "--no-filters", "--stdin-paths"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| ObserverError::Read {
            path: root.to_path_buf(),
            source,
        })?;
    {
        let Some(stdin) = child.stdin.as_mut() else {
            return Err(ObserverError::State {
                path: root.to_path_buf(),
                message: "cannot open git hash-object input".into(),
            });
        };
        for path in paths {
            stdin
                .write_all(path.as_bytes())
                .and_then(|_| stdin.write_all(b"\n"))
                .map_err(|source| ObserverError::Read {
                    path: root.to_path_buf(),
                    source,
                })?;
        }
    }
    let output = child
        .wait_with_output()
        .map_err(|source| ObserverError::Read {
            path: root.to_path_buf(),
            source,
        })?;
    if !output.status.success() {
        return Err(ObserverError::State {
            path: root.to_path_buf(),
            message: String::from_utf8_lossy(&output.stderr).trim().into(),
        });
    }
    let hashes = String::from_utf8_lossy(&output.stdout);
    Ok(paths
        .iter()
        .zip(hashes.lines())
        .map(|(path, hash)| (path.clone(), hash.to_owned()))
        .collect())
}

fn worktree_mode(path: &Path) -> String {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return "000000".into();
    };
    if metadata.file_type().is_symlink() {
        return "120000".into();
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 != 0 {
            return "100755".into();
        }
    }
    "100644".into()
}

/// Digest the repository facts that govern source verification, not the
/// governance records used to produce that verification.  In particular,
/// committing `.ai/` receipts after a successful verification must not make
/// the source evidence stale; a source commit or non-`.ai` working-tree change
/// must still invalidate it.  Absolute worktree paths and Git HEAD are also
/// excluded so local and hosted PR contexts share the same identity.
pub fn snapshot_digest(snapshot: &RepositorySnapshot) -> Result<Digest, ObserverError> {
    let mut stable = snapshot.clone();
    stable.root = PathBuf::from(".");
    stable.git_root = PathBuf::from(".");
    stable.head = None;
    // Git subprocess count is request telemetry, not repository identity.
    // Clean snapshots may skip the redundant diff call while dirty snapshots
    // still execute it; that implementation detail must not stale a source
    // digest or make governance-only changes look like source changes.
    stable.git_calls = 0;
    stable.tree_digest = source_tree_digest(snapshot)?.to_string();
    // Diff shape and read/measurement counters describe the current checkout
    // representation, not the source content.  Exclude them so verification
    // remains valid after the same worktree content is committed or exposed
    // through a hosted synthetic merge ref.
    stable.diff_digest =
        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into();
    stable.files_read = 0;
    stable.files_hashed = 0;
    stable.changed_paths.clear();
    stable
        .change_evidence
        .retain(|change| !change.path.starts_with(".ai/"));
    cockpit_protocol::digest_json(&serde_json::json!({
        "repositoryId": repository_id(&snapshot.root),
        "sourceSnapshot": stable,
    }))
    .map_err(|error| ObserverError::State {
        path: snapshot.root.join(".ai"),
        message: error.to_string(),
    })
}

fn policy_document(root: &Path) -> Result<Option<GovernancePolicyDocument>, ObserverError> {
    let path = root.join(".ai/policy.json");
    if !path.is_file() {
        return Ok(None);
    }
    let value = read_json(&path)?;
    let document: GovernancePolicyDocument =
        serde_json::from_value(value).map_err(|error| ObserverError::State {
            path: path.clone(),
            message: format!("invalid governance policy: {error}"),
        })?;
    if document.schema_version != 1 {
        return Err(ObserverError::State {
            path,
            message: format!(
                "unsupported governance policy schema {}",
                document.schema_version
            ),
        });
    }
    if document.organization.is_none() && document.project.is_none() {
        return Err(ObserverError::State {
            path,
            message: "governance policy must define organization or project policy".into(),
        });
    }
    if document
        .organization
        .as_ref()
        .is_some_and(|policy| !matches!(policy.layer, PolicyLayer::Organization))
        || document
            .project
            .as_ref()
            .is_some_and(|policy| !matches!(policy.layer, PolicyLayer::Project))
    {
        return Err(ObserverError::State {
            path,
            message: "governance policy layer does not match its document slot".into(),
        });
    }
    Ok(Some(document))
}

/// Return the effective repository + Work Item policy, if the repository has
/// opted into policy enforcement. Policy bytes remain repository-local and
/// are never inferred from natural-language requests.
pub fn effective_policy_for_contract(
    root: &Path,
    contract: &Contract,
) -> Result<Option<GovernancePolicy>, ObserverError> {
    let Some(document) = policy_document(root)? else {
        return Ok(None);
    };
    let mut layers = Vec::new();
    if let Some(organization) = document.organization.as_ref() {
        layers.push(organization);
    }
    if let Some(project) = document.project.as_ref() {
        layers.push(project);
    }
    if let Some(work_item) = contract.governance_policy.as_ref() {
        if !matches!(work_item.layer, PolicyLayer::WorkItem) {
            return Err(ObserverError::State {
                path: root.join(".ai/work-items/active"),
                message: "Work Item governance policy must use layer=work_item".into(),
            });
        }
        layers.push(work_item);
    }
    merge_policy_layers(&layers)
        .map(Some)
        .map_err(|error| ObserverError::State {
            path: root.join(".ai/policy.json"),
            message: error.to_string(),
        })
}

/// Resolve the policy-bound verification route for one Work Item.  This is a
/// request-scoped projection: it reads only the active Contract, the
/// repository-local policy, and the supplied snapshot.  No policy means the
/// historical route remains available with no invented requirement.
pub fn resolve_verification_route(
    root: &Path,
    work_item_id: &str,
    stage: VerificationStage,
    runner: &str,
    snapshot: &RepositorySnapshot,
) -> Result<VerificationRoute, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    if fs::canonicalize(&snapshot.root).ok().as_ref() != Some(&root) {
        return Err(ObserverError::SnapshotRootMismatch);
    }
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    let operation = verification_operation_for_contract(&contract).to_owned();
    let base_revision = if stage.requires_base_revision() {
        if !valid_git_object_id(&contract.base_revision) {
            return Err(ObserverError::State {
                path: contract_path.clone(),
                message: format!(
                    "verification stage {} requires a valid Contract baseRevision",
                    stage.as_str()
                ),
            });
        }
        Some(contract.base_revision.clone())
    } else {
        valid_git_object_id(&contract.base_revision).then(|| contract.base_revision.clone())
    };
    let policy_plan = if let Some(policy) = effective_policy_for_contract(&root, &contract)? {
        // Existing policy files may govern approval/evidence without opting
        // into the typed verification route. Preserve that no-requirement
        // compatibility lane; a declared requirement is always planned and
        // validated fail-closed.
        let has_requirement = policy
            .rules
            .iter()
            .find(|rule| rule.operation == operation)
            .and_then(|rule| rule.verification_requirement.as_ref())
            .is_some();
        if has_requirement {
            Some(
                cockpit_verification::plan_policy_requirement(
                    &cockpit_verification::PolicyPlannerInput {
                        operation: operation.clone(),
                        stage: stage.as_str().into(),
                        protected_gate: None,
                        policies: vec![policy],
                    },
                )
                .map_err(|error| ObserverError::State {
                    path: root.join(".ai/policy.json"),
                    message: error.to_string(),
                })?,
            )
        } else {
            None
        }
    } else {
        None
    };
    if let Some(plan) = policy_plan.as_ref() {
        let coverage = contract
            .scenario_coverage
            .as_ref()
            .and_then(|value| cockpit_protocol::validate_scenario_coverage_projection(value).ok())
            .unwrap_or_default();
        let scenarios = coverage
            .iter()
            .map(|entry| entry.scenario.clone())
            .collect::<Vec<_>>();
        let required_scenarios = coverage
            .iter()
            .filter(|entry| entry.required)
            .map(|entry| entry.scenario.clone())
            .collect::<Vec<_>>();
        let intent = if contract.intent.is_empty() {
            String::new()
        } else {
            "contract-intent-present".into()
        };
        cockpit_verification::bind_intent_scenario_route(
            &cockpit_verification::IntentScenarioRouteInput {
                intent,
                scenarios,
                required_scenarios,
                operation: operation.clone(),
                stage: stage.as_str().into(),
                high_risk: contract.risk.to_ascii_lowercase().contains("high"),
                policy_plan: plan.clone(),
            },
        )
        .map_err(|error| ObserverError::State {
            path: root.join(".ai/policy.json"),
            message: format!("intent/scenario verification route is not bound: {error}"),
        })?;
    }
    let actual_tier = match stage {
        VerificationStage::Task => VerificationTier::T0,
        VerificationStage::PreCi => VerificationTier::T1,
        VerificationStage::PullRequest | VerificationStage::Merge | VerificationStage::Release => {
            VerificationTier::T2
        }
    };
    let actual_assurance = if runner == "hosted" {
        EvidenceAssurance::ProviderVerified
    } else {
        EvidenceAssurance::RepositoryVerified
    };
    if let Some(plan) = &policy_plan
        && !plan
            .requirement
            .is_satisfied_by(actual_tier, actual_assurance)
    {
        return Err(ObserverError::State {
            path: root.join(".ai/policy.json"),
            message: format!(
                "verification requirement is not satisfied: required tier {:?}/assurance {:?}, actual {:?}/{:?}",
                plan.requirement.required_tier,
                plan.requirement.required_assurance,
                actual_tier,
                actual_assurance
            ),
        });
    }
    let mut affected_paths = snapshot
        .changed_paths
        .iter()
        .map(|path| path.replace('\\', "/"))
        .filter(|path| path != ".ai" && !path.starts_with(".ai/"))
        .collect::<Vec<_>>();
    affected_paths.sort();
    affected_paths.dedup();
    Ok(VerificationRoute {
        work_item_id: work_item_id.into(),
        operation,
        stage,
        policy_plan,
        actual_tier,
        actual_assurance,
        base_revision,
        affected_paths,
        // The repository observer does not infer a dependency graph from
        // changed paths.  Unknown is the truthful, conservative projection.
        dependency_confidence: cockpit_verification::DependencyConfidence::Unknown,
    })
}

/// Evaluate one active Contract for CI without recording preflight,
/// verification, checkpoint, or decision evidence.  CI may use this source
/// build projection before executing repository commands; lifecycle commands
/// remain the authority for mutable `.ai/` evidence.
pub fn evaluate_contract_quality_gate(
    root: &Path,
    contract_path: &Path,
    stage: VerificationStage,
    runner: &str,
    expected_base_revision: Option<&str>,
    runtime: &RuntimeContext,
) -> Result<ContractQualityGateReport, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let candidate = if contract_path.is_absolute() {
        contract_path.to_path_buf()
    } else {
        root.join(contract_path)
    };
    let metadata = fs::symlink_metadata(&candidate).map_err(|source| ObserverError::Read {
        path: candidate.clone(),
        source,
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(ObserverError::State {
            path: candidate,
            message: "Contract path must be a regular non-symlink file".into(),
        });
    }
    let contract_path = fs::canonicalize(&candidate).map_err(|source| ObserverError::Read {
        path: candidate.clone(),
        source,
    })?;
    contract_path
        .strip_prefix(&root)
        .map_err(|_| ObserverError::State {
            path: contract_path.clone(),
            message: "Contract path escapes repository".into(),
        })?;
    let contract = read_contract(&contract_path)?;
    if contract.work_item_id.trim().is_empty() {
        return Err(ObserverError::State {
            path: contract_path,
            message: "Contract workItemId is required for the CI gate".into(),
        });
    }
    let expected_repository_id = repository_id(&root);
    if contract.repository_id != expected_repository_id.to_string() {
        return Err(ObserverError::State {
            path: contract_path,
            message: "Contract repositoryId does not match the repository context".into(),
        });
    }
    if let Some(expected) = expected_base_revision
        && contract.base_revision != expected
    {
        return Err(ObserverError::State {
            path: contract_path,
            message: "Contract baseRevision does not match the CI base revision".into(),
        });
    }
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    if fs::canonicalize(&snapshot.root).ok().as_ref() != Some(&root) {
        return Err(ObserverError::SnapshotRootMismatch);
    }
    let current_snapshot_digest = snapshot_digest(&snapshot)?;
    let current_contract_digest = contract_digest(&contract_path)?;
    let contract_file_digest = Digest::sha256_bytes(&fs::read(&contract_path).map_err(
        |source| ObserverError::Read {
            path: contract_path.clone(),
            source,
        },
    )?);
    let route =
        resolve_verification_route(&root, &contract.work_item_id, stage, runner, &snapshot)?;
    let mut blockers = contract_freshness_findings(&root, &contract, &snapshot)?;
    let decision = governance_decision_for_contract(&root, &contract, &snapshot)?;
    blockers.extend(decision.blockers.clone());
    blockers.sort();
    blockers.dedup();
    let mut unknowns = decision.unknowns.clone();
    unknowns.sort();
    unknowns.dedup();
    let mut required_checks = decision.required_checks.clone();
    required_checks.sort();
    required_checks.dedup();
    let decision_state = decision_state_name(decision.state.clone()).to_string();
    let mut report = ContractQualityGateReport {
        schema_version: 1,
        kind: "repository_contract_quality_gate".into(),
        state: if decision.state == DecisionState::Green && blockers.is_empty() {
            "passed".into()
        } else {
            "blocked".into()
        },
        repository_id: expected_repository_id,
        work_item_id: contract.work_item_id.clone(),
        contract_digest: current_contract_digest,
        contract_file_digest,
        repository_snapshot_digest: current_snapshot_digest,
        base_revision: contract.base_revision.clone(),
        head_revision: snapshot.head.clone(),
        changed_paths: route.affected_paths.clone(),
        stage: stage.as_str().into(),
        runner: runner.into(),
        operation: route.operation.clone(),
        verification_tier: route.actual_tier,
        evidence_assurance: route.actual_assurance,
        dependency_confidence: route.dependency_confidence,
        decision_state,
        blockers,
        unknowns,
        required_checks,
        runtime_version: runtime.runtime_version.clone(),
        runtime_digest: runtime.runtime_digest.clone(),
        receipt_digest: Digest::sha256_bytes(b"pending"),
    };
    let mut payload = serde_json::to_value(&report).map_err(|error| ObserverError::State {
        path: root.join(".ai/work-items/active"),
        message: error.to_string(),
    })?;
    payload
        .as_object_mut()
        .expect("ContractQualityGateReport serializes as an object")
        .remove("receiptDigest");
    report.receipt_digest =
        cockpit_protocol::digest_json(&payload).map_err(|error| ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: error.to_string(),
        })?;
    Ok(report)
}

pub fn verification_operation_for_contract(contract: &cockpit_protocol::Contract) -> &str {
    contract.operation.as_deref().unwrap_or_else(|| {
        if contract.risk.to_ascii_lowercase().contains("destructive") {
            "production_destructive"
        } else {
            "modify_source"
        }
    })
}

fn contract_policy_rule<'a>(
    contract: &Contract,
    policy: &'a GovernancePolicy,
) -> Option<&'a cockpit_protocol::PolicyRule> {
    let operation = verification_operation_for_contract(contract);
    policy.rules.iter().find(|rule| rule.operation == operation)
}

fn apply_policy_to_governance_input(
    contract: &Contract,
    policy: Option<&GovernancePolicy>,
    input: &mut GovernanceInput,
) {
    let Some(policy) = policy else {
        return;
    };
    let Some(rule) = contract_policy_rule(contract, policy) else {
        return;
    };
    if rule
        .required_evidence
        .iter()
        .any(|required| !contract.required_evidence_classes.contains(required))
    {
        input
            .explicit_unknowns
            .push("policy_required_evidence_missing".into());
    }
    match rule.approval_mode {
        ApprovalMode::NoHumanApprovalForLowRisk => {}
        ApprovalMode::SingleAuthorizedHuman => {
            if input.authority != AuthorityState::Authorized {
                input
                    .explicit_unknowns
                    .push("human_authority_missing".into());
            }
        }
        ApprovalMode::MultiPartyApproval | ApprovalMode::ExternalProviderApproval => {
            input
                .explicit_unknowns
                .push("policy_approval_receipt_missing".into());
        }
    }
}

/// Verification may collect the evidence required by a policy, but it must
/// not run when the policy already says the actor lacks authority or when the
/// selected approval mode requires an unimplemented external receipt.
pub fn require_policy_for_verification(
    root: &Path,
    work_item_id: &str,
) -> Result<(), ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    let Some(policy) = effective_policy_for_contract(&root, &contract)? else {
        return Ok(());
    };
    let Some(rule) = contract_policy_rule(&contract, &policy) else {
        return Ok(());
    };
    match rule.approval_mode {
        ApprovalMode::NoHumanApprovalForLowRisk => Ok(()),
        ApprovalMode::SingleAuthorizedHuman if contract.authority == "authorized" => Ok(()),
        ApprovalMode::SingleAuthorizedHuman => Err(ObserverError::State {
            path: contract_path,
            message: "policy requires an authorized human before verification".into(),
        }),
        ApprovalMode::MultiPartyApproval | ApprovalMode::ExternalProviderApproval => {
            Err(ObserverError::State {
                path: contract_path,
                message: format!(
                    "policy approval mode {:?} requires an external approval receipt",
                    rule.approval_mode
                ),
            })
        }
    }
}

fn is_test_path(path: &str) -> bool {
    let normalized = path.to_ascii_lowercase();
    normalized.starts_with("tests/")
        || normalized.starts_with("test/")
        || normalized.contains("/tests/")
        || normalized.contains("/test/")
        || normalized.contains("/spec/")
        || normalized
            .rsplit('/')
            .next()
            .is_some_and(|name| name.contains("test") || name.contains("spec"))
}

fn is_coverage_path(path: &str) -> bool {
    let normalized = path.to_ascii_lowercase();
    let name = normalized.rsplit('/').next().unwrap_or(&normalized);
    matches!(
        name,
        ".coveragerc"
            | "coverage.json"
            | "coverage.yaml"
            | "coverage.yml"
            | "pyproject.toml"
            | "tox.ini"
            | "package.json"
    ) || name.starts_with("jest.config.")
}

fn is_textual_material_path(path: &str) -> bool {
    let normalized = path.to_ascii_lowercase();
    [
        ".md", ".txt", ".json", ".toml", ".yaml", ".yml", ".rs", ".py", ".js", ".ts", ".tsx",
        ".jsx", ".java", ".kt", ".swift", ".go", ".sh",
    ]
    .iter()
    .any(|extension| normalized.ends_with(extension))
}

/// The reference inventory is a large machine-generated conformance ledger.
/// Its own strict checker is the authority for its schema and path coverage;
/// treating an oversized JSON diff as source prose would otherwise make every
/// legitimate rebaseline permanently yellow.  This exemption is deliberately
/// exact and does not weaken inspection for other test or repository files.
fn is_strictly_checked_conformance_manifest(path: &str) -> bool {
    path == "tests/conformance/reference_file_inventory.json"
}

fn contains_strong_instruction_injection(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    let instruction = [
        "ignore previous instructions",
        "ignore all previous instructions",
        "ignore the contract",
        "override policy",
        "bypass policy",
        "disable governance",
        "system message",
    ]
    .iter()
    .any(|pattern| text.contains(pattern));
    let risky_operation = [
        "delete",
        "rm -rf",
        "execute",
        "run ",
        "curl ",
        "secret",
        "token",
        "upload",
        "exfil",
        "disable test",
        "skip test",
        "publish",
        "push main",
    ]
    .iter()
    .any(|pattern| text.contains(pattern));
    instruction && risky_operation
}

fn contains_skip_marker(lines: &[String]) -> bool {
    lines.iter().any(|line| {
        let line = line.to_ascii_lowercase();
        [
            "pytest.mark.skip",
            ".skip(",
            "#[ignore]",
            "@disabled",
            "@ignore",
            "disabled_",
            "xit(",
            "xdescribe(",
        ]
        .iter()
        .any(|marker| line.contains(marker))
    })
}

fn assertion_count(lines: &[String]) -> usize {
    lines
        .iter()
        .filter(|line| {
            let line = line.to_ascii_lowercase();
            ["assert", "expect(", "xctassert", "tothrow("]
                .iter()
                .any(|marker| line.contains(marker))
        })
        .count()
}

fn contains_test_bypass(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    let removes_tests = text.lines().any(|line| {
        let line = line.trim();
        (line.contains("delete")
            || line.contains("remove")
            || line.contains("disable")
            || line.contains("skip"))
            && line.contains("test")
    });
    let claims_success = text
        .lines()
        .any(|line| line.contains("pass") || line.contains("green") || line.contains("ci"));
    removes_tests && claims_success
        || text.lines().any(|line| {
            line.contains("continue-on-error: true")
                || line.contains("allow_failure: true")
                || line.contains("|| true")
        })
}

fn assignment(line: &str) -> Option<(String, String)> {
    let (name, value) = line.split_once('=').or_else(|| line.split_once(':'))?;
    Some((
        name.trim().to_ascii_lowercase(),
        value.trim().trim_matches(['\'', '"']).to_ascii_lowercase(),
    ))
}

fn first_number(value: &str) -> Option<f64> {
    let start = value.find(|character: char| character.is_ascii_digit())?;
    let number = value[start..]
        .chars()
        .take_while(|character| character.is_ascii_digit() || *character == '.')
        .collect::<String>();
    number.parse().ok()
}

fn coverage_weakened(removed: &[String], added: &[String]) -> bool {
    let removed = removed
        .iter()
        .filter_map(|line| assignment(line))
        .collect::<Vec<_>>();
    let added = added
        .iter()
        .filter_map(|line| assignment(line))
        .collect::<Vec<_>>();
    for (name, before) in &removed {
        if matches!(name.as_str(), "fail_under" | "threshold" | "minimum")
            && let Some((_, after)) = added.iter().find(|(candidate, _)| candidate == name)
            && let (Some(before), Some(after)) = (first_number(before), first_number(after))
            && after < before
        {
            return true;
        }
        if matches!(name.as_str(), "source" | "source_pkgs")
            && let Some((_, after)) = added.iter().find(|(candidate, _)| candidate == name)
        {
            let before = before
                .split([',', ' '])
                .filter(|item| !item.is_empty())
                .count();
            let after = after
                .split([',', ' '])
                .filter(|item| !item.is_empty())
                .count();
            if after < before {
                return true;
            }
        }
    }
    added.iter().any(|(name, value)| {
        matches!(
            name.as_str(),
            "omit" | "exclude" | "exclude_lines" | "coveragepathignorepatterns"
        ) && !removed
            .iter()
            .any(|(old_name, old_value)| old_name == name && old_value == value)
    })
}

pub fn derive_governance_signals(snapshot: &RepositorySnapshot) -> GovernanceSignalAssessment {
    let mut result = GovernanceSignalAssessment::default();
    for change in &snapshot.change_evidence {
        if change.path.starts_with(".ai/") {
            continue;
        }
        let test_path = is_test_path(&change.path);
        let coverage_path = is_coverage_path(&change.path);
        let inspectable = matches!(
            change.content_state,
            ChangeContentState::Text | ChangeContentState::Deleted
        );
        if !inspectable {
            let strictly_checked_manifest = is_strictly_checked_conformance_manifest(&change.path);
            if test_path && !strictly_checked_manifest {
                result
                    .unknowns
                    .push("test_weakening_inspection_unavailable".into());
            }
            if coverage_path {
                result
                    .unknowns
                    .push("coverage_weakening_inspection_unavailable".into());
            }
            if is_textual_material_path(&change.path) && !strictly_checked_manifest {
                result
                    .unknowns
                    .push("repository_material_inspection_unavailable".into());
            }
            continue;
        }

        let added_text = change.added_lines.join("\n");
        // Untracked material has no Git patch lines yet, so inspect its bounded
        // current text. For tracked files, inspect only added lines; scanning
        // the whole source file would match detector examples embedded in the
        // Runtime itself and produce a false governance finding.
        let material_text = if change.added_lines.is_empty() && change.kind == ChangeKind::Added {
            change.after_text.as_deref().unwrap_or("")
        } else {
            &added_text
        };
        if contains_strong_instruction_injection(material_text) {
            result.untrusted_material = true;
            result.findings.push("repository_prompt_injection".into());
        }
        if test_path
            && (change.kind == ChangeKind::Deleted
                || contains_skip_marker(&change.added_lines)
                || assertion_count(&change.removed_lines) > assertion_count(&change.added_lines)
                || contains_test_bypass(&added_text))
        {
            result.test_weakening = true;
            result.findings.push("test_weakening".into());
        }
        if coverage_path && coverage_weakened(&change.removed_lines, &change.added_lines) {
            result.coverage_weakening = true;
            result.findings.push("coverage_weakening".into());
        }
    }
    result.unknowns.sort();
    result.unknowns.dedup();
    result.findings.sort();
    result.findings.dedup();
    result
}

pub fn contract_freshness_findings(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    _snapshot: &RepositorySnapshot,
) -> Result<Vec<String>, ObserverError> {
    contract_freshness_findings_with_identity(root, contract, &repository_id(root))
}

fn contract_freshness_findings_with_identity(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    expected_repository_id: &Digest,
) -> Result<Vec<String>, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let ai_path = root.join(".ai");
    let config_path = ai_path.join("cockpit.toml");
    let profile_path = ai_path.join("project.json");
    if !ai_path.is_dir() {
        return Ok(Vec::new());
    }
    let mut findings = Vec::new();
    if !config_path.is_file() || !profile_path.is_file() {
        findings.push("stale_contract".into());
        return Ok(findings);
    }
    if contract.repository_id != expected_repository_id.to_string() {
        findings.push("stale_contract".into());
    }
    let profile: AttachedProfile = read_json(&profile_path).and_then(|value| {
        serde_json::from_value(value).map_err(|error| ObserverError::State {
            path: profile_path.clone(),
            message: error.to_string(),
        })
    })?;
    let profile_digest = attached_profile_digest(&profile, &profile_path)?;
    if contract.project_profile_digest != profile_digest {
        findings.push("stale_contract".into());
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

pub fn governance_decision_for_contract(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
) -> Result<GovernanceDecision, ObserverError> {
    governance_decision_for_contract_internal(root, contract, snapshot, None)
}

/// Evaluate governance while binding verification evidence to the Runtime
/// executing the request.  This keeps preflight and lifecycle gates aligned:
/// a foreign Runtime receipt cannot make a current preflight green and then
/// fail only later at finish.
pub fn governance_decision_for_contract_with_runtime(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    runtime: &RuntimeContext,
) -> Result<GovernanceDecision, ObserverError> {
    governance_decision_for_contract_internal(root, contract, snapshot, Some(runtime))
}

/// Evaluate one governance decision from an explicitly captured observation
/// phase. The compatibility wrappers remain available for embedders, while
/// request-bound callers can prevent lower-level helpers from resolving a
/// second repository identity or snapshot digest.
pub fn governance_decision_for_observation_context(
    observation: &ObservationContext,
    contract: &cockpit_protocol::Contract,
) -> Result<GovernanceDecision, ObserverError> {
    observation.require_phase(ObservationPhase::BeforeGovernance)?;
    if let Some(expected_contract_digest) = observation.contract_model_digest() {
        let contract_value =
            serde_json::to_value(contract).map_err(|error| ObserverError::State {
                path: observation.root().join(".ai/work-items"),
                message: error.to_string(),
            })?;
        let actual_contract_digest =
            cockpit_protocol::digest_json(&contract_value).map_err(|error| {
                ObserverError::State {
                    path: observation.root().join(".ai/work-items"),
                    message: error.to_string(),
                }
            })?;
        if &actual_contract_digest != expected_contract_digest {
            return Err(ObserverError::State {
                path: observation.root().join(".ai/work-items"),
                message: "governance Contract does not match the captured observation identity"
                    .into(),
            });
        }
    }
    let decision = governance_decision_for_contract_base_internal_with_archive(
        observation.root(),
        contract,
        observation.snapshot(),
        observation.runtime(),
        false,
        Some(observation),
    )?;
    let decision = apply_preflight_review_evidence(
        observation.root(),
        contract,
        observation.snapshot(),
        decision,
        false,
    )?;
    observation.validate_current()?;
    Ok(decision)
}

fn governance_decision_for_contract_internal(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    current_runtime: Option<&RuntimeContext>,
) -> Result<GovernanceDecision, ObserverError> {
    governance_decision_for_contract_internal_with_archive(
        root,
        contract,
        snapshot,
        current_runtime,
        false,
    )
}

fn governance_decision_for_archived_contract_internal(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    current_runtime: Option<&RuntimeContext>,
) -> Result<GovernanceDecision, ObserverError> {
    // Archived evidence remains immutable historical truth.  A Runtime
    // upgrade must not turn an otherwise valid archived Contract red merely
    // because its receipt was produced by an older executable.  Active
    // Work Items still use the strict current-runtime binding below.
    let effective_runtime =
        if archived_evidence_is_historical(root, contract, snapshot, current_runtime)? {
            None
        } else {
            current_runtime
        };
    governance_decision_for_contract_internal_with_archive(
        root,
        contract,
        snapshot,
        effective_runtime,
        true,
    )
}

fn governance_decision_for_contract_internal_with_archive(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    current_runtime: Option<&RuntimeContext>,
    archived: bool,
) -> Result<GovernanceDecision, ObserverError> {
    let decision = governance_decision_for_contract_base_internal_with_archive(
        root,
        contract,
        snapshot,
        current_runtime,
        archived,
        None,
    )?;
    apply_preflight_review_evidence(root, contract, snapshot, decision, archived)
}

fn governance_decision_for_contract_base_internal_with_archive(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    current_runtime: Option<&RuntimeContext>,
    archived: bool,
    observation: Option<&ObservationContext>,
) -> Result<GovernanceDecision, ObserverError> {
    let expected_repository_id = observation
        .map(|context| context.repository_id())
        .cloned()
        .unwrap_or_else(|| repository_id(root));
    let explicit_blockers =
        contract_freshness_findings_with_identity(root, contract, &expected_repository_id)?;
    let signals = derive_governance_signals(snapshot);
    let changed_paths = snapshot
        .changed_paths
        .iter()
        .filter(|path| !path.starts_with(".ai/"))
        .cloned()
        .collect();
    let action = if contract.risk.to_ascii_lowercase().contains("destructive") {
        ActionKind::Destructive
    } else {
        ActionKind::Write
    };
    let authority = if contract.authority == "authorized" {
        AuthorityState::Authorized
    } else {
        AuthorityState::Missing
    };
    let evidence = evidence_state_for_contract_internal_with_archive(
        root,
        contract,
        snapshot,
        current_runtime,
        archived,
    )?;
    let mut explicit_unknowns = signals.unknowns;
    explicit_unknowns.extend(contract_review_unknowns(contract));
    let project_unknowns = observation.map_or_else(
        || project_governance_unknowns(root, contract, snapshot),
        |context| {
            Ok(project_governance::project_governance_unknowns_from_facts(
                contract,
                context.project_governance_facts(),
            ))
        },
    )?;
    explicit_unknowns.extend(project_unknowns);
    let contract_value = serde_json::to_value(contract).map_err(|error| ObserverError::State {
        path: root.join(".ai/work-items"),
        message: error.to_string(),
    })?;
    explicit_unknowns.extend(scenario_coverage_preflight_unknowns(&contract_value));
    let mut input = GovernanceInput {
        scope: contract.scope.clone(),
        out_of_scope: contract.out_of_scope.clone(),
        changed_paths,
        action,
        authority,
        evidence,
        untrusted_material: signals.untrusted_material,
        test_weakening: signals.test_weakening,
        coverage_weakening: signals.coverage_weakening,
        explicit_blockers,
        explicit_unknowns,
        outcome_state_override: None,
        authority_override: None,
    };
    let policy = effective_policy_for_contract(root, contract)?;
    apply_policy_to_governance_input(contract, policy.as_ref(), &mut input);
    Ok(evaluate(input))
}

fn apply_preflight_review_evidence(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    mut decision: GovernanceDecision,
    archived: bool,
) -> Result<GovernanceDecision, ObserverError> {
    if archived {
        return Ok(decision);
    }
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{}.contract.json", contract.work_item_id));
    // `preflight --contract` also supports a read-only standalone Contract
    // outside the active Work Item directory. Such a document has no place
    // to persist a decision receipt; retain the normal advisory decision and
    // do not manufacture an `active/.contract.json` lookup.
    if contract.work_item_id.trim().is_empty() || !contract_path.is_file() {
        return Ok(decision);
    }
    let contract_digest = contract_digest(&contract_path)?;
    // Digest the canonical JSON projection, not the Rust struct directly.
    // serde_json::Value is the wire representation stored in Summary and in
    // the human decision receipt; hashing two different serialization paths
    // would make a valid receipt appear stale immediately.
    let decision_value = serde_json::to_value(&decision).map_err(|error| ObserverError::State {
        path: contract_path.clone(),
        message: error.to_string(),
    })?;
    let raw_decision_digest =
        cockpit_protocol::digest_json(&decision_value).map_err(|error| ObserverError::State {
            path: contract_path.clone(),
            message: error.to_string(),
        })?;
    let current_snapshot_digest = snapshot_digest(snapshot)?;
    match preflight_decision_evidence_state(
        root,
        &contract.work_item_id,
        &contract_digest,
        &raw_decision_digest,
        &current_snapshot_digest,
    ) {
        governance_controls::PreflightDecisionEvidenceState::Missing => {}
        governance_controls::PreflightDecisionEvidenceState::Valid => {
            if decision.review_state.as_deref() == Some("needs_human_confirmation")
                && decision.blockers.is_empty()
            {
                decision.review_state = Some("human_decision_recorded".into());
                decision.outcome_state = "verification_pending".into();
                decision.safe_actions.push("continue_to_checkpoint".into());
                decision.safe_actions.sort();
                decision.safe_actions.dedup();
            }
        }
        governance_controls::PreflightDecisionEvidenceState::Invalid => {
            decision
                .unknowns
                .push("preflight_decision_evidence_invalid".into());
            decision.unknowns.sort();
            decision.unknowns.dedup();
            decision
                .safe_actions
                .push("record_fresh_preflight_decision".into());
            decision.safe_actions.sort();
            decision.safe_actions.dedup();
            decision.required_checks.push("human_review".into());
            decision.required_checks.sort();
            decision.required_checks.dedup();
            decision.state = DecisionState::Yellow;
            decision.outcome_state = "needs_human_decision".into();
            decision.review_state = Some("needs_human_confirmation".into());
            decision.human_decision_request = Some(cockpit_core::HumanDecisionRequest {
                decision_id: "contract-preflight-review".into(),
                status: "needs_human_confirmation".into(),
                what_happened: "The recorded preflight decision evidence is missing, stale, or invalid.".into(),
                why_it_matters: "A previous human review cannot authorize a changed or foreign Contract.".into(),
                options: vec![
                    cockpit_core::HumanDecisionOption {
                        id: "complete_contract".into(),
                        label: "Complete or amend the Contract".into(),
                        effect: "Provide current human-owned facts and rerun preflight.".into(),
                    },
                    cockpit_core::HumanDecisionOption {
                        id: "confirm_review".into(),
                        label: "Confirm a bounded human decision".into(),
                        effect: "Record a new identity-bound receipt for this exact Contract and snapshot.".into(),
                    },
                    cockpit_core::HumanDecisionOption {
                        id: "stop_work".into(),
                        label: "Stop the Work Item".into(),
                        effect: "Leave the item recoverable without entering implementation.".into(),
                    },
                ],
                recommended_option: "confirm_review".into(),
                recommendation_reason: "The previous receipt cannot be reused after its binding facts changed.".into(),
                question: "Which bounded decision should authorize the next step?".into(),
                resume_condition: "A fresh repository-bound preflight decision receipt matches the current Contract and snapshot.".into(),
            });
        }
    }
    Ok(decision)
}

/// Return only deterministic Contract-completeness gaps.  These are human
/// decisions, not facts the Observer is allowed to invent.  A scaffold (or a
/// Contract with missing authority) therefore remains yellow and must stop at
/// the pre-edit review boundary.
fn contract_review_unknowns(contract: &cockpit_protocol::Contract) -> Vec<String> {
    let mut unknowns = Vec::new();
    if contract.state.as_deref() == Some("not_ready") {
        if contract.intent.is_empty() {
            unknowns.push("contract_intent_missing".into());
        }
        if contract.goal.trim().is_empty() {
            unknowns.push("contract_goal_missing".into());
        }
        if contract.scope.is_empty() {
            unknowns.push("contract_scope_missing".into());
        }
        if contract.out_of_scope.is_empty() {
            unknowns.push("contract_out_of_scope_missing".into());
        }
        if contract.acceptance_criteria.is_empty() {
            unknowns.push("contract_acceptance_missing".into());
        }
        if !matches!(contract.authority.as_str(), "authorized") {
            unknowns.push("human_authority_missing".into());
        }
    }
    if contract.contract_version == Some(2) {
        match contract.intent.structured() {
            Some(intent) => {
                if intent
                    .problem
                    .as_deref()
                    .is_none_or(|value| value.trim().is_empty())
                {
                    unknowns.push("contract_intent_problem_missing".into());
                }
                if intent.constraints.is_empty() {
                    unknowns.push("contract_intent_constraints_missing".into());
                }
                if intent
                    .rationale
                    .as_deref()
                    .is_none_or(|value| value.trim().is_empty())
                {
                    unknowns.push("contract_intent_rationale_missing".into());
                }
            }
            None => unknowns.push("contract_intent_structured_required".into()),
        }
        if contract
            .problem_statement
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            unknowns.push("contract_problem_statement_missing".into());
        }
    }
    if contract.not_codable == Some(true) {
        unknowns.push("contract_not_codable".into());
    }
    unknowns.extend(
        contract
            .unknowns
            .iter()
            .map(|unknown| format!("contract_declared_unknown:{unknown}")),
    );
    if let Some(capability) = &contract.agent_capability {
        if !capability.can_implement {
            unknowns.push("agent_cannot_implement".into());
        }
        if !capability.can_verify {
            unknowns.push("agent_cannot_verify".into());
        }
        if capability.needs_human_decision {
            unknowns.push("agent_needs_human_decision".into());
        }
    }
    if let Some(decision) = &contract.execution_decision {
        if !matches!(
            decision.status.as_str(),
            "continue" | "defer" | "needs_human_decision" | "block"
        ) {
            unknowns.push("execution_decision_invalid".into());
        } else if decision.status != "continue" {
            unknowns.push(format!("execution_decision:{}", decision.status));
        }
    }
    unknowns
}

struct DuplicateKeySeed;

struct DuplicateKeyVisitor;

impl<'de> DeserializeSeed<'de> for DuplicateKeySeed {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(DuplicateKeyVisitor)
    }
}

impl<'de> Visitor<'de> for DuplicateKeyVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }

    fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(DuplicateKeyVisitor)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while sequence.next_element_seed(DuplicateKeySeed)?.is_some() {}
        Ok(())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut keys = std::collections::BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate JSON object key: {key}"
                )));
            }
            map.next_value_seed(DuplicateKeySeed)?;
        }
        Ok(())
    }
}

fn reject_duplicate_json_keys(bytes: &[u8]) -> Result<(), String> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    deserializer
        .deserialize_any(DuplicateKeyVisitor)
        .map_err(|error| error.to_string())?;
    deserializer.end().map_err(|error| error.to_string())
}

fn read_contract(path: &Path) -> Result<cockpit_protocol::Contract, ObserverError> {
    let bytes = fs::read(path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })?;
    reject_duplicate_json_keys(&bytes).map_err(|message| ObserverError::State {
        path: path.to_path_buf(),
        message: format!("invalid Contract JSON: {message}"),
    })?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
    let contract: cockpit_protocol::Contract =
        serde_json::from_value(value).map_err(|error| ObserverError::State {
            path: path.to_path_buf(),
            message: format!("invalid work item contract: {error}"),
        })?;
    contract.validate().map_err(|errors| ObserverError::State {
        path: path.to_path_buf(),
        message: format!(
            "invalid work item contract invariants: {}",
            errors.join("; ")
        ),
    })?;
    Ok(contract)
}

fn require_green_governance(
    root: &Path,
    contract_path: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    operation: &str,
) -> Result<(), ObserverError> {
    require_green_governance_internal(root, contract_path, contract, snapshot, operation, None)
}

fn require_green_governance_with_runtime(
    root: &Path,
    contract_path: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    operation: &str,
    runtime: &RuntimeContext,
) -> Result<(), ObserverError> {
    require_green_governance_internal(
        root,
        contract_path,
        contract,
        snapshot,
        operation,
        Some(runtime),
    )
}

fn require_green_governance_internal(
    root: &Path,
    contract_path: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    operation: &str,
    current_runtime: Option<&RuntimeContext>,
) -> Result<(), ObserverError> {
    require_green_governance_internal_with_archive(
        root,
        contract_path,
        contract,
        snapshot,
        operation,
        current_runtime,
        false,
    )
}

fn require_green_governance_for_archived_contract(
    root: &Path,
    contract_path: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    operation: &str,
    current_runtime: Option<&RuntimeContext>,
) -> Result<(), ObserverError> {
    require_green_governance_internal_with_archive(
        root,
        contract_path,
        contract,
        snapshot,
        operation,
        current_runtime,
        true,
    )
}

fn require_green_governance_internal_with_archive(
    root: &Path,
    contract_path: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    operation: &str,
    current_runtime: Option<&RuntimeContext>,
    archived: bool,
) -> Result<(), ObserverError> {
    let decision = if archived {
        governance_decision_for_archived_contract_internal(
            root,
            contract,
            snapshot,
            current_runtime,
        )?
    } else {
        governance_decision_for_contract_internal(root, contract, snapshot, current_runtime)?
    };
    if decision.state != DecisionState::Green {
        return Err(ObserverError::State {
            path: contract_path.to_path_buf(),
            message: format!(
                "{operation} requires a green governance decision (state={:?}, blockers={:?}, unknowns={:?})",
                decision.state, decision.blockers, decision.unknowns
            ),
        });
    }
    Ok(())
}

/// Validate the repository-local verification receipt at every lifecycle
/// boundary.  A file existing at the expected path is not evidence: the
/// receipt must be schema-versioned, passed, identity-bound, snapshot-bound,
/// and internally digest-consistent.
fn verification_evidence_state(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    archived: bool,
    current_runtime: Option<&RuntimeContext>,
) -> Result<EvidenceState, ObserverError> {
    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{}.verification.json", contract.work_item_id));
    let metadata = match fs::symlink_metadata(&evidence_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(EvidenceState::Missing);
        }
        Err(_) => return Ok(EvidenceState::Unknown),
    };
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Ok(EvidenceState::Contradictory);
    }
    let evidence = match read_json(&evidence_path) {
        Ok(value) => value,
        Err(_) => return Ok(EvidenceState::Unknown),
    };
    let envelope = match serde_json::from_value::<VerificationEvidenceEnvelope>(evidence.clone()) {
        Ok(value) => value,
        Err(_) => return Ok(EvidenceState::Contradictory),
    };
    if envelope.protocol_version != 1
        || envelope.evidence_schema_version != 2
        || envelope.work_item_id != contract.work_item_id
        || !envelope.passed
        || DateTime::parse_from_rfc3339(&envelope.created_at).is_err()
        || envelope.retention.as_ref().is_some_and(|policy| {
            DateTime::parse_from_rfc3339(&policy.created_at).is_err()
                || policy
                    .retention
                    .expires_at
                    .as_deref()
                    .is_some_and(|value| !valid_retention_expiry(value))
        })
        || envelope
            .runtime_digest
            .to_string()
            .parse::<Digest>()
            .is_err()
        || envelope
            .repository_snapshot_digest
            .to_string()
            .parse::<Digest>()
            .is_err()
        || envelope
            .receipt_digest
            .to_string()
            .parse::<Digest>()
            .is_err()
    {
        return Ok(EvidenceState::Contradictory);
    }

    let expected_repository_id = repository_id(root).to_string();
    if contract.repository_id != expected_repository_id
        || envelope.repository_id != expected_repository_id
    {
        return Ok(EvidenceState::Contradictory);
    }
    let expected_contract_digest = contract_digest_for_evidence(root, contract)?;
    // An explicit Contract amendment deliberately invalidates the previous
    // verification cycle.  Treat only that bound predecessor digest as
    // stale so a fresh verify can replace it; malformed, foreign, or
    // tampered evidence still remains contradictory and fail-closed.
    let amendment_invalidates_previous = !archived
        && contract_amendment_invalidates_verification(root, contract, &expected_contract_digest);
    if envelope.contract_digest.as_ref() != Some(&expected_contract_digest) {
        return Ok(if amendment_invalidates_previous {
            EvidenceState::Stale
        } else {
            EvidenceState::Contradictory
        });
    }

    if let Some(embedded_retention) = envelope.retention.as_ref() {
        let retention_path = root
            .join(".ai/evidence")
            .join(format!("{}.retention.json", contract.work_item_id));
        if validate_retention_policy_binding(
            embedded_retention,
            &expected_repository_id,
            &contract.work_item_id,
            &retention_path,
        )
        .is_err()
            || validate_evidence_retention(&embedded_retention.retention).is_err()
        {
            return Ok(EvidenceState::Contradictory);
        }
        match read_evidence_retention_policy(root, &contract.work_item_id) {
            Ok(Some(standalone_retention)) if standalone_retention == *embedded_retention => {}
            _ => return Ok(EvidenceState::Contradictory),
        }
    }

    if let Some(runtime) = current_runtime
        && (envelope.runtime_version != runtime.runtime_version
            || envelope.runtime_digest != runtime.runtime_digest)
    {
        return Ok(EvidenceState::Contradictory);
    }
    let current_snapshot_digest = snapshot_digest(snapshot)?;
    if !archived && envelope.repository_snapshot_digest != current_snapshot_digest {
        return Ok(EvidenceState::Stale);
    }

    if envelope.runtime_version.trim().is_empty() {
        return Ok(EvidenceState::Contradictory);
    }
    match envelope.capture_mode {
        VerificationCaptureMode::DigestOnly => {
            if envelope.receipt.is_some() {
                return Ok(EvidenceState::Contradictory);
            }
        }
        VerificationCaptureMode::FullCapture | VerificationCaptureMode::RedactedCapture => {
            let Some(receipt) = envelope.receipt.as_ref() else {
                return Ok(EvidenceState::Contradictory);
            };
            let typed: cockpit_verification::VerificationReceipt =
                match serde_json::from_value(receipt.clone()) {
                    Ok(value) => value,
                    Err(_) => return Ok(EvidenceState::Contradictory),
                };
            if !validate_plan_receipt_binding(
                root,
                contract,
                snapshot,
                &envelope,
                typed.plan_receipt.as_ref(),
                archived,
            )? {
                return Ok(EvidenceState::Contradictory);
            }
            if !typed.passed
                || typed.work_item_id.as_deref() != Some(contract.work_item_id.as_str())
                || typed.repository_id.as_deref() != Some(expected_repository_id.as_str())
                || typed.runtime_version.as_deref() != Some(envelope.runtime_version.as_str())
                || typed.runtime_digest.as_deref()
                    != Some(envelope.runtime_digest.to_string().as_str())
            {
                return Ok(EvidenceState::Contradictory);
            }
            let Ok(computed) = cockpit_protocol::digest_json(receipt) else {
                return Ok(EvidenceState::Contradictory);
            };
            if computed != envelope.receipt_digest {
                return Ok(EvidenceState::Contradictory);
            }
        }
        VerificationCaptureMode::LegacyUntyped => {
            // This compatibility lane is readable only through the legacy
            // Rust API.  A Runtime-bound CLI/MCP lifecycle must regenerate a
            // typed v2 receipt instead of treating the old payload as green.
            if current_runtime.is_some() {
                return Ok(EvidenceState::Contradictory);
            }
        }
    }

    if archived {
        let archive = root.join(".ai/work-items/archive");
        let outcome_path = archive.join(format!("{}.outcome.json", contract.work_item_id));
        let manifest_path = archive.join(format!("{}.archive.json", contract.work_item_id));
        let outcome = match read_json(&outcome_path) {
            Ok(value) => value,
            Err(_) => return Ok(EvidenceState::Contradictory),
        };
        let manifest = match read_json(&manifest_path) {
            Ok(value) => value,
            Err(_) => return Ok(EvidenceState::Contradictory),
        };
        let Ok(evidence_digest) = cockpit_protocol::digest_json(&evidence) else {
            return Ok(EvidenceState::Contradictory);
        };
        let outcome_bytes = match fs::read(&outcome_path) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(EvidenceState::Contradictory),
        };
        let outcome_file_digest = Digest::sha256_bytes(&outcome_bytes);
        if outcome["evidenceDigest"] != serde_json::json!(evidence_digest.to_string())
            || manifest["files"]["outcomeDigest"]
                != serde_json::json!(outcome_file_digest.to_string())
        {
            return Ok(EvidenceState::Contradictory);
        }
    }
    Ok(EvidenceState::Complete)
}

/// Return whether the active Summary records an explicit amendment for the
/// current Contract.  This is intentionally a narrow compatibility lane for
/// replacing a stale predecessor verification receipt; it never authorizes
/// malformed or identity-mismatched evidence.
fn contract_amendment_invalidates_verification(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    current_contract_digest: &Digest,
) -> bool {
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{}.summary.json", contract.work_item_id));
    let Ok(summary) = read_json(&summary_path) else {
        return false;
    };
    summary
        .get("verificationInvalidatedByContractAmendment")
        .and_then(|value| value.get("contractHash"))
        .and_then(serde_json::Value::as_str)
        .and_then(|value| value.parse::<Digest>().ok())
        .is_some_and(|digest| digest == *current_contract_digest)
}

/// Return true when an archived receipt is integrity-valid as historical
/// evidence but was produced by a different Runtime identity.  This is an
/// explicit compatibility lane: active Work Items never call it, and callers
/// must still require a fresh current Runtime receipt for any new operation
/// such as resource finalization.
fn archived_evidence_is_historical(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    current_runtime: Option<&RuntimeContext>,
) -> Result<bool, ObserverError> {
    let Some(current_runtime) = current_runtime else {
        return Ok(false);
    };
    if verification_evidence_state(root, contract, snapshot, true, None)? != EvidenceState::Complete
    {
        return Ok(false);
    }
    Ok(
        verification_evidence_state(root, contract, snapshot, true, Some(current_runtime))?
            != EvidenceState::Complete,
    )
}

/// Validate the policy route projection embedded in a typed verification
/// receipt.  Historical receipts without a route projection remain readable;
/// a receipt for a currently policy-routed Work Item must carry every binding
/// needed to prevent a weaker or foreign route from becoming lifecycle truth.
fn validate_plan_receipt_binding(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    envelope: &VerificationEvidenceEnvelope,
    plan: Option<&cockpit_verification::VerificationPlanReceipt>,
    archived: bool,
) -> Result<bool, ObserverError> {
    let Some(plan) = plan else {
        let has_policy_requirement = effective_policy_for_contract(root, contract)?
            .as_ref()
            .and_then(|policy| {
                policy
                    .rules
                    .iter()
                    .find(|rule| rule.operation == verification_operation_for_contract(contract))
            })
            .and_then(|rule| rule.verification_requirement.as_ref())
            .is_some();
        return Ok(!has_policy_requirement);
    };
    if plan.validate_monotonic().is_err() {
        return Ok(false);
    }
    let expected_repository_id = repository_id(root).to_string();
    let expected = effective_policy_requirement_for_contract(root, contract, plan.stage.as_str())?;
    let Some(requirement) = expected else {
        // A non-policy route may retain a historical unbound plan. Newer
        // routes may carry identity/fact bindings, but they must not smuggle
        // a policy requirement that the repository never declared.
        if plan.required_tier.is_some()
            || plan.required_assurance.is_some()
            || !plan.policy_refs.is_empty()
        {
            return Ok(false);
        }
        if plan.work_item_id.is_none()
            && plan.repository_id.is_none()
            && plan.repository_snapshot_digest.is_none()
        {
            return Ok(true);
        }
        let expected_snapshot = snapshot_digest(snapshot)?;
        let snapshot_matches_current = archived
            || plan.repository_snapshot_digest.as_deref()
                == Some(expected_snapshot.to_string().as_str());
        return Ok(
            plan.work_item_id.as_deref() == Some(contract.work_item_id.as_str())
                && plan.repository_id.as_deref() == Some(expected_repository_id.as_str())
                && snapshot_matches_current
                && plan.repository_snapshot_digest.as_deref()
                    == Some(envelope.repository_snapshot_digest.to_string().as_str()),
        );
    };
    if plan.work_item_id.as_deref() != Some(contract.work_item_id.as_str())
        || plan.repository_id.as_deref() != Some(expected_repository_id.as_str())
        || plan.repository_snapshot_digest.as_deref()
            != Some(envelope.repository_snapshot_digest.to_string().as_str())
    {
        return Ok(false);
    }
    let expected_snapshot = snapshot_digest(snapshot)?;
    if !archived
        && plan
            .repository_snapshot_digest
            .as_deref()
            .is_none_or(|digest| digest != expected_snapshot.to_string())
    {
        return Ok(false);
    }
    if plan.required_tier != Some(requirement.required_tier)
        || plan.required_assurance != Some(requirement.required_assurance)
        || plan.policy_refs != requirement.policy_refs
        || plan.dependency_confidence.is_none()
        || plan.base_revision.as_deref() != Some(contract.base_revision.as_str())
        || !requirement.is_satisfied_by(plan.final_tier, plan.assurance)
    {
        return Ok(false);
    }
    Ok(true)
}

fn effective_policy_requirement_for_contract(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    stage: &str,
) -> Result<Option<cockpit_protocol::VerificationRequirement>, ObserverError> {
    let Some(policy) = effective_policy_for_contract(root, contract)? else {
        return Ok(None);
    };
    let operation = verification_operation_for_contract(contract);
    let Some(rule) = policy.rules.iter().find(|rule| rule.operation == operation) else {
        return Ok(None);
    };
    if rule.verification_requirement.is_none() {
        return Ok(None);
    }
    let plan =
        cockpit_verification::plan_policy_requirement(&cockpit_verification::PolicyPlannerInput {
            operation: operation.into(),
            stage: stage.into(),
            protected_gate: None,
            policies: vec![policy],
        })
        .map_err(|error| ObserverError::State {
            path: root.join(".ai/policy.json"),
            message: error.to_string(),
        })?;
    Ok(Some(plan.requirement))
}

pub fn evidence_state_for_contract(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
) -> Result<EvidenceState, ObserverError> {
    evidence_state_for_contract_internal(root, contract, snapshot, None)
}

pub fn evidence_state_for_contract_with_runtime(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    runtime: &RuntimeContext,
) -> Result<EvidenceState, ObserverError> {
    evidence_state_for_contract_internal(root, contract, snapshot, Some(runtime))
}

fn evidence_state_for_contract_internal(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    current_runtime: Option<&RuntimeContext>,
) -> Result<EvidenceState, ObserverError> {
    evidence_state_for_contract_internal_with_archive(
        root,
        contract,
        snapshot,
        current_runtime,
        false,
    )
}

fn evidence_state_for_contract_internal_with_archive(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    current_runtime: Option<&RuntimeContext>,
    archived: bool,
) -> Result<EvidenceState, ObserverError> {
    if contract.required_evidence_classes.is_empty() {
        // Verification evidence is an integrity surface even when the
        // Contract did not declare it as a required class.  Preserve the
        // historical no-evidence behavior for a fresh Work Item, but never
        // let an existing tampered receipt be ignored by preflight/governance.
        let evidence_path = root
            .join(".ai/evidence")
            .join(format!("{}.verification.json", contract.work_item_id));
        if fs::symlink_metadata(&evidence_path).is_ok() {
            let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
                path: root.into(),
                source,
            })?;
            return verification_evidence_state(
                &root,
                contract,
                snapshot,
                archived,
                current_runtime,
            );
        }
        return Ok(EvidenceState::Complete);
    }
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let requires_verification = contract.required_evidence_classes.iter().any(|class| {
        matches!(
            class.to_ascii_lowercase().as_str(),
            "verification" | "verification_receipt" | "verification-receipt"
        )
    });
    if requires_verification {
        return verification_evidence_state(&root, contract, snapshot, archived, current_runtime);
    }
    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{}.verification.json", contract.work_item_id));
    if fs::symlink_metadata(&evidence_path).is_ok() {
        let state =
            verification_evidence_state(&root, contract, snapshot, archived, current_runtime)?;
        if state != EvidenceState::Complete {
            return Ok(state);
        }
    }
    for class in &contract.required_evidence_classes {
        let normalized = class.to_ascii_lowercase();
        if matches!(
            normalized.as_str(),
            "verification" | "verification_receipt" | "verification-receipt"
        ) {
            continue;
        }
        if normalized.starts_with("delegated:")
            || matches!(
                normalized.as_str(),
                "delegated_evidence" | "external_evidence"
            )
        {
            if !delegated_evidence_satisfies(&root, &contract.work_item_id, &normalized)? {
                return Ok(EvidenceState::Missing);
            }
            continue;
        }
        return Ok(EvidenceState::Missing);
    }
    Ok(EvidenceState::Complete)
}

pub fn archive_work_item(
    root: &Path,
    work_item_id: &str,
) -> Result<LifecycleReceipt, ObserverError> {
    archive_work_item_internal(root, work_item_id, None)
}

/// Archive a Work Item only when its evidence was produced by this Runtime
/// identity.  This is the current CLI/MCP lifecycle boundary.
pub fn archive_work_item_with_runtime(
    root: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> Result<LifecycleReceipt, ObserverError> {
    archive_work_item_internal(root, work_item_id, Some(runtime))
}

/// Reconcile Runtime-owned failed-attempt projections left in `active` after
/// an older or interrupted archive.  This operation is intentionally
/// separate from `archive`: the canonical Work Item is already immutable, so
/// its archive manifest must remain byte-for-byte unchanged.  Moved variants
/// are instead bound by an append-only reconciliation receipt.
pub fn reconcile_active_artifacts(
    root: &Path,
    work_item_id: &str,
) -> Result<ActiveArtifactReconciliationReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let _lifecycle_lock = acquire_lifecycle_lock(&root, work_item_id)?;
    let active = root.join(".ai/work-items/active");
    let archive = root.join(".ai/work-items/archive");
    let manifest_path = archive.join(format!("{work_item_id}.archive.json"));
    let manifest = read_json(&manifest_path)?;
    verify_archive_manifest(&root, work_item_id, &manifest)?;
    let archived_contract_path = archive.join(format!("{work_item_id}.contract.json"));
    let archived_contract = read_contract(&archived_contract_path)?;
    let expected_repository_id = repository_id(&root).to_string();
    if archived_contract.repository_id != expected_repository_id {
        return Err(ObserverError::State {
            path: archived_contract_path,
            message: "archived Work Item repository identity does not match the current repository"
                .into(),
        });
    }
    let active_contract_path = active.join(format!("{work_item_id}.contract.json"));
    match fs::symlink_metadata(&active_contract_path) {
        Ok(_) => {
            return Err(ObserverError::State {
                path: active_contract_path,
                message:
                    "active Work Item must be archived before reconciling historical artifacts"
                        .into(),
            });
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(ObserverError::Read {
                path: active_contract_path,
                source,
            });
        }
    }

    let variants = active_artifact_variants(&active)?
        .into_iter()
        .filter(|variant| {
            active_artifact_variant_name(&variant.name).is_some_and(|(id, _)| id == work_item_id)
        })
        .collect::<Vec<_>>();
    let receipt_path = archive.join(format!("{work_item_id}.artifact-reconciliation.json"));
    if variants.is_empty() {
        if optional_regular_artifact(&receipt_path, "artifact reconciliation receipt")? {
            let value = read_json(&receipt_path)?;
            return serde_json::from_value(value).map_err(|error| ObserverError::State {
                path: receipt_path,
                message: format!("invalid artifact reconciliation receipt: {error}"),
            });
        }
        return Ok(ActiveArtifactReconciliationReceipt {
            schema_version: 1,
            repository_id: expected_repository_id,
            work_item_id: work_item_id.into(),
            state: "already_clean".into(),
            archive_manifest_path: repository_relative_path(&root, &manifest_path),
            moved_artifacts: Vec::new(),
            recorded_at: now(),
        });
    }

    let mut moved: Vec<ActiveArtifactReconciliationArtifact> = Vec::new();
    for variant in &variants {
        let source = active.join(&variant.name);
        optional_regular_artifact(&source, "historical Work Item artifact")?;
        let target = archive.join(&variant.name);
        if fs::symlink_metadata(&target).is_ok() {
            return Err(ObserverError::State {
                path: target,
                message: "artifact reconciliation target already exists".into(),
            });
        }
        let bytes = fs::read(&source).map_err(|source_error| ObserverError::Read {
            path: source.clone(),
            source: source_error,
        })?;
        let digest = Digest::sha256_bytes(&bytes);
        if let Err(source_error) = fs::rename(&source, &target) {
            for artifact in moved.iter().rev() {
                let moved_source = root.join(&artifact.source_path);
                let moved_target = root.join(&artifact.target_path);
                let _ = fs::rename(&moved_target, &moved_source);
            }
            return Err(ObserverError::Read {
                path: target,
                source: source_error,
            });
        }
        moved.push(ActiveArtifactReconciliationArtifact {
            source_path: repository_relative_path(&root, &source),
            target_path: repository_relative_path(&root, &target),
            digest,
        });
    }

    let receipt = ActiveArtifactReconciliationReceipt {
        schema_version: 1,
        repository_id: expected_repository_id,
        work_item_id: work_item_id.into(),
        state: "reconciled".into(),
        archive_manifest_path: repository_relative_path(&root, &manifest_path),
        moved_artifacts: moved,
        recorded_at: now(),
    };
    let value = serde_json::to_value(&receipt).map_err(|error| ObserverError::State {
        path: receipt_path.clone(),
        message: error.to_string(),
    })?;
    let destination = if fs::symlink_metadata(&receipt_path).is_ok() {
        let digest =
            cockpit_protocol::digest_json(&value).map_err(|error| ObserverError::State {
                path: receipt_path.clone(),
                message: error.to_string(),
            })?;
        let digest_string = digest.to_string();
        let digest_suffix = digest_string
            .strip_prefix("sha256:")
            .unwrap_or(&digest_string);
        archive.join(format!(
            "{work_item_id}.artifact-reconciliation.{digest_suffix}.json"
        ))
    } else {
        receipt_path.clone()
    };
    if let Err(error) = atomic_json(&destination, &value) {
        for artifact in &receipt.moved_artifacts {
            let source = root.join(&artifact.source_path);
            let target = root.join(&artifact.target_path);
            let _ = fs::rename(target, source);
        }
        return Err(error);
    }
    Ok(receipt)
}

fn archive_work_item_internal(
    root: &Path,
    work_item_id: &str,
    current_runtime: Option<&RuntimeContext>,
) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let _lifecycle_lock = acquire_lifecycle_lock(&root, work_item_id)?;
    let ai = root.join(".ai");
    let active = ai.join("work-items/active");
    let archive = ai.join("work-items/archive");
    let existing_manifest_path = archive.join(format!("{work_item_id}.archive.json"));
    match fs::symlink_metadata(&existing_manifest_path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(ObserverError::State {
                path: existing_manifest_path,
                message: "archive manifest must not be a symlink".into(),
            });
        }
        Ok(_) => {
            return Err(ObserverError::State {
                path: existing_manifest_path,
                message: "work item is already archived".into(),
            });
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(ObserverError::Read {
                path: existing_manifest_path,
                source,
            });
        }
    }
    let contract_path = active.join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    let summary_path = active.join(format!("{work_item_id}.summary.json"));
    let summary: serde_json::Value = read_json(&summary_path)?;
    let active_leases = list_parallel_slots(&root)?;
    if let Some(lease) = active_leases
        .iter()
        .find(|lease| lease.work_item_id == work_item_id)
    {
        return Err(ObserverError::State {
            path: slot_lease_path(&root, lease.slot_id),
            message: "archive requires releasing the Work Item's active parallel slot".into(),
        });
    }
    if let Some(decision) = load_recovery_decision(&root, work_item_id, current_runtime)?
        .filter(|decision| decision.decision == "supersede")
    {
        if let Some(runtime) = current_runtime
            && (decision.runtime_version != runtime.runtime_version
                || decision.runtime_digest != runtime.runtime_digest)
        {
            return Err(ObserverError::State {
                path: root.join(".ai/decisions"),
                message:
                    "supersession decision Runtime identity does not match the current Runtime"
                        .into(),
            });
        }
        return archive_superseded_work_item(&root, work_item_id, &decision);
    }
    require_explicit_resource_finalization_plan(&contract, &contract_path, "archive")?;
    if summary["state"] != serde_json::json!("finish_ready") {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "archive requires a finish_ready Work Item state".into(),
        });
    }
    if summary["checkpointCount"] != serde_json::json!(1)
        || summary["preflightState"] != serde_json::json!("green")
    {
        return Err(ObserverError::State {
            path: summary_path,
            message: "archive requires one checkpoint and a green preflight result".into(),
        });
    }
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    if contract.checkpoint_policy.is_some() {
        let current_contract_hash = contract_digest(&contract_path)?.to_string();
        if let Err(errors) = validate_checkpoint_evidence_bindings(
            &contract,
            &summary,
            &repository_id(&root).to_string(),
            &snapshot_digest(&snapshot)?.to_string(),
            &current_contract_hash,
        ) {
            return Err(ObserverError::State {
                path: summary_path.clone(),
                message: format!("checkpoint evidence is invalid: {}", errors.join(", ")),
            });
        }
    }
    if verification_evidence_state(&root, &contract, &snapshot, false, current_runtime)?
        != EvidenceState::Complete
    {
        return Err(ObserverError::State {
            path: root
                .join(".ai/evidence")
                .join(format!("{work_item_id}.verification.json")),
            message: "archive requires valid verification evidence".into(),
        });
    }
    if let Some(runtime) = current_runtime {
        require_green_governance_with_runtime(
            &root,
            &contract_path,
            &contract,
            &snapshot,
            "archive",
            runtime,
        )?;
    } else {
        require_green_governance(&root, &contract_path, &contract, &snapshot, "archive")?;
    }
    let outcome_path = active.join(format!("{work_item_id}.outcome.json"));
    let outcome = read_json(&outcome_path)?;
    if outcome["verification"]["status"] != "verified" {
        return Err(ObserverError::State {
            path: outcome_path,
            message: "archive requires a verified outcome".into(),
        });
    }
    if outcome.get("taskOutcomeReport").is_some() {
        let events_path = task_outcome_event_path(&root, work_item_id, false);
        validate_task_outcome_events(&root, &events_path, &contract.repository_id, work_item_id)?;
    }
    fs::create_dir_all(&archive).map_err(|source| ObserverError::Read {
        path: archive.clone(),
        source,
    })?;
    let manifest_path = archive.join(format!("{work_item_id}.archive.json"));
    if manifest_path.exists() {
        return Err(ObserverError::State {
            path: manifest_path,
            message: "archive manifest already exists".into(),
        });
    }
    let mut artifacts: Vec<(String, String)> = vec![
        ("contract".into(), "contract.json".into()),
        ("summary".into(), "summary.json".into()),
        ("outcome".into(), "outcome.json".into()),
    ];
    let events_source = task_outcome_event_path(&root, work_item_id, false);
    if optional_regular_artifact(&events_source, "Task Outcome event stream")? {
        artifacts.push(("events".into(), "events.jsonl".into()));
    }
    let report_source = active.join(format!("{work_item_id}.task-report.json"));
    if optional_regular_artifact(&report_source, "Task Outcome report")? {
        artifacts.push(("taskReport".into(), "task-report.json".into()));
    }
    let markdown_source = active.join(format!("{work_item_id}.task-report.md"));
    if optional_regular_artifact(&markdown_source, "Task Outcome Markdown report")? {
        artifacts.push(("taskReportMarkdown".into(), "task-report.md".into()));
    }
    let approach_source = active.join(format!("{work_item_id}.approach.json"));
    if optional_regular_artifact(&approach_source, "Implementation approach")? {
        artifacts.push(("approach".into(), "approach.json".into()));
    }
    let intelligence_source = active.join(format!("{work_item_id}.intelligence.json"));
    if optional_regular_artifact(&intelligence_source, "Work Item intelligence sidecar")? {
        artifacts.push(("intelligence".into(), "intelligence.json".into()));
    }
    for (index, variant) in active_artifact_variants(&active)?.into_iter().enumerate() {
        let Some(suffix) = variant.name.strip_prefix(&format!("{work_item_id}.")) else {
            continue;
        };
        artifacts.push((format!("historicalArtifact{index}"), suffix.to_owned()));
    }
    let mut pending = Vec::new();
    for (name, suffix) in artifacts {
        let source_path = active.join(format!("{work_item_id}.{suffix}"));
        if name.starts_with("historicalArtifact")
            && !optional_regular_artifact(&source_path, "historical Work Item artifact")?
        {
            continue;
        }
        let target = archive.join(format!("{work_item_id}.{suffix}"));
        let source_bytes = fs::read(&source_path).map_err(|error| ObserverError::Read {
            path: source_path.clone(),
            source: error,
        })?;
        let archived_bytes =
            normalized_archive_artifact_bytes(&suffix, &source_bytes, work_item_id)?;
        if target.exists() {
            return Err(ObserverError::State {
                path: target,
                message: "archive target already exists".into(),
            });
        }
        pending.push((
            name.to_string(),
            suffix.to_string(),
            source_path,
            target,
            source_bytes,
            archived_bytes,
        ));
    }
    let task_report_digest = pending
        .iter()
        .find(|(_, suffix, ..)| suffix == "task-report.json")
        .map(|(_, _, _, _, _, bytes)| Digest::sha256_bytes(bytes).to_string());
    let task_report_markdown_digest = pending
        .iter()
        .find(|(_, suffix, ..)| suffix == "task-report.md")
        .map(|(_, _, _, _, _, bytes)| Digest::sha256_bytes(bytes).to_string());
    if let Some((_, suffix, _, _, _, archived_bytes)) = pending
        .iter_mut()
        .find(|(_, suffix, ..)| suffix == "outcome.json")
    {
        let mut outcome: serde_json::Value =
            serde_json::from_slice(archived_bytes).map_err(|error| ObserverError::State {
                path: archive.join(format!("{work_item_id}.{suffix}")),
                message: format!("invalid normalized Outcome while archiving: {error}"),
            })?;
        if let Some(digest) = task_report_digest {
            outcome["taskReportDigest"] = serde_json::Value::String(digest);
        }
        if let Some(digest) = task_report_markdown_digest {
            outcome["taskReportMarkdownDigest"] = serde_json::Value::String(digest);
        }
        *archived_bytes =
            serde_json::to_vec_pretty(&outcome).map_err(|error| ObserverError::State {
                path: archive.join(format!("{work_item_id}.{suffix}")),
                message: format!("serialize normalized Outcome while archiving: {error}"),
            })?;
    }
    let mut files = serde_json::Map::new();
    let mut historical_artifacts = Vec::new();
    for (name, suffix, _, _, _, archived_bytes) in &pending {
        files.insert(
            format!("{name}Path"),
            serde_json::Value::String(format!(".ai/work-items/archive/{work_item_id}.{suffix}")),
        );
        files.insert(
            format!("{name}Digest"),
            serde_json::Value::String(Digest::sha256_bytes(archived_bytes).to_string()),
        );
        if name.starts_with("historicalArtifact") {
            let path = format!(".ai/work-items/archive/{work_item_id}.{suffix}");
            let kind = if suffix.starts_with("outcome.") {
                "outcome"
            } else {
                "events"
            };
            historical_artifacts.push(serde_json::json!({
                "path": path,
                "kind": kind,
                "digest": Digest::sha256_bytes(archived_bytes),
            }));
        }
    }
    let mut moved: Vec<(PathBuf, PathBuf, Vec<u8>, bool)> = Vec::new();
    for (_, _, source, target, source_bytes, archived_bytes) in &pending {
        let normalized = source_bytes != archived_bytes;
        let result: Result<(), ObserverError> = if normalized {
            atomic_write(target, archived_bytes)
        } else {
            fs::rename(source, target).map_err(|source_error| ObserverError::Read {
                path: target.clone(),
                source: source_error,
            })
        };
        if let Err(error) = result {
            for (moved_source, moved_target, original, moved_normalized) in moved.into_iter().rev()
            {
                if moved_normalized {
                    let _ = fs::remove_file(moved_target);
                    let _ = atomic_write(&moved_source, &original);
                } else {
                    let _ = fs::rename(moved_target, moved_source);
                }
            }
            return Err(error);
        }
        moved.push((
            source.clone(),
            target.clone(),
            source_bytes.clone(),
            normalized,
        ));
    }
    for (source, _target, _original, normalized) in &moved {
        if *normalized && let Err(source_error) = fs::remove_file(source) {
            for (moved_source, moved_target, moved_original, moved_normalized) in moved.iter().rev()
            {
                if *moved_normalized {
                    let _ = fs::remove_file(moved_target);
                    let _ = atomic_write(moved_source, moved_original);
                } else {
                    let _ = fs::rename(moved_target, moved_source);
                }
            }
            return Err(ObserverError::Read {
                path: source.clone(),
                source: source_error,
            });
        }
    }
    let timestamp = now();
    let manifest = serde_json::json!({
        "protocolVersion": 1,
        "workItemId": work_item_id,
        "state": "archived",
        "closeRequired": true,
        "files": files,
        "historicalArtifacts": historical_artifacts,
        "createdAt": timestamp,
    });
    if let Err(error) = atomic_json(&manifest_path, &manifest) {
        for (moved_source, moved_target, original, normalized) in moved.into_iter().rev() {
            if normalized {
                let _ = fs::remove_file(moved_target);
                let _ = atomic_write(&moved_source, &original);
            } else {
                let _ = fs::rename(moved_target, moved_source);
            }
        }
        return Err(error);
    }
    let _ = fs::remove_file(ai.join("knowledge/index.json"));
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "archived".into(),
        timestamp,
    })
}

fn archive_superseded_work_item(
    root: &Path,
    work_item_id: &str,
    decision: &RecoveryDecisionReceipt,
) -> Result<LifecycleReceipt, ObserverError> {
    let ai = root.join(".ai");
    let active = ai.join("work-items/active");
    let archive = ai.join("work-items/archive");
    fs::create_dir_all(&archive).map_err(|source| ObserverError::Read {
        path: archive.clone(),
        source,
    })?;
    let manifest_path = archive.join(format!("{work_item_id}.archive.json"));
    if fs::symlink_metadata(&manifest_path).is_ok() {
        return Err(ObserverError::State {
            path: manifest_path,
            message: "archive manifest already exists".into(),
        });
    }
    let mut candidates: Vec<(String, String)> = vec![
        ("contract".into(), "contract.json".into()),
        ("summary".into(), "summary.json".into()),
        ("outcome".into(), "outcome.json".into()),
        ("approach".into(), "approach.json".into()),
        ("intelligence".into(), "intelligence.json".into()),
        ("events".into(), "events.jsonl".into()),
        ("taskReport".into(), "task-report.json".into()),
        ("taskReportMarkdown".into(), "task-report.md".into()),
    ];
    for (index, variant) in active_artifact_variants(&active)?.into_iter().enumerate() {
        let Some(suffix) = variant.name.strip_prefix(&format!("{work_item_id}.")) else {
            continue;
        };
        candidates.push((format!("historicalArtifact{index}"), suffix.to_owned()));
    }
    let mut files = serde_json::Map::new();
    let mut historical_artifacts = Vec::new();
    let mut pending = Vec::new();
    for (name, suffix) in candidates {
        let source = active.join(format!("{work_item_id}.{suffix}"));
        if !optional_regular_artifact(&source, name.as_str())? {
            continue;
        }
        let target = archive.join(format!("{work_item_id}.{suffix}"));
        if fs::symlink_metadata(&target).is_ok() {
            return Err(ObserverError::State {
                path: target,
                message: "superseded archive target already exists".into(),
            });
        }
        let bytes = fs::read(&source).map_err(|source_error| ObserverError::Read {
            path: source.clone(),
            source: source_error,
        })?;
        files.insert(
            format!("{name}Path"),
            serde_json::json!(format!(".ai/work-items/archive/{work_item_id}.{suffix}")),
        );
        files.insert(
            format!("{name}Digest"),
            serde_json::json!(Digest::sha256_bytes(&bytes).to_string()),
        );
        if name.starts_with("historicalArtifact") {
            historical_artifacts.push(serde_json::json!({
                "path": format!(".ai/work-items/archive/{work_item_id}.{suffix}"),
                "kind": if suffix.starts_with("outcome.") { "outcome" } else { "events" },
                "digest": Digest::sha256_bytes(&bytes),
            }));
        }
        pending.push((source, target));
    }
    for (required, suffix) in [
        ("contract", "contract.json"),
        ("summary", "summary.json"),
        ("outcome", "outcome.json"),
    ] {
        if !files.contains_key(&format!("{required}Digest")) {
            return Err(ObserverError::State {
                path: active.join(format!("{work_item_id}.{suffix}")),
                message: "superseded archive requires contract, summary, and outcome".into(),
            });
        }
    }
    let mut moved = Vec::new();
    for (source, target) in &pending {
        if let Err(source_error) = fs::rename(source, target) {
            for (moved_source, moved_target) in moved.into_iter().rev() {
                let _ = fs::rename(moved_target, moved_source);
            }
            return Err(ObserverError::Read {
                path: target.clone(),
                source: source_error,
            });
        }
        moved.push((source.clone(), target.clone()));
    }
    let timestamp = now();
    let decision_value = serde_json::to_value(decision).map_err(|error| ObserverError::State {
        path: ai.join("decisions"),
        message: error.to_string(),
    })?;
    let canonical_decision_path = ai
        .join("decisions")
        .join(format!("{work_item_id}.recovery.json"));
    let decision_path =
        if read_json(&canonical_decision_path).ok().as_ref() == Some(&decision_value) {
            canonical_decision_path
        } else {
            let digest = cockpit_protocol::digest_json(&decision_value).map_err(|error| {
                ObserverError::State {
                    path: ai.join("decisions"),
                    message: error.to_string(),
                }
            })?;
            let digest_string = digest.to_string();
            let suffix = digest_string
                .strip_prefix("sha256:")
                .unwrap_or(&digest_string);
            ai.join("decisions")
                .join(format!("{work_item_id}.recovery.{suffix}.json"))
        };
    let manifest = serde_json::json!({
        "protocolVersion": 1,
        "workItemId": work_item_id,
        "state": "superseded",
        "historicalEvidence": true,
        "supersededBy": decision.successor_work_item_id,
        "supersessionDecisionPath": repository_relative_path(root, &decision_path),
        "files": files,
        "historicalArtifacts": historical_artifacts,
        "createdAt": timestamp,
    });
    if let Err(error) = atomic_json(&manifest_path, &manifest) {
        for (moved_source, moved_target) in moved.into_iter().rev() {
            let _ = fs::rename(moved_target, moved_source);
        }
        return Err(error);
    }
    let _ = fs::remove_file(ai.join("knowledge/index.json"));
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "superseded".into(),
        timestamp,
    })
}

/// Return a best-effort local branch/worktree context for a newly started
/// Work Item.  Provider/PR identity is intentionally provisional until an
/// explicit `finalize-plan` receipt is supplied; no local fact is promoted to
/// provider assurance implicitly.
fn provisional_resource_context(root: &Path) -> ResourceFinalizationContext {
    let branch = git_text(root, &["symbolic-ref", "--quiet", "--short", "HEAD"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "detached".into());
    ResourceFinalizationContext {
        branch,
        worktree: root.to_string_lossy().into_owned(),
        base_branch: "unknown".into(),
        base_remote: "unknown".into(),
        provider: "unknown".into(),
        pull_request: "unknown".into(),
    }
}

fn git_text(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8(output.stdout).ok()?.trim().to_owned())
}

fn set_resource_context_on_active_contract(
    root: &Path,
    work_item_id: &str,
    context: &ResourceFinalizationContext,
) -> Result<Contract, ObserverError> {
    cockpit_protocol::validate_resource_finalization_context(context).map_err(|error| {
        ObserverError::State {
            path: root
                .join(".ai/work-items/active")
                .join(format!("{work_item_id}.contract.json")),
            message: error.to_string(),
        }
    })?;
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let mut value = read_json(&contract_path)?;
    value["resourceContext"] =
        serde_json::to_value(context).map_err(|error| ObserverError::State {
            path: contract_path.clone(),
            message: error.to_string(),
        })?;
    atomic_json(&contract_path, &value)?;
    read_contract(&contract_path)
}

fn require_explicit_resource_finalization_plan(
    contract: &Contract,
    contract_path: &Path,
    operation: &str,
) -> Result<(), ObserverError> {
    let Some(context) = contract.resource_context.as_ref() else {
        return Err(ObserverError::State {
            path: contract_path.to_path_buf(),
            message: format!(
                "{operation} requires an explicit resource finalization plan; run finalize-plan before {operation}"
            ),
        });
    };
    if context.is_provisional() {
        return Err(ObserverError::State {
            path: contract_path.to_path_buf(),
            message: format!(
                "{operation} requires a non-provisional resource finalization plan; run finalize-plan before {operation}"
            ),
        });
    }
    cockpit_protocol::validate_resource_finalization_context(context).map_err(|error| {
        ObserverError::State {
            path: contract_path.to_path_buf(),
            message: format!("{operation} resource finalization plan is invalid: {error}"),
        }
    })
}

/// Bind provider/branch/worktree context to an active Contract before it is
/// archived.  The operation is deliberately explicit and idempotent only for
/// the same context; replacing a complete context after planning is refused.
pub fn plan_resource_finalization(
    root: &Path,
    work_item_id: &str,
    context: &ResourceFinalizationContext,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let contract = read_contract(&contract_path)?;
    let summary = read_json(&summary_path)?;
    if !matches!(
        summary["state"].as_str(),
        Some("implementation_active" | "checkpointed")
    ) {
        return Err(ObserverError::State {
            path: summary_path,
            message: "finalize-plan must run before verification/finish_ready; changing resource context after the verification cycle would invalidate lifecycle evidence".into(),
        });
    }
    if let Some(existing) = &contract.resource_context
        && existing != context
        && !existing.is_provisional()
    {
        return Err(ObserverError::State {
            path: contract_path,
            message:
                "resource finalization context is already bound; use the same context for replay"
                    .into(),
        });
    }
    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    if summary["state"] == serde_json::json!("finish_ready")
        && fs::symlink_metadata(&evidence_path).is_ok()
        && contract.resource_context.as_ref() != Some(context)
    {
        return Err(ObserverError::State {
            path: evidence_path,
            message: "finalize-plan must run before verification evidence is recorded; re-run verify after changing the context".into(),
        });
    }
    set_resource_context_on_active_contract(&root, work_item_id, context)?;
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let digest =
        Digest::sha256_bytes(
            &fs::read(&contract_path).map_err(|source| ObserverError::Read {
                path: contract_path.clone(),
                source,
            })?,
        );
    Ok(serde_json::json!({
        "protocolVersion": 1,
        "workItemId": work_item_id,
        "state": "planned",
        "resourceContext": context,
        "contractDigest": digest,
        "next": ["archive", "finalize", "finalize-verify", "close"]
    }))
}

fn read_resource_finalization_receipt(
    path: &Path,
) -> Result<ResourceFinalizationReceipt, ObserverError> {
    if !is_regular_non_symlink(path)? {
        return Err(ObserverError::State {
            path: path.into(),
            message: "resource finalization receipt must be a regular non-symlink file".into(),
        });
    }
    let bytes = fs::read(path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })?;
    reject_duplicate_json_keys(&bytes).map_err(|message| ObserverError::State {
        path: path.into(),
        message: format!("invalid resource finalization receipt JSON: {message}"),
    })?;
    serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
        path: path.into(),
        message: format!("invalid resource finalization receipt: {error}"),
    })
}

fn resource_finalization_decision_path(root: &Path, work_item_id: &str) -> PathBuf {
    root.join(".ai/decisions")
        .join(format!("{work_item_id}.finalize.json"))
}

fn read_resource_finalization_transition(
    path: &Path,
) -> Result<ResourceFinalizationTransitionReceipt, ObserverError> {
    if !is_regular_non_symlink(path)? {
        return Err(ObserverError::State {
            path: path.into(),
            message: "resource finalization transition must be a regular non-symlink file".into(),
        });
    }
    let bytes = fs::read(path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })?;
    reject_duplicate_json_keys(&bytes).map_err(|message| ObserverError::State {
        path: path.into(),
        message: format!("invalid resource finalization transition JSON: {message}"),
    })?;
    serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
        path: path.into(),
        message: format!("invalid resource finalization transition: {error}"),
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PostFinalizeQualityRoutePathDecision {
    path: String,
    profile: String,
    reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PostFinalizeQualityRouteReceipt {
    schema_version: u32,
    kind: String,
    automatic_profile: String,
    base_revision: String,
    changed_paths: Vec<String>,
    contract_digest: Digest,
    contract_path: String,
    head_revision: String,
    manifest_digest: Digest,
    path_decisions: Vec<PostFinalizeQualityRoutePathDecision>,
    reasons: Vec<String>,
    receipt_digest: Digest,
    requested_profile: Option<String>,
    requested_risk: String,
    required_gate_ids: Vec<String>,
    risk: String,
    selected_profile: String,
    stage: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PostFinalizeRepositoryGate {
    id: String,
    category: String,
    command: Vec<String>,
    #[serde(default)]
    covers: Vec<String>,
    state: String,
    exit_code: i32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PostFinalizeRepositoryGateRoute {
    manifest_digest: Digest,
    receipt_digest: Digest,
    required_gate_ids: Vec<String>,
    selected_profile: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PostFinalizeRepositoryGatesReceipt {
    schema_version: u32,
    state: String,
    route: PostFinalizeRepositoryGateRoute,
    gates: Vec<PostFinalizeRepositoryGate>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PostFinalizeEvidenceKind {
    QualityRoute,
    RepositoryGates,
}

fn post_finalize_evidence_kind(
    work_item_id: &str,
    candidate: &str,
) -> Option<PostFinalizeEvidenceKind> {
    let prefix = format!(".ai/evidence/{work_item_id}/");
    match candidate.strip_prefix(&prefix)? {
        "quality-route-post-finalize.json" => Some(PostFinalizeEvidenceKind::QualityRoute),
        "repository-gates-post-finalize.json" => Some(PostFinalizeEvidenceKind::RepositoryGates),
        _ => None,
    }
}

fn read_governance_append_blob(
    root: &Path,
    revision: &str,
    candidate: &str,
    path: &Path,
) -> Result<Vec<u8>, ObserverError> {
    let object = format!("{revision}:{candidate}");
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "blob", &object])
        .output()
        .map_err(|source| ObserverError::Read {
            path: path.into(),
            source,
        })?;
    if !output.status.success() {
        return Err(ObserverError::State {
            path: path.into(),
            message: "cannot read governance append evidence blob".into(),
        });
    }
    if output.stdout.len() > MAX_EXTERNAL_EVIDENCE_BYTES {
        return Err(ObserverError::State {
            path: path.into(),
            message: "governance append evidence exceeds the bounded size limit".into(),
        });
    }
    reject_duplicate_json_keys(&output.stdout).map_err(|message| ObserverError::State {
        path: path.into(),
        message: format!("invalid governance append evidence JSON: {message}"),
    })?;
    Ok(output.stdout)
}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn validate_post_finalize_evidence_bundle(
    root: &Path,
    work_item_id: &str,
    previous: &ResourceFinalizationReceipt,
    append_revision: &str,
    quality_bytes: &[u8],
    gates_bytes: &[u8],
    path: &Path,
) -> Result<(), ObserverError> {
    let quality_value: serde_json::Value =
        serde_json::from_slice(quality_bytes).map_err(|error| ObserverError::State {
            path: path.into(),
            message: format!("invalid post-finalize quality route JSON: {error}"),
        })?;
    let quality: PostFinalizeQualityRouteReceipt = serde_json::from_value(quality_value.clone())
        .map_err(|error| ObserverError::State {
            path: path.into(),
            message: format!("invalid post-finalize quality route schema: {error}"),
        })?;
    let gates: PostFinalizeRepositoryGatesReceipt =
        serde_json::from_slice(gates_bytes).map_err(|error| ObserverError::State {
            path: path.into(),
            message: format!("invalid post-finalize repository gates schema: {error}"),
        })?;

    let expected_contract_path = format!(".ai/work-items/archive/{work_item_id}.contract.json");
    let Some(expected_contract_digest) = previous.contract_digest.as_ref() else {
        return Err(ObserverError::State {
            path: path.into(),
            message: "post-finalize evidence requires a contract-bound predecessor receipt".into(),
        });
    };
    let quality_head_is_bounded = valid_git_object_id(&quality.head_revision)
        && git_text(
            root,
            &[
                "merge-base",
                "--is-ancestor",
                &previous.pull_request.head_revision,
                &quality.head_revision,
            ],
        )
        .is_some()
        && git_text(
            root,
            &[
                "merge-base",
                "--is-ancestor",
                &quality.head_revision,
                append_revision,
            ],
        )
        .is_some();
    let string_lists_are_valid = [
        quality.changed_paths.as_slice(),
        quality.reasons.as_slice(),
        quality.required_gate_ids.as_slice(),
    ]
    .into_iter()
    .all(|values| !values.is_empty() && values.iter().all(|value| nonempty(value)));
    let path_decisions_are_valid = !quality.path_decisions.is_empty()
        && quality.path_decisions.iter().all(|decision| {
            nonempty(&decision.path) && nonempty(&decision.profile) && nonempty(&decision.reason)
        })
        && quality
            .path_decisions
            .iter()
            .map(|decision| decision.path.as_str())
            .eq(quality.changed_paths.iter().map(String::as_str));
    if quality.schema_version != 1
        || quality.kind != "repository_quality_route"
        || quality.stage != "pull_request"
        || quality.contract_path != expected_contract_path
        || &quality.contract_digest != expected_contract_digest
        || quality.base_revision != previous.pull_request.base_revision
        || !quality_head_is_bounded
        || !string_lists_are_valid
        || !path_decisions_are_valid
        || !nonempty(&quality.automatic_profile)
        || !nonempty(&quality.risk)
        || !nonempty(&quality.requested_risk)
        || quality
            .requested_profile
            .as_deref()
            .is_some_and(|value| !nonempty(value))
        || !nonempty(&quality.selected_profile)
    {
        return Err(ObserverError::State {
            path: path.into(),
            message: "post-finalize quality route binding is invalid".into(),
        });
    }

    let mut digest_payload = quality_value;
    let Some(payload) = digest_payload.as_object_mut() else {
        return Err(ObserverError::State {
            path: path.into(),
            message: "post-finalize quality route must be a JSON object".into(),
        });
    };
    payload.remove("receiptDigest");
    let computed_receipt_digest =
        cockpit_protocol::digest_json(&digest_payload).map_err(|error| ObserverError::State {
            path: path.into(),
            message: format!("cannot digest post-finalize quality route: {error}"),
        })?;
    if quality.receipt_digest != computed_receipt_digest {
        return Err(ObserverError::State {
            path: path.into(),
            message: "post-finalize quality route receipt digest mismatch".into(),
        });
    }

    let route_ids = quality
        .required_gate_ids
        .iter()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let gate_ids = gates
        .gates
        .iter()
        .map(|gate| gate.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let gates_are_valid = !gates.gates.is_empty()
        && gate_ids.len() == gates.gates.len()
        && route_ids.len() == quality.required_gate_ids.len()
        && route_ids == gate_ids
        && gates.gates.iter().all(|gate| {
            nonempty(&gate.id)
                && nonempty(&gate.category)
                && !gate.command.is_empty()
                && gate.command.iter().all(|value| nonempty(value))
                && gate.covers.iter().all(|value| nonempty(value))
                && gate.state == "passed"
                && gate.exit_code == 0
        });
    if gates.schema_version != 2
        || gates.state != "passed"
        || gates.route.manifest_digest != quality.manifest_digest
        || gates.route.receipt_digest != quality.receipt_digest
        || gates.route.required_gate_ids != quality.required_gate_ids
        || gates.route.selected_profile != quality.selected_profile
        || !gates_are_valid
    {
        return Err(ObserverError::State {
            path: path.into(),
            message: "post-finalize repository gates binding is invalid".into(),
        });
    }
    Ok(())
}

fn validate_governance_append_revision(
    root: &Path,
    work_item_id: &str,
    previous: &ResourceFinalizationReceipt,
    transition: &ResourceFinalizationTransitionReceipt,
    path: &Path,
) -> Result<(), ObserverError> {
    let Some(append_revision) = transition.governance_append_revision.as_deref() else {
        return Ok(());
    };
    let previous_spec = format!("{}^{{commit}}", previous.pull_request.head_revision);
    let append_spec = format!("{append_revision}^{{commit}}");
    let previous_revision = git_text(
        root,
        &["rev-parse", "--verify", "--end-of-options", &previous_spec],
    )
    .ok_or_else(|| ObserverError::State {
        path: path.into(),
        message: "governance append predecessor revision is not a local commit".into(),
    })?;
    let append_revision = git_text(
        root,
        &["rev-parse", "--verify", "--end-of-options", &append_spec],
    )
    .ok_or_else(|| ObserverError::State {
        path: path.into(),
        message: "governance append revision is not a local commit".into(),
    })?;
    if git_text(
        root,
        &[
            "merge-base",
            "--is-ancestor",
            &previous_revision,
            &append_revision,
        ],
    )
    .is_none()
    {
        return Err(ObserverError::State {
            path: path.into(),
            message: "governance append revision does not descend from the predecessor head".into(),
        });
    }
    let changes = git_text(
        root,
        &[
            "diff",
            "--name-status",
            &previous_revision,
            &append_revision,
            "--",
        ],
    )
    .ok_or_else(|| ObserverError::State {
        path: path.into(),
        message: "cannot inspect governance append revision changes".into(),
    })?;
    let canonical = format!(".ai/decisions/{work_item_id}.finalize.json");
    let transition_prefix = format!(".ai/decisions/{work_item_id}.finalize.");
    let allowed = |candidate: &str| {
        candidate == canonical
            || candidate
                .strip_prefix(&transition_prefix)
                .and_then(|suffix| suffix.strip_suffix(".json"))
                .is_some_and(|digest| {
                    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
                })
    };
    let mut finalization_count = 0usize;
    let mut quality_route = None;
    let mut repository_gates = None;
    for change in changes.lines() {
        let Some(candidate) = change.strip_prefix("A\t") else {
            return Err(ObserverError::State {
                path: path.into(),
                message: "governance append revision contains a non-append change".into(),
            });
        };
        let evidence_kind = post_finalize_evidence_kind(work_item_id, candidate);
        if !allowed(candidate) && evidence_kind.is_none() {
            return Err(ObserverError::State {
                path: path.into(),
                message: "governance append revision contains a foreign path".into(),
            });
        }
        let tree_entry = git_text(root, &["ls-tree", &append_revision, "--", candidate])
            .ok_or_else(|| ObserverError::State {
                path: path.into(),
                message: "cannot inspect governance append receipt file mode".into(),
            })?;
        if !tree_entry.starts_with("100644 blob ") || !tree_entry.ends_with(candidate) {
            return Err(ObserverError::State {
                path: path.into(),
                message: "governance append receipt is not a regular non-symlink JSON file".into(),
            });
        }
        if allowed(candidate) {
            finalization_count += 1;
        } else if let Some(kind) = evidence_kind {
            let bytes = read_governance_append_blob(root, &append_revision, candidate, path)?;
            match kind {
                PostFinalizeEvidenceKind::QualityRoute => quality_route = Some(bytes),
                PostFinalizeEvidenceKind::RepositoryGates => repository_gates = Some(bytes),
            }
        }
    }
    if finalization_count == 0 {
        return Err(ObserverError::State {
            path: path.into(),
            message: "governance append revision contains no finalization receipt append".into(),
        });
    }
    match (quality_route.as_deref(), repository_gates.as_deref()) {
        (None, None) => {}
        (Some(quality_route), Some(repository_gates)) => validate_post_finalize_evidence_bundle(
            root,
            work_item_id,
            previous,
            &append_revision,
            quality_route,
            repository_gates,
            path,
        )?,
        _ => {
            return Err(ObserverError::State {
                path: path.into(),
                message: "governance append revision must include the complete post-finalize evidence bundle"
                    .into(),
            });
        }
    }
    Ok(())
}

fn resolve_resource_finalization_head(
    root: &Path,
    work_item_id: &str,
) -> Result<(ResourceFinalizationReceipt, PathBuf, Digest, u64), ObserverError> {
    let (receipt, path, digest) = read_resource_finalization_head(root, work_item_id)?;
    let prefix = format!("{work_item_id}.finalize.");
    let canonical_name = format!("{work_item_id}.finalize.json");
    let candidates = fs::read_dir(root.join(".ai/decisions"))
        .map_err(|source| ObserverError::Read {
            path: root.join(".ai/decisions"),
            source,
        })?
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            (name != canonical_name && name.starts_with(&prefix) && name.ends_with(".json"))
                .then_some((entry.path(), name))
        })
        .map(|(candidate, name)| {
            let value = read_resource_finalization_transition(&candidate)?;
            let encoded = serde_json::to_value(&value).map_err(|error| ObserverError::State {
                path: candidate.clone(),
                message: error.to_string(),
            })?;
            let digest =
                cockpit_protocol::digest_json(&encoded).map_err(|error| ObserverError::State {
                    path: candidate.clone(),
                    message: error.to_string(),
                })?;
            let digest = digest.to_string();
            let expected = format!(
                "{work_item_id}.finalize.{}.json",
                digest.strip_prefix("sha256:").unwrap_or(&digest)
            );
            if name != expected {
                return Err(ObserverError::State {
                    path: candidate,
                    message: "resource finalization transition filename digest mismatch".into(),
                });
            }
            Ok((candidate, value))
        })
        .collect::<Result<Vec<_>, _>>()?;
    resolve_resource_finalization_head_from_observed(
        root,
        work_item_id,
        receipt,
        path,
        digest,
        candidates,
    )
}

fn resolve_resource_finalization_head_with_index(
    root: &Path,
    work_item_id: &str,
    index: &status_projection::FinalizationTransitionIndex,
) -> Result<(ResourceFinalizationReceipt, PathBuf, Digest, u64), ObserverError> {
    let (receipt, path, digest) = read_resource_finalization_head(root, work_item_id)?;
    let candidates = index
        .candidates(work_item_id)
        .iter()
        .map(|candidate| match candidate {
            status_projection::IndexedFinalizationTransition::Valid {
                path,
                digest,
                value,
            } => {
                let _validated_digest = digest;
                Ok((path.clone(), (**value).clone()))
            }
            status_projection::IndexedFinalizationTransition::Invalid { path, message } => {
                Err(ObserverError::State {
                    path: path.clone(),
                    message: message.clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    resolve_resource_finalization_head_from_observed(
        root,
        work_item_id,
        receipt,
        path,
        digest,
        candidates,
    )
}

fn read_resource_finalization_head(
    root: &Path,
    work_item_id: &str,
) -> Result<(ResourceFinalizationReceipt, PathBuf, Digest), ObserverError> {
    let canonical = resource_finalization_decision_path(root, work_item_id);
    let receipt = read_resource_finalization_receipt(&canonical)?;
    let digest =
        cockpit_protocol::digest_json(&serde_json::to_value(&receipt).map_err(|error| {
            ObserverError::State {
                path: canonical.clone(),
                message: error.to_string(),
            }
        })?)
        .map_err(|error| ObserverError::State {
            path: canonical.clone(),
            message: error.to_string(),
        })?;
    Ok((receipt, canonical, digest))
}

fn resolve_resource_finalization_head_from_observed(
    root: &Path,
    work_item_id: &str,
    mut receipt: ResourceFinalizationReceipt,
    mut path: PathBuf,
    mut digest: Digest,
    mut candidates: Vec<(PathBuf, ResourceFinalizationTransitionReceipt)>,
) -> Result<(ResourceFinalizationReceipt, PathBuf, Digest, u64), ObserverError> {
    let mut sequence = 0;
    loop {
        let matches = candidates
            .iter()
            .enumerate()
            .filter(|(_, (_, value))| value.predecessor_receipt_digest == digest)
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if matches.is_empty() {
            break;
        }
        if matches.len() != 1 {
            return Err(ObserverError::State {
                path: path.clone(),
                message: "resource finalization transition chain is forked".into(),
            });
        }
        let (next_path, transition) = candidates.remove(matches[0]);
        validate_resource_finalization_transition(&receipt, &transition, sequence + 1).map_err(
            |error| ObserverError::State {
                path: next_path.clone(),
                message: error.to_string(),
            },
        )?;
        validate_governance_append_revision(root, work_item_id, &receipt, &transition, &next_path)?;
        sequence += 1;
        receipt = transition.receipt;
        path = next_path;
        digest =
            cockpit_protocol::digest_json(&serde_json::to_value(&receipt).map_err(|error| {
                ObserverError::State {
                    path: path.clone(),
                    message: error.to_string(),
                }
            })?)
            .map_err(|error| ObserverError::State {
                path: path.clone(),
                message: error.to_string(),
            })?;
    }
    if !candidates.is_empty() {
        return Err(ObserverError::State {
            path: candidates[0].0.clone(),
            message: "resource finalization transition has a missing or stale predecessor".into(),
        });
    }
    Ok((receipt, path, digest, sequence))
}

fn archived_contract_digest(
    root: &Path,
    work_item_id: &str,
) -> Result<(Contract, Digest), ObserverError> {
    let path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&path)?;
    let digest = Digest::sha256_bytes(&fs::read(&path).map_err(|source| ObserverError::Read {
        path: path.clone(),
        source,
    })?);
    Ok((contract, digest))
}

fn ensure_resource_runtime_identity(
    receipt: &ResourceFinalizationReceipt,
    runtime: &RuntimeContext,
    path: &Path,
) -> Result<(), ObserverError> {
    if receipt.runtime_version != runtime.runtime_version
        || receipt.runtime_digest != runtime.runtime_digest
    {
        return Err(ObserverError::State {
            path: path.into(),
            message: "resource finalization receipt Runtime identity does not match the executing Runtime".into(),
        });
    }
    Ok(())
}

/// Return the historical classification for an old-runtime finalization head
/// that is already closed.  Historical bytes remain immutable: the only
/// authority for this projection is the close receipt binding its exact head
/// path, digest, sequence, Work Item, and repository identity.
fn closed_finalization_projection_kind(
    root: &Path,
    work_item_id: &str,
    receipt: &ResourceFinalizationReceipt,
    receipt_path: &Path,
    receipt_digest: &Digest,
    sequence: u64,
    repository_id: &str,
) -> Option<&'static str> {
    if !close_decision_is_valid_for_status(root, work_item_id, repository_id) {
        return None;
    }
    let close_path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    let close = read_json(&close_path).ok()?;
    if close.get("resourceFinalizationSequence")?.as_u64()? != sequence
        || close.get("resourceFinalizationHeadPath")?.as_str()?
            != repository_relative_path(root, receipt_path)
        || close.get("resourceFinalizationHeadDigest")?.as_str()? != receipt_digest.to_string()
    {
        return None;
    }
    if let Some(historical) = close.get("historicalRevalidation")
        && historical.get("state").and_then(serde_json::Value::as_str)
            == Some("current_successor_revalidated")
        && let Ok(Some(recovery)) = load_recovery_decision(root, work_item_id, None)
        && recovery.successor_binding_mode.as_deref() == Some("contract_amendment_revalidation")
        && recovery.successor_work_item_id.as_deref()
            == historical
                .get("successorWorkItemId")
                .and_then(serde_json::Value::as_str)
        && recovery.predecessor_finalization_contract_digest.as_ref()
            == receipt.contract_digest.as_ref()
        && historical
            .get("assurance")
            .and_then(serde_json::Value::as_str)
            == Some("historical_low")
        && historical
            .get("originalEvidencePreserved")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        && historical_digest_matches(
            historical,
            "historicalContractDigest",
            &recovery.predecessor_contract_digest,
        )
        && recovery
            .current_contract_digest
            .as_ref()
            .is_some_and(|digest| {
                historical_digest_matches(historical, "currentContractDigest", digest)
            })
        && recovery
            .predecessor_verification_evidence_digest
            .as_ref()
            .is_some_and(|digest| {
                historical_digest_matches(
                    historical,
                    "historicalVerificationEvidenceDigest",
                    digest,
                )
            })
        && recovery
            .predecessor_archive_manifest_digest
            .as_ref()
            .is_some_and(|digest| {
                historical_digest_matches(historical, "archiveManifestDigest", digest)
            })
        && recovery_successor_resolves_pending_close(root, work_item_id, repository_id)
    {
        return Some("contract_amendment_revalidation");
    }
    if close.get("historicalRevalidation").is_some() {
        // A close record that declares this projection must satisfy the full
        // lineage binding above. Do not downgrade an invalid declaration to
        // the generic shared-worktree compatibility lane.
        return None;
    }
    if matches!(
        receipt.result.disposition,
        ResourceFinalizationDisposition::Retained
    ) && receipt.before.branch == receipt.after.branch
        && receipt.before.worktree == receipt.after.worktree
    {
        Some("shared_worktree_retained")
    } else {
        Some("legacy_runtime")
    }
}

fn historical_digest_matches(
    historical: &serde_json::Value,
    field: &str,
    expected: &Digest,
) -> bool {
    historical
        .get(field)
        .and_then(serde_json::Value::as_str)
        .and_then(|value| value.parse::<Digest>().ok())
        .is_some_and(|actual| actual == *expected)
}

fn ensure_resource_finalization_base_binding(
    receipt: &ResourceFinalizationReceipt,
    contract: &Contract,
    path: &Path,
) -> Result<(), ObserverError> {
    if receipt.pull_request.base_revision == contract.base_revision {
        return Ok(());
    }
    let direct_merge = receipt.historical.as_ref().filter(|historical| {
        matches!(
            historical.kind,
            cockpit_protocol::HistoricalFinalizationKind::DirectMergeNoPr
        )
    });
    if let Some(historical) = direct_merge {
        if historical.contract_base_revision.as_deref() == Some(contract.base_revision.as_str()) {
            // `pullRequest.baseRevision` is the real first parent of the
            // historical merge. The immutable Contract base is bound
            // separately so a bundled merge can be recorded without
            // rewriting either fact or inventing a PR.
            return Ok(());
        }
        return Err(ObserverError::State {
            path: path.into(),
            message: format!(
                "historical direct-merge Contract base mismatch: receipt must bind historical.contractBaseRevision={} while pullRequest.baseRevision remains the real merge first parent ({})",
                contract.base_revision, receipt.pull_request.base_revision
            ),
        });
    }
    Err(ObserverError::State {
        path: path.into(),
        message: "resource finalization pull request base revision does not match the archived Contract base revision".into(),
    })
}

/// Validate the additional facts that make a historical compatibility receipt
/// honest.  The protocol validates the typed shape; this repository-bound
/// check binds a direct merge to the actual Git commit/parents and restricts
/// shared-worktree history to the repository's primary checkout.
fn validate_historical_finalization(
    root: &Path,
    receipt: &ResourceFinalizationReceipt,
    path: &Path,
) -> Result<(), ObserverError> {
    let Some(historical) = receipt.historical.as_ref() else {
        return Ok(());
    };
    match historical.kind {
        HistoricalFinalizationKind::SharedWorktreeRetained => {
            let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
                path: root.into(),
                source,
            })?;
            let worktree = fs::canonicalize(&receipt.worktree.path).map_err(|source| {
                ObserverError::State {
                    path: path.into(),
                    message: format!("historical shared worktree is not present: {source}"),
                }
            })?;
            if worktree != root || receipt.worktree.branch != receipt.branch.name {
                return Err(ObserverError::State {
                    path: path.into(),
                    message: "historical shared-worktree receipt is not bound to the primary repository worktree".into(),
                });
            }
        }
        HistoricalFinalizationKind::DirectMergeNoPr => {
            let merge_commit = receipt
                .historical
                .as_ref()
                .and_then(|value| value.merge_commit.as_deref())
                .ok_or_else(|| ObserverError::State {
                    path: path.into(),
                    message: "historical direct-merge receipt is missing merge commit".into(),
                })?;
            let observation = git_text(root, &["rev-list", "--parents", "-n", "1", merge_commit])
                .ok_or_else(|| ObserverError::State {
                path: path.into(),
                message: "historical direct-merge commit is not present in this repository".into(),
            })?;
            let parts = observation.split_whitespace().collect::<Vec<_>>();
            let parents = receipt
                .historical
                .as_ref()
                .map(|value| {
                    value
                        .merge_parents
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if parts.first().copied() != Some(merge_commit)
                || parts.get(1..).unwrap_or_default() != parents.as_slice()
                || parents.len() < 2
            {
                return Err(ObserverError::State {
                    path: path.into(),
                    message: "historical direct-merge commit parents do not match Git".into(),
                });
            }
            let base = receipt
                .historical
                .as_ref()
                .map(|value| value.base_revision.as_str())
                .unwrap_or_default();
            if parents.first().copied() != Some(base) {
                return Err(ObserverError::State {
                    path: path.into(),
                    message: format!(
                        "historical direct-merge base must equal Git's first parent: expected {}, receipt has {}",
                        parents.first().copied().unwrap_or("<missing>"),
                        base
                    ),
                });
            }
            let ancestor = Command::new("git")
                .arg("-C")
                .arg(root)
                .args(["merge-base", "--is-ancestor", base, merge_commit])
                .status()
                .map(|status| status.success())
                .unwrap_or(false);
            if !ancestor {
                return Err(ObserverError::State {
                    path: path.into(),
                    message: "historical direct-merge base is not an ancestor of merge commit"
                        .into(),
                });
            }
        }
    }
    Ok(())
}

/// Infer the narrow compatibility classification for a legacy receipt that
/// predates the explicit `historical` field.  This is deliberately stricter
/// than merely seeing `disposition=retained`: the receipt must prove that the
/// provider was the repository-local/shared mode, both the Contract and the
/// receipt point at the canonical primary checkout, and the branch/worktree
/// remained present and unchanged.  A linked worktree or an external provider
/// is never accepted by this projection.
fn infer_legacy_shared_worktree_retained(
    root: &Path,
    receipt: &ResourceFinalizationReceipt,
    contract: &Contract,
) -> bool {
    if receipt.historical.is_some()
        || !matches!(
            receipt.result.disposition,
            ResourceFinalizationDisposition::Retained
        )
        || receipt.provider != "local"
        || !matches!(
            receipt.before.pull_request,
            cockpit_protocol::ResourceFinalizationPullRequestState::Merged
        )
        || !matches!(
            receipt.after.pull_request,
            cockpit_protocol::ResourceFinalizationPullRequestState::Merged
        )
        || receipt.before.branch != receipt.after.branch
        || receipt.before.worktree != receipt.after.worktree
        || !matches!(
            receipt.after.branch,
            cockpit_protocol::ResourceFinalizationBranchState::Present
        )
        || !matches!(
            receipt.after.worktree,
            cockpit_protocol::ResourceFinalizationWorktreeState::Clean
        )
        || receipt.branch.name != receipt.worktree.branch
    {
        return false;
    }
    let Some(context) = receipt.resource_context.as_ref() else {
        return false;
    };
    if context.provider != "local"
        || contract.resource_context.as_ref() != Some(context)
        || context.branch != receipt.branch.name
        || context.worktree != receipt.worktree.path
    {
        return false;
    }
    let Ok(root) = fs::canonicalize(root) else {
        return false;
    };
    let Ok(layout) = discover_worktree_layout(&root) else {
        return false;
    };
    if layout.primary != root {
        return false;
    }
    fs::canonicalize(&context.worktree).ok().as_ref() == Some(&root)
        && fs::canonicalize(&receipt.worktree.path).ok().as_ref() == Some(&root)
}

fn historical_finalization_recovery_path(root: &Path, work_item_id: &str) -> PathBuf {
    root.join(".ai/decisions")
        .join(format!("{work_item_id}.finalize-recovery.json"))
}

fn read_historical_finalization_recovery(
    path: &Path,
) -> Result<HistoricalFinalizationRecoveryReceipt, ObserverError> {
    if !is_regular_non_symlink(path)? {
        return Err(ObserverError::State {
            path: path.into(),
            message: "historical finalization recovery must be a regular non-symlink file".into(),
        });
    }
    let bytes = fs::read(path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })?;
    if bytes.len() > MAX_EXTERNAL_EVIDENCE_BYTES {
        return Err(ObserverError::State {
            path: path.into(),
            message: "historical finalization recovery exceeds the bounded size limit".into(),
        });
    }
    reject_duplicate_json_keys(&bytes).map_err(|message| ObserverError::State {
        path: path.into(),
        message: format!("invalid historical finalization recovery JSON: {message}"),
    })?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
            path: path.into(),
            message: format!("invalid historical finalization recovery JSON: {error}"),
        })?;
    serde_json::from_value(value).map_err(|error| ObserverError::State {
        path: path.into(),
        message: format!("invalid historical finalization recovery: {error}"),
    })
}

fn validate_historical_finalization_recovery_binding(
    root: &Path,
    work_item_id: &str,
    recovery: &HistoricalFinalizationRecoveryReceipt,
    receipt: &ResourceFinalizationReceipt,
    contract: &Contract,
    recovery_path: &Path,
    current_runtime: &RuntimeContext,
) -> Result<(), ObserverError> {
    validate_historical_finalization_recovery(recovery).map_err(|error| ObserverError::State {
        path: recovery_path.into(),
        message: error.to_string(),
    })?;
    let expected_repository_id = repository_id(root).to_string();
    if recovery.work_item_id != work_item_id
        || recovery.repository_id != expected_repository_id
        || receipt.work_item_id != work_item_id
        || receipt.repository_id != expected_repository_id
    {
        return Err(ObserverError::State {
            path: recovery_path.into(),
            message: "historical finalization recovery repository or Work Item identity mismatch"
                .into(),
        });
    }
    let expected_path = repository_relative_path(
        root,
        &resource_finalization_decision_path(root, work_item_id),
    );
    if recovery.predecessor_path != expected_path {
        return Err(ObserverError::State {
            path: recovery_path.into(),
            message: "historical finalization recovery predecessor path mismatch".into(),
        });
    }
    let predecessor_path = root.join(&recovery.predecessor_path);
    let predecessor_value =
        serde_json::to_value(receipt).map_err(|error| ObserverError::State {
            path: predecessor_path.clone(),
            message: error.to_string(),
        })?;
    let predecessor_digest =
        cockpit_protocol::digest_json(&predecessor_value).map_err(|error| {
            ObserverError::State {
                path: predecessor_path.clone(),
                message: error.to_string(),
            }
        })?;
    if recovery.predecessor_receipt_digest != predecessor_digest {
        return Err(ObserverError::State {
            path: recovery_path.into(),
            message: "historical finalization recovery predecessor digest mismatch".into(),
        });
    }
    if recovery.base_revision != contract.base_revision
        || receipt.pull_request.base_revision != contract.base_revision
    {
        return Err(ObserverError::State {
            path: recovery_path.into(),
            message: "historical finalization recovery base revision mismatch".into(),
        });
    }
    if recovery.runtime_version != current_runtime.runtime_version
        || recovery.runtime_digest != current_runtime.runtime_digest
    {
        return Err(ObserverError::State {
            path: recovery_path.into(),
            message: "historical finalization recovery Runtime identity does not match the executing Runtime".into(),
        });
    }
    if receipt.runtime_version == current_runtime.runtime_version
        && receipt.runtime_digest == current_runtime.runtime_digest
    {
        return Err(ObserverError::State {
            path: recovery_path.into(),
            message: "historical finalization recovery requires an older predecessor Runtime"
                .into(),
        });
    }
    match recovery.historical_kind {
        HistoricalFinalizationKind::SharedWorktreeRetained => {
            if !matches!(
                receipt.result.disposition,
                ResourceFinalizationDisposition::Retained
            ) {
                return Err(ObserverError::State {
                    path: recovery_path.into(),
                    message: "shared-worktree historical recovery requires retained disposition"
                        .into(),
                });
            }
            let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
                path: root.into(),
                source,
            })?;
            let worktree = fs::canonicalize(&receipt.worktree.path).map_err(|source| {
                ObserverError::State {
                    path: recovery_path.into(),
                    message: format!("historical shared worktree is not present: {source}"),
                }
            })?;
            if worktree != root || receipt.worktree.branch != receipt.branch.name {
                return Err(ObserverError::State {
                    path: recovery_path.into(),
                    message: "historical shared-worktree recovery is not bound to the primary repository worktree".into(),
                });
            }
        }
        HistoricalFinalizationKind::DirectMergeNoPr => {
            return Err(ObserverError::State {
                path: recovery_path.into(),
                message: "direct-merge history must be recorded as a complete historical finalization receipt, not a reclassification of a PR receipt".into(),
            });
        }
    }
    Ok(())
}

fn load_historical_finalization_recovery(
    root: &Path,
    work_item_id: &str,
    receipt: &ResourceFinalizationReceipt,
    contract: &Contract,
    current_runtime: &RuntimeContext,
) -> Result<Option<HistoricalFinalizationRecoveryReceipt>, ObserverError> {
    let path = historical_finalization_recovery_path(root, work_item_id);
    if fs::symlink_metadata(&path).is_err() {
        return Ok(None);
    }
    let recovery = read_historical_finalization_recovery(&path)?;
    validate_historical_finalization_recovery_binding(
        root,
        work_item_id,
        &recovery,
        receipt,
        contract,
        &path,
        current_runtime,
    )?;
    Ok(Some(recovery))
}

/// Record an explicit Runtime-bound classification for a legacy finalization
/// receipt. The predecessor remains byte-for-byte immutable; the new record
/// is accepted only when it binds the exact predecessor digest and the
/// current Runtime identity.
pub fn record_historical_finalization_recovery(
    root: &Path,
    work_item_id: &str,
    input_path: &Path,
    runtime: &RuntimeContext,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let _lifecycle_lock = acquire_lifecycle_lock(&root, work_item_id)?;
    let (contract, contract_digest) = archived_contract_digest(&root, work_item_id)?;
    let predecessor_path = resource_finalization_decision_path(&root, work_item_id);
    if fs::symlink_metadata(&predecessor_path).is_err() {
        // A direct merge without a PR can be the first finalization record:
        // there is no immutable predecessor to classify.  Accept only a
        // complete, explicitly historical direct-merge receipt and route it
        // through the same strict archive/Contract/Git/runtime validation as
        // `finalize`; all other recovery inputs remain fail-closed.
        if let Ok(candidate) = read_resource_finalization_receipt(input_path)
            && matches!(
                candidate.historical.as_ref().map(|value| &value.kind),
                Some(HistoricalFinalizationKind::DirectMergeNoPr)
            )
        {
            return record_resource_finalization(&root, work_item_id, input_path, runtime);
        }
        return Err(ObserverError::State {
            path: predecessor_path,
            message: "historical finalization recovery requires an existing predecessor; for a first-record direct merge use a complete direct_merge_no_pr receipt".into(),
        });
    }
    let receipt = read_resource_finalization_receipt(&predecessor_path)?;
    validate_resource_finalization_receipt_for(
        &receipt,
        &contract.repository_id,
        work_item_id,
        Some(&contract_digest),
        contract.resource_context.as_ref(),
    )
    .map_err(|error| ObserverError::State {
        path: predecessor_path.clone(),
        message: error.to_string(),
    })?;
    let recovery = read_historical_finalization_recovery(input_path)?;
    validate_historical_finalization_recovery_binding(
        &root,
        work_item_id,
        &recovery,
        &receipt,
        &contract,
        input_path,
        runtime,
    )?;
    let recovery_path = historical_finalization_recovery_path(&root, work_item_id);
    let value = serde_json::to_value(&recovery).map_err(|error| ObserverError::State {
        path: recovery_path.clone(),
        message: error.to_string(),
    })?;
    if fs::symlink_metadata(&recovery_path).is_ok() {
        if !is_regular_non_symlink(&recovery_path)? {
            return Err(ObserverError::State {
                path: recovery_path,
                message: "historical finalization recovery destination is not a regular file"
                    .into(),
            });
        }
        let existing = read_json(&recovery_path)?;
        if existing == value {
            return Ok(serde_json::json!({
                "workItemId": work_item_id,
                "state": "idempotent",
                "path": repository_relative_path(&root, &recovery_path)
            }));
        }
        return Err(ObserverError::State {
            path: recovery_path,
            message: "historical finalization recovery already exists with different content"
                .into(),
        });
    }
    atomic_json(&recovery_path, &value)?;
    Ok(serde_json::json!({
        "workItemId": work_item_id,
        "state": "recorded",
        "historicalKind": recovery.historical_kind,
        "assurance": recovery.assurance,
        "path": repository_relative_path(&root, &recovery_path)
    }))
}

/// Produce a read-only, fact-bound recovery plan for a legacy finalization.
/// The plan deliberately contains no generated human authority or decision;
/// it only supplies immutable predecessor facts and, when explicitly given,
/// Git's real direct-merge parents.
pub fn historical_finalization_recovery_plan(
    root: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
    merge_commit: Option<&str>,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let repository_id = repository_id(&root).to_string();
    let mut result = serde_json::json!({
        "workItemId": work_item_id,
        "repositoryId": repository_id,
        "runtimeVersion": runtime.runtime_version,
        "runtimeDigest": runtime.runtime_digest,
        "state": "needs_human_review",
        "writesRepositoryState": false,
        "humanInputRequired": ["authoritySource", "reason", "decidedAt"],
    });

    // An archived Contract is read-only input to the plan.  Expose its digest
    // and immutable base so a human can bind the eventual receipt without
    // guessing which historical snapshot the Work Item used.  Provisional
    // resource-context values are reported as facts, never promoted to
    // concrete provider identity.
    let archived_contract = archived_contract_digest(&root, work_item_id).ok();
    if let Some((contract, digest)) = archived_contract.as_ref() {
        result["contractDigest"] = digest.to_string().into();
        result["contractBaseRevision"] = contract.base_revision.clone().into();
        if let Some(context) = contract.resource_context.as_ref() {
            result["contractResourceContext"] =
                serde_json::to_value(context).map_err(|error| ObserverError::State {
                    path: root.clone(),
                    message: error.to_string(),
                })?;
        }
    }

    if let Ok((receipt, path, digest, sequence)) =
        resolve_resource_finalization_head(&root, work_item_id)
    {
        let contract = archived_contract_digest(&root, work_item_id).ok();
        let kind = receipt
            .historical
            .as_ref()
            .map(|historical| match historical.kind {
                HistoricalFinalizationKind::SharedWorktreeRetained => "shared_worktree_retained",
                HistoricalFinalizationKind::DirectMergeNoPr => "direct_merge_no_pr",
            })
            .or_else(|| {
                contract.as_ref().and_then(|(contract, _)| {
                    infer_legacy_shared_worktree_retained(&root, &receipt, contract)
                        .then_some("shared_worktree_retained")
                })
            });
        let stale = receipt.runtime_version != runtime.runtime_version
            || receipt.runtime_digest != runtime.runtime_digest;
        let closed = stale
            && contract.as_ref().is_some_and(|(contract, _)| {
                closed_finalization_projection_kind(
                    &root,
                    work_item_id,
                    &receipt,
                    &path,
                    &digest,
                    sequence,
                    &contract.repository_id,
                )
                .is_some()
            });
        result["predecessorPath"] = repository_relative_path(&root, &path).into();
        result["predecessorDigest"] = digest.to_string().into();
        result["sequence"] = sequence.into();
        result["predecessorRuntimeVersion"] = receipt.runtime_version.clone().into();
        result["predecessorRuntimeDigest"] = receipt.runtime_digest.to_string().into();
        result["historicalKind"] = kind
            .map(serde_json::Value::from)
            .unwrap_or(serde_json::Value::Null);
        result["baseRevision"] = receipt.pull_request.base_revision.clone().into();
        if closed {
            result["state"] = "already_closed_historical".into();
            result["assurance"] = "historical_low".into();
            result["humanInputRequired"] = serde_json::json!([]);
        } else if !stale {
            result["state"] = "current_runtime_no_recovery_required".into();
            result["humanInputRequired"] = serde_json::json!([]);
        } else {
            result["suggestedRecovery"] = serde_json::json!({
                "kind": "historical_finalization_recovery",
                "historicalKind": kind,
                "assurance": "historical_low",
                "workItemId": work_item_id,
                "repositoryId": repository_id,
                "predecessorPath": repository_relative_path(&root, &path),
                "predecessorReceiptDigest": digest,
                "baseRevision": receipt.pull_request.base_revision,
                "runtimeVersion": runtime.runtime_version,
                "runtimeDigest": runtime.runtime_digest,
            });
        }
        return Ok(result);
    }

    let Some(merge_commit) = merge_commit else {
        result["state"] = "predecessor_missing_or_unreadable".into();
        result["safeAction"] =
            "provide a real merge commit for historical direct-merge inspection".into();
        return Ok(result);
    };
    let parents = git_text(&root, &["rev-list", "--parents", "-n", "1", merge_commit])
        .ok_or_else(|| ObserverError::State {
            path: root.clone(),
            message: "cannot inspect historical direct-merge commit".into(),
        })?
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if parents.len() < 3 {
        return Err(ObserverError::State {
            path: root,
            message: "historical direct-merge commit must have at least two parents".into(),
        });
    }
    result["state"] = "direct_merge_candidate".into();
    result["historicalKind"] = "direct_merge_no_pr".into();
    result["assurance"] = "historical_low".into();
    result["mergeCommit"] = merge_commit.into();
    result["mergeParents"] =
        serde_json::to_value(&parents[1..]).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    result["baseRevision"] = parents[1].clone().into();
    let head_revision = parents[2].clone();
    result["knownFacts"] = serde_json::json!({
        "schemaVersion": 1,
        "workItemId": work_item_id,
        "repositoryId": repository_id,
        "runtimeVersion": runtime.runtime_version,
        "runtimeDigest": runtime.runtime_digest,
        "contractBaseRevision": archived_contract
            .as_ref()
            .map(|(contract, _)| contract.base_revision.clone()),
        "pullRequest": {
            "number": 0,
            "url": format!("historical://direct-merge/{merge_commit}"),
            "headRevision": head_revision,
            "baseRevision": parents[1],
            "mergeCommit": merge_commit
        },
        "historical": {
            "kind": "direct_merge_no_pr",
            "assurance": "historical_low",
            "baseRevision": parents[1],
            "mergeCommit": merge_commit,
            "mergeParents": &parents[1..]
        }
    });
    if let Some((contract, _)) = archived_contract.as_ref() {
        result["knownFacts"]["historical"]["contractBaseRevision"] =
            contract.base_revision.clone().into();
    }
    if let Some((_, digest)) = archived_contract.as_ref() {
        result["knownFacts"]["contractDigest"] = digest.to_string().into();
    }
    if let Some((contract, _)) = archived_contract.as_ref()
        && let Some(context) = contract.resource_context.as_ref()
        && !context.is_provisional()
    {
        result["knownFacts"]["pullRequest"]["baseBranch"] = context.base_branch.clone().into();
        result["knownFacts"]["pullRequest"]["baseRemote"] = context.base_remote.clone().into();
    }
    // The merge commit proves that the historical operation was merged, but
    // it does not prove whether the old branch/worktree was later removed.
    // Emit that distinction explicitly instead of asking the caller to
    // reconstruct a partially typed receipt.  `retained` plus a stable
    // unknown code is the conservative closeable projection for historical
    // low-assurance records; a human may replace it with `deleted` only when
    // fresh cleanup evidence proves the stronger claim.
    let historical_url = format!("historical://direct-merge/{merge_commit}");
    let context = archived_contract
        .as_ref()
        .and_then(|(contract, _)| contract.resource_context.as_ref())
        .filter(|context| !context.is_provisional());
    let mut human_input_required = vec![
        "actor".to_owned(),
        "authoritySource".to_owned(),
        "reason".to_owned(),
        "timestamp".to_owned(),
    ];
    let mut suggested_receipt = serde_json::json!({
        "schemaVersion": 1,
        "receiptId": format!("historical-direct-merge-{work_item_id}-{merge_commit}"),
        "operationId": format!("historical-direct-merge-operation-{merge_commit}"),
        "runtimeVersion": runtime.runtime_version,
        "runtimeDigest": runtime.runtime_digest,
        "pullRequest": {
            "number": 0,
            "url": historical_url,
            "headRevision": parents[2],
            "baseBranch": "unknown",
            "baseRemote": "unknown",
            "baseRevision": parents[1],
            "mergeCommit": merge_commit
        },
        "provider": "historical",
        "branch": {
            "name": "unknown",
            "remote": "unknown",
            "headRevision": parents[2]
        },
        "worktree": {
            "worktreeId": "unknown",
            "path": "unknown",
            "branch": "unknown",
            "headRevision": parents[2]
        },
        "before": {
            "pullRequest": "merged",
            "branch": "unknown",
            "worktree": "unknown"
        },
        "after": {
            "pullRequest": "merged",
            "branch": "unknown",
            "worktree": "unknown"
        },
        "result": {
            "disposition": "retained",
            "failureCodes": [],
            "unknownCodes": ["historical_resource_state_unknown"]
        },
        "historical": {
            "kind": "direct_merge_no_pr",
            "assurance": "historical_low",
            "baseRevision": parents[1],
            "mergeCommit": merge_commit,
            "mergeParents": &parents[1..],
            "contractBaseRevision": archived_contract
                .as_ref()
                .map(|(contract, _)| contract.base_revision.clone())
        },
        "workItemId": work_item_id,
        "repositoryId": repository_id
    });
    if let Some(context) = context {
        suggested_receipt["pullRequest"]["baseBranch"] = context.base_branch.clone().into();
        suggested_receipt["pullRequest"]["baseRemote"] = context.base_remote.clone().into();
        suggested_receipt["branch"] = serde_json::json!({
            "name": context.branch,
            "remote": context.base_remote,
            "headRevision": parents[2]
        });
        suggested_receipt["worktree"] = serde_json::json!({
            "worktreeId": format!("historical-direct-merge-{merge_commit}"),
            "path": context.worktree,
            "branch": context.branch,
            "headRevision": parents[2]
        });
        suggested_receipt["resourceContext"] = serde_json::json!({
            "branch": context.branch,
            "worktree": context.worktree,
            "baseBranch": context.base_branch,
            "baseRemote": context.base_remote,
            "provider": "historical",
            "pullRequest": historical_url,
        });
    } else {
        human_input_required.extend([
            "pullRequest.baseBranch".to_owned(),
            "pullRequest.baseRemote".to_owned(),
            "resourceContext".to_owned(),
            "branch.name".to_owned(),
            "branch.remote".to_owned(),
            "worktree.worktreeId".to_owned(),
            "worktree.path".to_owned(),
            "worktree.branch".to_owned(),
        ]);
    }
    result["humanInputRequired"] = human_input_required.into();
    result["suggestedReceipt"] = suggested_receipt;
    if let Some((_, digest)) = archived_contract.as_ref() {
        result["suggestedReceipt"]["contractDigest"] = digest.to_string().into();
    }
    Ok(result)
}

/// Persist a provider-side finalization receipt after strict identity and
/// local postcondition validation.  The Runtime never calls a provider or
/// deletes a branch implicitly; it records delegated evidence and refuses
/// close on blocked/unknown/contradictory results.
pub fn record_resource_finalization(
    root: &Path,
    work_item_id: &str,
    receipt_path: &Path,
    runtime: &RuntimeContext,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let close_path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    let close_present = fs::symlink_metadata(&close_path).is_ok();
    let manifest_path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.archive.json"));
    let manifest = read_json(&manifest_path)?;
    verify_archive_manifest(&root, work_item_id, &manifest)?;
    let (contract, contract_digest) = archived_contract_digest(&root, work_item_id)?;
    let input_value = read_json(receipt_path)?;
    let transition = input_value
        .get("receipt")
        .map(|_| read_resource_finalization_transition(receipt_path))
        .transpose()?;
    let receipt = if let Some(transition) = &transition {
        transition.receipt.clone()
    } else {
        read_resource_finalization_receipt(receipt_path)?
    };
    if close_present && transition.is_none() {
        return Err(ObserverError::State {
            path: close_path.clone(),
            message: "resource finalization reconciliation after close requires an append-only transition".into(),
        });
    }
    if transition.is_none() {
        validate_resource_finalization_receipt_for(
            &receipt,
            &contract.repository_id,
            work_item_id,
            Some(&contract_digest),
            contract.resource_context.as_ref(),
        )
        .map_err(|error| ObserverError::State {
            path: receipt_path.into(),
            message: error.to_string(),
        })?;
    }
    validate_historical_finalization(&root, &receipt, receipt_path)?;
    ensure_resource_finalization_base_binding(&receipt, &contract, receipt_path)?;
    if receipt.provider == "unknown"
        || receipt
            .resource_context
            .as_ref()
            .is_some_and(|context| context.provider == "unknown")
    {
        return Err(ObserverError::State {
            path: receipt_path.into(),
            message: "resource finalization requires an identified external provider".into(),
        });
    }
    // A post-close transition repairs an immutable receipt emitted by an older
    // Runtime.  Bind it to the predecessor's Runtime identity below instead
    // of rejecting a valid historical chain merely because the validator was
    // upgraded.  New canonical receipts and pre-close transitions still must
    // match the executing Runtime.
    if !(close_present && transition.is_some()) {
        ensure_resource_runtime_identity(&receipt, runtime, receipt_path)?;
    }
    if matches!(
        receipt.result.disposition,
        ResourceFinalizationDisposition::Deleted | ResourceFinalizationDisposition::Abandoned
    ) && !local_resources_deleted(&root, &receipt)?
    {
        return Err(ObserverError::State {
            path: receipt_path.into(),
            message:
                "finalization receipt does not match local branch/worktree cleanup postconditions"
                    .into(),
        });
    }
    let decision_path = resource_finalization_decision_path(&root, work_item_id);
    if decision_path.exists() {
        let (existing, head_path, _head_digest, sequence) =
            resolve_resource_finalization_head(&root, work_item_id)?;
        if let Some(transition) = transition {
            if close_present {
                validate_post_close_finalization_reconciliation(
                    &root,
                    work_item_id,
                    &close_path,
                    &existing,
                    &head_path,
                    sequence,
                    &transition,
                )?;
            }
            validate_resource_finalization_transition(&existing, &transition, sequence + 1)
                .map_err(|error| ObserverError::State {
                    path: receipt_path.into(),
                    message: error.to_string(),
                })?;
            validate_governance_append_revision(
                &root,
                work_item_id,
                &existing,
                &transition,
                receipt_path,
            )?;
            let value =
                serde_json::to_value(&transition).map_err(|error| ObserverError::State {
                    path: receipt_path.into(),
                    message: error.to_string(),
                })?;
            let transition_digest =
                cockpit_protocol::digest_json(&value).map_err(|error| ObserverError::State {
                    path: receipt_path.into(),
                    message: error.to_string(),
                })?;
            let suffix = transition_digest.to_string();
            let appended_path = root.join(".ai/decisions").join(format!(
                "{work_item_id}.finalize.{}.json",
                suffix.strip_prefix("sha256:").unwrap_or(&suffix)
            ));
            if appended_path.exists() {
                let existing_value = read_json(&appended_path)?;
                if existing_value == value {
                    return Ok(
                        serde_json::json!({"workItemId": work_item_id, "state": "idempotent", "disposition": receipt.result.disposition, "path": repository_relative_path(&root, &appended_path)}),
                    );
                }
                return Err(ObserverError::State {
                    path: appended_path,
                    message: "resource finalization transition digest collision".into(),
                });
            }
            atomic_json(&appended_path, &value)?;
            return Ok(serde_json::json!({
                "workItemId": work_item_id,
                "state": "appended",
                "sequence": transition.sequence,
                "disposition": receipt.result.disposition,
                "predecessorPath": repository_relative_path(&root, &head_path),
                "path": repository_relative_path(&root, &appended_path)
            }));
        }
        validate_resource_finalization_replay(&existing, &receipt).map_err(|error| {
            ObserverError::State {
                path: head_path.clone(),
                message: error.to_string(),
            }
        })?;
        return Ok(serde_json::json!({
            "workItemId": work_item_id,
            "state": "idempotent",
            "disposition": existing.result.disposition,
            "path": repository_relative_path(&root, &head_path)
        }));
    }
    fs::create_dir_all(decision_path.parent().unwrap_or(root.as_path())).map_err(|source| {
        ObserverError::Read {
            path: decision_path.clone(),
            source,
        }
    })?;
    let value = serde_json::to_value(&receipt).map_err(|error| ObserverError::State {
        path: decision_path.clone(),
        message: error.to_string(),
    })?;
    atomic_json(&decision_path, &value)?;
    Ok(serde_json::json!({
        "workItemId": work_item_id,
        "state": "recorded",
        "disposition": receipt.result.disposition,
        "path": repository_relative_path(&root, &decision_path)
    }))
}

/// A close receipt is immutable, but an older Runtime could record close
/// while the provider-side finalization receipt was still retained.  Permit
/// exactly one append-only cleanup transition for that legacy case: the
/// close must bind the current finalization head, and the new transition must
/// be the next sequence with a fully deleted result.  New closes are blocked
/// before this path by `require_resource_finalization_for_close`.
fn validate_post_close_finalization_reconciliation(
    root: &Path,
    work_item_id: &str,
    close_path: &Path,
    previous: &ResourceFinalizationReceipt,
    previous_path: &Path,
    previous_sequence: u64,
    transition: &ResourceFinalizationTransitionReceipt,
) -> Result<(), ObserverError> {
    let close_metadata =
        fs::symlink_metadata(close_path).map_err(|source| ObserverError::Read {
            path: close_path.into(),
            source,
        })?;
    if !close_metadata.is_file() || close_metadata.file_type().is_symlink() {
        return Err(ObserverError::State {
            path: close_path.into(),
            message: "post-close finalization reconciliation requires a regular close receipt"
                .into(),
        });
    }
    let close = read_json(close_path)?;
    if close["state"] != serde_json::json!("closed")
        || close["workItemId"] != serde_json::json!(work_item_id)
        || close["repositoryId"] != serde_json::json!(repository_id(root).to_string())
        || close["decisionState"] != serde_json::json!("confirmed")
        || close["humanDecision"] != serde_json::json!("approved")
        || close["resourceFinalizationSequence"] != serde_json::json!(previous_sequence)
        || close["resourceFinalizationHeadPath"]
            != serde_json::json!(repository_relative_path(root, previous_path))
    {
        return Err(ObserverError::State {
            path: close_path.into(),
            message: "post-close finalization reconciliation is not bound to the closed head"
                .into(),
        });
    }
    let previous_value = serde_json::to_value(previous).map_err(|error| ObserverError::State {
        path: previous_path.into(),
        message: error.to_string(),
    })?;
    let previous_digest =
        cockpit_protocol::digest_json(&previous_value).map_err(|error| ObserverError::State {
            path: previous_path.into(),
            message: error.to_string(),
        })?;
    if transition.receipt.runtime_version != previous.runtime_version
        || transition.receipt.runtime_digest != previous.runtime_digest
    {
        return Err(ObserverError::State {
            path: close_path.into(),
            message:
                "post-close reconciliation Runtime identity must match the historical predecessor"
                    .into(),
        });
    }
    if close["resourceFinalizationHeadDigest"] != serde_json::json!(previous_digest.to_string())
        || transition.sequence != previous_sequence + 1
        || !matches!(
            transition.receipt.result.disposition,
            ResourceFinalizationDisposition::Deleted
        )
    {
        return Err(ObserverError::State {
            path: close_path.into(),
            message: "post-close reconciliation must append the next deleted finalization head"
                .into(),
        });
    }
    Ok(())
}

fn local_resources_deleted(
    root: &Path,
    receipt: &ResourceFinalizationReceipt,
) -> Result<bool, ObserverError> {
    let branches = git_text(root, &["branch", "--format=%(refname:short)"]).ok_or_else(|| {
        ObserverError::State {
            path: root.to_path_buf(),
            message: "cannot determine local branch state".into(),
        }
    })?;
    if branches
        .lines()
        .any(|branch| branch.trim() == receipt.branch.name)
    {
        return Ok(false);
    }
    let worktrees = git_text(root, &["worktree", "list", "--porcelain"]).ok_or_else(|| {
        ObserverError::State {
            path: root.to_path_buf(),
            message: "cannot determine local worktree state".into(),
        }
    })?;
    if worktrees.lines().any(|line| {
        line.strip_prefix("worktree ")
            .is_some_and(|path| path == receipt.worktree.path)
    }) {
        return Ok(false);
    }
    Ok(true)
}

/// Revalidate a stored finalization receipt and local cleanup postconditions.
pub fn verify_resource_finalization(
    root: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> Result<serde_json::Value, ObserverError> {
    verify_resource_finalization_internal(root, work_item_id, Some(runtime))
}

fn verify_resource_finalization_internal(
    root: &Path,
    work_item_id: &str,
    runtime: Option<&RuntimeContext>,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let (receipt, path, receipt_digest, sequence) =
        resolve_resource_finalization_head(&root, work_item_id).map_err(|error| {
            finalization_observation_error(
                resource_finalization_decision_path(&root, work_item_id),
                FinalizationErrorCode::RecordCorrupt,
                error,
            )
        })?;
    let (contract, finalization_contract_digest) = archived_contract_digest(&root, work_item_id)
        .map_err(|error| {
            finalization_observation_error(
                root.join(".ai/work-items/archive")
                    .join(format!("{work_item_id}.contract.json")),
                FinalizationErrorCode::RecordCorrupt,
                error,
            )
        })?;
    let current_contract_canonical_digest = contract_digest(
        &root
            .join(".ai/work-items/archive")
            .join(format!("{work_item_id}.contract.json")),
    )?;
    let contract_amendment_revalidation = runtime
        .and_then(|_| {
            load_recovery_decision(&root, work_item_id, None)
                .ok()
                .flatten()
        })
        .filter(|decision| {
            decision.successor_binding_mode.as_deref() == Some("contract_amendment_revalidation")
                && decision.current_contract_digest.as_ref()
                    == Some(&current_contract_canonical_digest)
                && decision.current_contract_digest.as_ref()
                    != Some(&decision.predecessor_contract_digest)
                && receipt.contract_digest.as_ref()
                    == Some(
                        decision
                            .predecessor_finalization_contract_digest
                            .as_ref()
                            .unwrap_or(&decision.predecessor_contract_digest),
                    )
        })
        .is_some();
    // A Contract-amendment successor is an explicit current revalidation of
    // the amended Contract. Once that successor has completed its own
    // verified close, the predecessor's provider finalization remains valid
    // historical evidence even when it was emitted by an older Runtime. This
    // projection is deliberately narrower than a general Runtime upgrade:
    // the recovery decision, current archive, historical evidence, successor,
    // and finalization head must all bind before the old Runtime identity is
    // tolerated.
    let contract_amendment_revalidation_resolved = contract_amendment_revalidation
        && recovery_successor_resolves_pending_close(&root, work_item_id, &contract.repository_id);
    let mut receipt_for_context_validation = receipt.clone();
    if sequence > 0
        && matches!(
            receipt.result.disposition,
            ResourceFinalizationDisposition::Retained
        )
        && matches!(
            receipt.before.pull_request,
            cockpit_protocol::ResourceFinalizationPullRequestState::Unmerged
        )
        && matches!(
            receipt.after.pull_request,
            cockpit_protocol::ResourceFinalizationPullRequestState::Merged
        )
        && receipt.before.branch == receipt.after.branch
        && receipt.before.worktree == receipt.after.worktree
    {
        // The resolver has already validated this member in transition
        // context. Normalize only the intrinsic precondition while binding
        // the unchanged repository, Contract, and resource identities below.
        receipt_for_context_validation.before.pull_request =
            cockpit_protocol::ResourceFinalizationPullRequestState::Merged;
    }
    validate_resource_finalization_receipt_for(
        &receipt_for_context_validation,
        &contract.repository_id,
        work_item_id,
        (!contract_amendment_revalidation).then_some(&finalization_contract_digest),
        contract.resource_context.as_ref(),
    )
    .map_err(|error| {
        finalization_observation_error(path.clone(), error.finalization_error_code(), error)
    })?;
    validate_historical_finalization(&root, &receipt, &path).map_err(|error| {
        finalization_observation_error(path.clone(), FinalizationErrorCode::RecordCorrupt, error)
    })?;
    ensure_resource_finalization_base_binding(&receipt, &contract, &path).map_err(|error| {
        finalization_observation_error(path.clone(), FinalizationErrorCode::BaseMismatch, error)
    })?;
    let inferred_legacy_shared_worktree =
        infer_legacy_shared_worktree_retained(&root, &receipt, &contract);
    let historical_recovery = if let Some(runtime) = runtime {
        load_historical_finalization_recovery(&root, work_item_id, &receipt, &contract, runtime)
            .map_err(|error| {
                finalization_observation_error(
                    historical_finalization_recovery_path(&root, work_item_id),
                    FinalizationErrorCode::HistoricalRecoveryRequired,
                    error,
                )
            })?
    } else {
        None
    };
    let historical_runtime_projection = if let Some(runtime) = runtime {
        if historical_recovery.is_none()
            && (receipt.runtime_version != runtime.runtime_version
                || receipt.runtime_digest != runtime.runtime_digest)
        {
            let kind = closed_finalization_projection_kind(
                &root,
                work_item_id,
                &receipt,
                &path,
                &receipt_digest,
                sequence,
                &contract.repository_id,
            )
            .or_else(|| {
                if fs::symlink_metadata(
                    root.join(".ai/decisions")
                        .join(format!("{work_item_id}.close.json")),
                )
                .is_ok()
                {
                    // Once a close record exists, its historical projection
                    // must be validated by `closed_finalization_projection_kind`.
                    // Do not fall back to the pre-close successor proof when
                    // the close binding itself is missing, malformed, or
                    // tampered.
                    None
                } else {
                    contract_amendment_revalidation_resolved
                        .then_some("contract_amendment_revalidation")
                }
            })
            .or_else(|| inferred_legacy_shared_worktree.then_some("shared_worktree_retained"));
            if kind.is_none()
                && let Err(error) = ensure_resource_runtime_identity(&receipt, runtime, &path)
            {
                return Err(finalization_observation_error(
                    path.clone(),
                    FinalizationErrorCode::RuntimeMismatch,
                    format!(
                        "{error}; inspect with `ai-cockpit work-item finalize-recovery-plan --repo <repository> --id {work_item_id}` before recording historical recovery"
                    ),
                ));
            }
            kind
        } else {
            None
        }
    } else {
        None
    };
    let resources_deleted = local_resources_deleted(&root, &receipt).map_err(|error| {
        finalization_observation_error(
            path.clone(),
            FinalizationErrorCode::ObservationUnavailable,
            error,
        )
    })?;
    if matches!(
        receipt.result.disposition,
        ResourceFinalizationDisposition::Deleted | ResourceFinalizationDisposition::Abandoned
    ) && !resources_deleted
    {
        return Err(finalization_observation_error(
            path,
            FinalizationErrorCode::CleanupPending,
            "resource finalization cleanup postconditions are not satisfied",
        ));
    }
    let mut result = serde_json::json!({
        "workItemId": work_item_id,
        "state": if historical_runtime_projection.is_some() {
            "historical_verified"
        } else {
            "verified"
        },
        "disposition": receipt.result.disposition,
        "sequence": sequence,
        "headPath": repository_relative_path(&root, &path),
        "headDigest": receipt_digest,
        "receipt": receipt
    });
    if let Some(recovery) = historical_recovery {
        result["historical"] = serde_json::json!(true);
        result["historicalKind"] =
            serde_json::to_value(recovery.historical_kind).map_err(|error| {
                ObserverError::State {
                    path: path.clone(),
                    message: error.to_string(),
                }
            })?;
        result["assurance"] = recovery.assurance.into();
        result["recoveryPath"] = repository_relative_path(
            &root,
            &historical_finalization_recovery_path(&root, work_item_id),
        )
        .into();
    } else if let Some(kind) = historical_runtime_projection {
        result["historical"] = serde_json::json!(true);
        result["historicalKind"] = kind.into();
        result["assurance"] = "historical_low".into();
        result["historicalReason"] =
            "closed predecessor receipt was verified under an older Runtime".into();
    } else if inferred_legacy_shared_worktree {
        result["historical"] = serde_json::json!(true);
        result["historicalKind"] = "shared_worktree_retained".into();
        result["assurance"] = "historical_low".into();
        result["historicalReason"] =
            "legacy local shared-worktree receipt was verified from repository-bound facts".into();
    } else if let Some(historical) = receipt.historical.as_ref() {
        result["historical"] = serde_json::json!(true);
        result["historicalKind"] =
            serde_json::to_value(&historical.kind).map_err(|error| ObserverError::State {
                path: path.clone(),
                message: error.to_string(),
            })?;
        result["assurance"] = historical.assurance.clone().into();
    }
    Ok(result)
}

pub fn close_work_item(root: &Path, work_item_id: &str) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    Err(ObserverError::State {
        path: root.join(".ai/decisions"),
        message: "close requires an explicit human decision".into(),
    })
}

pub fn close_work_item_with_decision(
    root: &Path,
    work_item_id: &str,
    human_decision: &str,
) -> Result<LifecycleReceipt, ObserverError> {
    if human_decision.trim().is_empty() {
        return Err(ObserverError::State {
            path: root.join(".ai/decisions"),
            message: "human decision must not be empty".into(),
        });
    }
    close_work_item_with_structured_decision(
        root,
        work_item_id,
        &HumanDecision {
            decision: human_decision.trim().into(),
            actor: "legacy-cli".into(),
            authority_source: "explicit-cli".into(),
            reason:
                "legacy human-decision input; provide structured fields for enterprise assurance"
                    .into(),
            evidence_refs: Vec::new(),
            policy_refs: Vec::new(),
            decided_at: now(),
            resume_condition: None,
        },
    )
}

pub fn close_work_item_with_decision_and_runtime(
    root: &Path,
    work_item_id: &str,
    human_decision: &str,
    runtime: &RuntimeContext,
) -> Result<LifecycleReceipt, ObserverError> {
    if human_decision.trim().is_empty() {
        return Err(ObserverError::State {
            path: root.join(".ai/decisions"),
            message: "human decision must not be empty".into(),
        });
    }
    close_work_item_with_structured_decision_and_runtime(
        root,
        work_item_id,
        &HumanDecision {
            decision: human_decision.trim().into(),
            actor: "legacy-cli".into(),
            authority_source: "explicit-cli".into(),
            reason:
                "legacy human-decision input; provide structured fields for enterprise assurance"
                    .into(),
            evidence_refs: Vec::new(),
            policy_refs: Vec::new(),
            decided_at: now(),
            resume_condition: None,
        },
        runtime,
    )
}

pub fn close_work_item_with_structured_decision(
    root: &Path,
    work_item_id: &str,
    human_decision: &HumanDecision,
) -> Result<LifecycleReceipt, ObserverError> {
    close_work_item_with_structured_decision_internal(root, work_item_id, human_decision, None)
}

pub fn close_work_item_with_structured_decision_and_runtime(
    root: &Path,
    work_item_id: &str,
    human_decision: &HumanDecision,
    runtime: &RuntimeContext,
) -> Result<LifecycleReceipt, ObserverError> {
    close_work_item_with_structured_decision_internal(
        root,
        work_item_id,
        human_decision,
        Some(runtime),
    )
}

fn close_work_item_with_structured_decision_internal(
    root: &Path,
    work_item_id: &str,
    human_decision: &HumanDecision,
    current_runtime: Option<&RuntimeContext>,
) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    for (field, value) in [
        ("decision", human_decision.decision.as_str()),
        ("actor", human_decision.actor.as_str()),
        ("authoritySource", human_decision.authority_source.as_str()),
        ("reason", human_decision.reason.as_str()),
        ("decidedAt", human_decision.decided_at.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(ObserverError::State {
                path: root.join(".ai/decisions"),
                message: format!("human decision field {field} must not be empty"),
            });
        }
    }
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let _lifecycle_lock = acquire_lifecycle_lock(&root, work_item_id)?;
    let archive = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.archive.json"));
    let manifest = read_json(&archive)?;
    // An archived predecessor may have a valid, append-only supersede
    // recovery decision recorded after the original archive.  The recovery
    // decision is the explicit authority for this transition; the immutable
    // archive manifest remains `archived` and is never rewritten merely to
    // make close succeed.
    let recovery_decision = load_recovery_decision(&root, work_item_id, current_runtime)?;
    let amendment_revalidation = recovery_decision.as_ref().is_some_and(|decision| {
        decision.decision == "successor"
            && decision.successor_binding_mode.as_deref() == Some("contract_amendment_revalidation")
    });
    // A Contract-amendment recovery is still non-terminal until its
    // repository-bound successor has completed its own lifecycle.  Once the
    // successor is closed, the predecessor can be closed as a historical
    // lineage record without pretending that the old evidence proves the
    // amended Contract.
    let amendment_revalidation_resolved = amendment_revalidation
        && recovery_successor_resolves_pending_close(
            &root,
            work_item_id,
            &repository_id(&root).to_string(),
        );
    let mut historical_archive_integrity = None;
    if let Err(manifest_error) = verify_archive_manifest(&root, work_item_id, &manifest) {
        let can_quarantine = recovery_decision
            .as_ref()
            .is_some_and(|decision| decision.decision == "supersede");
        if can_quarantine {
            if recovery_decision
                .as_ref()
                .and_then(|decision| decision.predecessor_archive_manifest_digest.as_ref())
                .is_some()
            {
                match verify_historical_supersede_manifest(
                    &root,
                    work_item_id,
                    &manifest,
                    recovery_decision
                        .as_ref()
                        .expect("supersede decision exists"),
                ) {
                    Ok(Some(artifact)) => historical_archive_integrity = Some(artifact),
                    Ok(None) => return Err(manifest_error),
                    Err(error) => return Err(error),
                }
            } else {
                return Err(manifest_error);
            }
        } else {
            return Err(manifest_error);
        }
    }
    let manifest_superseded = manifest["state"] == serde_json::json!("superseded");
    let superseded = manifest_superseded
        || recovery_decision
            .as_ref()
            .is_some_and(|decision| decision.decision == "supersede");
    if manifest_superseded
        && (!manifest["historicalEvidence"].as_bool().unwrap_or(false)
            || manifest["supersededBy"].as_str().is_none())
    {
        return Err(ObserverError::State {
            path: archive.clone(),
            message: "superseded archive manifest is missing historical binding".into(),
        });
    }
    let contract_path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    let summary_path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.summary.json"));
    let summary: serde_json::Value = read_json(&summary_path)?;
    let mut finalization_binding: Option<serde_json::Value> = None;
    if amendment_revalidation_resolved && contract.resource_context.is_some() {
        let finalization_path = resource_finalization_decision_path(&root, work_item_id);
        finalization_binding = Some(require_resource_finalization_for_close(
            &root,
            work_item_id,
            current_runtime,
        )?);
        if !finalization_path.exists() {
            return Err(ObserverError::State {
                path: finalization_path,
                message: "resolved Contract-amendment revalidation still requires provider finalization evidence".into(),
            });
        }
    }
    if (!superseded && summary["state"] != serde_json::json!("finish_ready"))
        || summary["checkpointCount"] != serde_json::json!(1)
        || (!superseded && summary["preflightState"] != serde_json::json!("green"))
    {
        return Err(ObserverError::State {
            path: summary_path,
            message:
                "close requires archived finish_ready state, one checkpoint, and green preflight"
                    .into(),
        });
    }
    if !superseded && !amendment_revalidation_resolved {
        let git =
            cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
                path: root.clone(),
                message: error.to_string(),
            })?;
        let snapshot = git.snapshot().map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
        if contract.checkpoint_policy.is_some() {
            let current_contract_hash = contract_digest(&contract_path)?.to_string();
            let archived_snapshot = summary
                .get("preflightRepositorySnapshotDigest")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_owned();
            let expected_snapshot = if archived_snapshot.is_empty() {
                snapshot_digest(&snapshot)?.to_string()
            } else {
                archived_snapshot
            };
            if let Err(errors) = validate_checkpoint_evidence_bindings(
                &contract,
                &summary,
                &repository_id(&root).to_string(),
                &expected_snapshot,
                &current_contract_hash,
            ) {
                return Err(ObserverError::State {
                    path: summary_path.clone(),
                    message: format!("checkpoint evidence is invalid: {}", errors.join(", ")),
                });
            }
        }
        let evidence_state =
            verification_evidence_state(&root, &contract, &snapshot, true, current_runtime)?;
        let historical_compatible = evidence_state != EvidenceState::Complete
            && archived_evidence_is_historical(&root, &contract, &snapshot, current_runtime)?;
        if evidence_state != EvidenceState::Complete && !historical_compatible {
            return Err(ObserverError::State {
                path: root
                    .join(".ai/evidence")
                    .join(format!("{work_item_id}.verification.json")),
                message: "close requires valid verification evidence".into(),
            });
        }
        require_green_governance_for_archived_contract(
            &root,
            &contract_path,
            &contract,
            &snapshot,
            "close",
            current_runtime,
        )?;
        let finalization_path = resource_finalization_decision_path(&root, work_item_id);
        if contract.resource_context.is_some()
            && (current_runtime.is_some() || fs::symlink_metadata(&finalization_path).is_ok())
        {
            finalization_binding = Some(require_resource_finalization_for_close(
                &root,
                work_item_id,
                current_runtime,
            )?);
        }
    }
    validate_policy_decision(&root, &contract, human_decision)?;
    let outcome = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.outcome.json"));
    let outcome_value = read_json(&outcome)?;
    if !superseded && outcome_value["verification"]["status"] != "verified" {
        return Err(ObserverError::State {
            path: outcome,
            message: "close requires a verified outcome".into(),
        });
    }
    let timestamp = now();
    let receipt = LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "closed".into(),
        timestamp: timestamp.clone(),
    };
    let receipt_value = serde_json::to_value(&receipt).map_err(|error| ObserverError::State {
        path: root.join(".ai/decisions"),
        message: error.to_string(),
    })?;
    let decision_path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    if decision_path.exists() {
        return Err(ObserverError::State {
            path: decision_path,
            message: "work item is already closed".into(),
        });
    }
    let mut decision = receipt_value;
    decision["repositoryId"] = contract.repository_id.clone().into();
    if let Some(binding) = finalization_binding {
        decision["resourceFinalizationHeadPath"] = binding["headPath"].clone();
        decision["resourceFinalizationHeadDigest"] = binding["headDigest"].clone();
        decision["resourceFinalizationSequence"] = binding["sequence"].clone();
    }
    decision["humanDecision"] = serde_json::Value::String(human_decision.decision.trim().into());
    decision["decisionState"] = serde_json::Value::String("confirmed".into());
    if amendment_revalidation_resolved {
        let recovery = recovery_decision
            .as_ref()
            .expect("amendment revalidation resolution has a recovery decision");
        decision["historicalRevalidation"] = serde_json::json!({
            "state": "current_successor_revalidated",
            "successorWorkItemId": recovery.successor_work_item_id,
            "historicalContractDigest": recovery.predecessor_contract_digest,
            "currentContractDigest": recovery.current_contract_digest,
            "historicalVerificationEvidenceDigest": recovery.predecessor_verification_evidence_digest,
            "archiveManifestDigest": recovery.predecessor_archive_manifest_digest,
            "assurance": "historical_low",
            "originalEvidencePreserved": true,
        });
    }
    if let Some(artifact) = historical_archive_integrity {
        decision["historicalArchiveIntegrity"] = serde_json::json!({
            "state": "quarantined",
            "artifact": artifact,
            "manifestDigest": recovery_decision
                .as_ref()
                .and_then(|value| value.predecessor_archive_manifest_digest.as_ref())
                .map(ToString::to_string),
            "assurance": "historical_low",
            "reason": "optional archived Task Outcome Markdown bytes differ from the immutable manifest digest; bytes were preserved"
        });
    }
    if let Some(final_report) = outcome_value.get("taskOutcomeReport") {
        let mut final_report: TaskOutcomeReport = serde_json::from_value(final_report.clone())
            .map_err(|error| ObserverError::State {
                path: outcome.clone(),
                message: format!("archived Task Outcome report is invalid: {error}"),
            })?;
        final_report.sections.human_decisions.push(OutcomeClaim {
            text: format!(
                "Human decision '{}' by {} via {} at {}.",
                human_decision.decision,
                human_decision.actor,
                human_decision.authority_source,
                human_decision.decided_at
            ),
            evidence_refs: human_decision.evidence_refs.clone(),
            inference: human_decision.evidence_refs.is_empty(),
        });
        let final_report =
            serde_json::to_value(&final_report).map_err(|error| ObserverError::State {
                path: outcome.clone(),
                message: format!("final Task Outcome report cannot be encoded: {error}"),
            })?;
        decision["finalReport"] = final_report.clone();
        decision["finalReportDigest"] = cockpit_protocol::digest_json(&final_report)
            .map_err(|error| ObserverError::State {
                path: outcome.clone(),
                message: error.to_string(),
            })?
            .to_string()
            .into();
    }
    decision["structuredDecision"] =
        serde_json::to_value(human_decision).map_err(|error| ObserverError::State {
            path: root.join(".ai/decisions"),
            message: error.to_string(),
        })?;
    atomic_json(&decision_path, &decision)?;
    Ok(receipt)
}

fn require_resource_finalization_for_close(
    root: &Path,
    work_item_id: &str,
    runtime: Option<&RuntimeContext>,
) -> Result<serde_json::Value, ObserverError> {
    let result = verify_resource_finalization_internal(root, work_item_id, runtime)?;
    let disposition = result["disposition"].as_str().unwrap_or_default();
    let historical_kind = result["receipt"]["historical"]["kind"].as_str();
    let recovered_historical_kind = result["historicalKind"].as_str();
    let historical_retained = disposition == "retained"
        && (historical_kind == Some("shared_worktree_retained")
            || recovered_historical_kind == Some("shared_worktree_retained")
            || historical_kind == Some("direct_merge_no_pr")
            || recovered_historical_kind == Some("contract_amendment_revalidation"));
    if !matches!(disposition, "deleted" | "abandoned") && !historical_retained {
        return Err(ObserverError::State {
            path: resource_finalization_decision_path(root, work_item_id),
            message: format!(
                "close requires resource finalization disposition deleted or abandoned; retained resources require cleanup before close, got {disposition}"
            ),
        });
    }
    Ok(result)
}

fn validate_policy_decision(
    root: &Path,
    contract: &Contract,
    decision: &HumanDecision,
) -> Result<(), ObserverError> {
    let Some(policy) = effective_policy_for_contract(root, contract)? else {
        return Ok(());
    };
    let Some(rule) = contract_policy_rule(contract, &policy) else {
        return Ok(());
    };
    match rule.approval_mode {
        ApprovalMode::NoHumanApprovalForLowRisk => Ok(()),
        ApprovalMode::SingleAuthorizedHuman => {
            if decision.actor == "legacy-cli" || decision.authority_source == "explicit-cli" {
                return Err(ObserverError::State {
                    path: root.join(".ai/decisions"),
                    message: "policy-protected close requires structured human identity and authority source".into(),
                });
            }
            let policy_ids = policy
                .policy_id
                .strip_prefix("effective:")
                .unwrap_or(&policy.policy_id)
                .split(':')
                .collect::<Vec<_>>();
            if !decision.policy_refs.iter().any(|reference| {
                reference == &policy.policy_id || policy_ids.contains(&reference.as_str())
            }) {
                return Err(ObserverError::State {
                    path: root.join(".ai/decisions"),
                    message: format!("structured decision must bind policy {}", policy.policy_id),
                });
            }
            Ok(())
        }
        ApprovalMode::MultiPartyApproval | ApprovalMode::ExternalProviderApproval => {
            Err(ObserverError::State {
                path: root.join(".ai/decisions"),
                message: format!(
                    "policy approval mode {:?} is fail-closed until its external receipt is bound",
                    rule.approval_mode
                ),
            })
        }
    }
}

fn verify_archive_manifest(
    root: &Path,
    work_item_id: &str,
    manifest: &serde_json::Value,
) -> Result<(), ObserverError> {
    verify_archive_manifest_with_options(root, work_item_id, manifest, false).map(|_| ())
}

/// Verify an archive manifest while optionally quarantining one narrowly
/// classified historical mismatch. Required contract/summary/outcome bytes,
/// identity, historical artifacts, events, JSON reports, and all other
/// optional artifacts remain strict. The only permitted exception is a
/// `taskReportMarkdown` byte digest mismatch bound by an explicit supersede
/// recovery receipt.
fn verify_archive_manifest_with_options(
    root: &Path,
    work_item_id: &str,
    manifest: &serde_json::Value,
    allow_task_report_markdown_digest_mismatch: bool,
) -> Result<Option<String>, ObserverError> {
    if manifest["workItemId"] != serde_json::Value::String(work_item_id.into())
        || !matches!(manifest["state"].as_str(), Some("archived" | "superseded"))
    {
        return Err(ObserverError::State {
            path: root
                .join(".ai/work-items/archive")
                .join(format!("{work_item_id}.archive.json")),
            message: "archive manifest identity or state is invalid".into(),
        });
    }
    let archive = root.join(".ai/work-items/archive");
    for name in ["contract", "summary", "outcome"] {
        let path = archive.join(format!("{work_item_id}.{name}.json"));
        let bytes = fs::read(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        let expected = manifest["files"][format!("{name}Digest")]
            .as_str()
            .ok_or_else(|| ObserverError::State {
                path: path.clone(),
                message: format!("archive manifest is missing {name} digest"),
            })?;
        let actual = Digest::sha256_bytes(&bytes).to_string();
        if actual != expected {
            return Err(ObserverError::State {
                path,
                message: format!("archived {name} digest does not match manifest"),
            });
        }
    }
    // Normal terminal archives embed the generated report digests in the
    // archived Outcome. Superseded predecessors must retain their original
    // Outcome bytes verbatim; their manifest is the immutable binding for the
    // copied report artifacts and therefore must not force a historical rewrite.
    if manifest["state"] != serde_json::json!("superseded") {
        let archived_outcome = read_json(&archive.join(format!("{work_item_id}.outcome.json")))?;
        for name in ["taskReport", "taskReportMarkdown"] {
            let manifest_digest = manifest["files"][format!("{name}Digest")].as_str();
            let outcome_key = match name {
                "taskReport" => "taskReportDigest",
                _ => "taskReportMarkdownDigest",
            };
            let outcome_digest = archived_outcome
                .get(outcome_key)
                .and_then(|value| value.as_str());
            if manifest_digest != outcome_digest {
                return Err(ObserverError::State {
                    path: archive.join(format!("{work_item_id}.outcome.json")),
                    message: format!("archived outcome and manifest {name} digests are not bound"),
                });
            }
        }
    }
    for (name, suffix) in [
        ("events", "events.jsonl"),
        ("approach", "approach.json"),
        ("intelligence", "intelligence.json"),
        ("taskReport", "task-report.json"),
        ("taskReportMarkdown", "task-report.md"),
    ] {
        if !manifest["files"][format!("{name}Digest")].is_string() {
            continue;
        }
        let path = archive.join(format!("{work_item_id}.{suffix}"));
        let bytes = fs::read(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        let expected = manifest["files"][format!("{name}Digest")]
            .as_str()
            .ok_or_else(|| ObserverError::State {
                path: path.clone(),
                message: format!("archive manifest has an invalid {name} digest"),
            })?;
        if Digest::sha256_bytes(&bytes).to_string() != expected {
            if allow_task_report_markdown_digest_mismatch && name == "taskReportMarkdown" {
                continue;
            }
            return Err(ObserverError::State {
                path: path.clone(),
                message: format!("archived {name} digest does not match manifest"),
            });
        }
        if name == "events" {
            validate_task_outcome_events(
                root,
                &path,
                &repository_id(root).to_string(),
                work_item_id,
            )?;
        } else if name == "taskReport" {
            let value: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
                    path: path.clone(),
                    message: error.to_string(),
                })?;
            let report: TaskOutcomeReport =
                serde_json::from_value(value).map_err(|error| ObserverError::State {
                    path: path.clone(),
                    message: format!("archived Task Outcome report is invalid: {error}"),
                })?;
            if report.bindings.repository_id != repository_id(root).to_string()
                || report.bindings.work_item_id != work_item_id
                || report.work_item_id != work_item_id
            {
                return Err(ObserverError::State {
                    path,
                    message: "archived Task Outcome report identity does not match repository or Work Item".into(),
                });
            }
        } else if name == "intelligence" {
            let value: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
                    path: path.clone(),
                    message: error.to_string(),
                })?;
            let intelligence: WorkItemIntelligence =
                serde_json::from_value(value).map_err(|error| ObserverError::State {
                    path: path.clone(),
                    message: format!("archived Work Item intelligence is invalid: {error}"),
                })?;
            if intelligence.repository_id != repository_id(root).to_string()
                || intelligence.work_item_id != work_item_id
            {
                return Err(ObserverError::State {
                    path,
                    message: "archived Work Item intelligence identity does not match repository or Work Item".into(),
                });
            }
        }
    }
    if let Some(historical) = manifest.get("historicalArtifacts") {
        let entries = historical.as_array().ok_or_else(|| ObserverError::State {
            path: archive.join(format!("{work_item_id}.archive.json")),
            message: "archive historicalArtifacts must be an array".into(),
        })?;
        let expected_prefix = format!(".ai/work-items/archive/{work_item_id}.");
        let mut paths = BTreeSet::new();
        for entry in entries {
            let path_value = entry
                .get("path")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| ObserverError::State {
                    path: archive.join(format!("{work_item_id}.archive.json")),
                    message: "archive historical artifact path is missing".into(),
                })?;
            let digest = entry
                .get("digest")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| ObserverError::State {
                    path: archive.join(format!("{work_item_id}.archive.json")),
                    message: "archive historical artifact digest is missing".into(),
                })?;
            if !path_value.starts_with(&expected_prefix)
                || path_value.contains("..")
                || path_value.contains('\\')
                || !paths.insert(path_value.to_owned())
            {
                return Err(ObserverError::State {
                    path: archive.join(format!("{work_item_id}.archive.json")),
                    message: "archive historical artifact path is unsafe or duplicated".into(),
                });
            }
            let relative = path_value.strip_prefix("./").unwrap_or(path_value);
            let path = root.join(relative);
            if !optional_regular_artifact(&path, "archived historical Work Item artifact")? {
                return Err(ObserverError::State {
                    path,
                    message: "archive historical artifact is missing".into(),
                });
            }
            let actual =
                Digest::sha256_bytes(&fs::read(&path).map_err(|source| ObserverError::Read {
                    path: path.clone(),
                    source,
                })?);
            if actual.to_string() != digest {
                return Err(ObserverError::State {
                    path,
                    message: "archive historical artifact digest does not match manifest".into(),
                });
            }
        }
    }
    let mismatched_report = if allow_task_report_markdown_digest_mismatch {
        let expected = manifest["files"]["taskReportMarkdownDigest"].as_str();
        let path = archive.join(format!("{work_item_id}.task-report.md"));
        expected.and_then(|expected| {
            fs::read(&path).ok().and_then(|bytes| {
                (Digest::sha256_bytes(&bytes).to_string() != expected)
                    .then(|| "taskReportMarkdown".to_owned())
            })
        })
    } else {
        None
    };
    Ok(mismatched_report)
}

/// Bind a recovery receipt to the exact bytes of an archived manifest and
/// verify only its identity/state envelope. Full artifact verification is
/// deliberately separate so a supersede receipt can quarantine one bounded
/// historical optional-report mismatch without rewriting immutable bytes.
fn validate_recovery_archive_manifest_binding(
    root: &Path,
    work_item_id: &str,
    receipt: &RecoveryDecisionReceipt,
) -> Result<(), ObserverError> {
    let contract_amendment_revalidation =
        receipt.successor_binding_mode.as_deref() == Some("contract_amendment_revalidation");
    if receipt.decision != "supersede" && !contract_amendment_revalidation {
        return Err(recovery_decision_error(
            root.join(".ai/decisions"),
            "archive_manifest_binding_invalid",
            "predecessorArchiveManifestDigest is only valid for supersede or contract amendment revalidation decisions",
        ));
    }
    let Some(expected) = receipt.predecessor_archive_manifest_digest.as_ref() else {
        return Ok(());
    };
    let path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.archive.json"));
    if !is_regular_non_symlink(&path)? {
        return Err(recovery_decision_error(
            path,
            "archive_manifest_binding_invalid",
            "predecessor archive manifest must be a regular non-symlink file",
        ));
    }
    let bytes = fs::read(&path).map_err(|source| ObserverError::Read {
        path: path.clone(),
        source,
    })?;
    if Digest::sha256_bytes(&bytes) != *expected {
        return Err(recovery_decision_error(
            path,
            "archive_manifest_digest_mismatch",
            "predecessor archive manifest bytes do not match the recovery binding",
        ));
    }
    let manifest: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
        recovery_decision_error(
            path.clone(),
            "archive_manifest_binding_invalid",
            format!("predecessor archive manifest is not valid JSON: {error}"),
        )
    })?;
    if manifest["workItemId"] != serde_json::Value::String(work_item_id.into())
        || !matches!(manifest["state"].as_str(), Some("archived" | "superseded"))
    {
        return Err(recovery_decision_error(
            path,
            "archive_manifest_binding_invalid",
            "predecessor archive manifest identity or state is invalid",
        ));
    }
    Ok(())
}

fn verify_historical_supersede_manifest(
    root: &Path,
    work_item_id: &str,
    manifest: &serde_json::Value,
    receipt: &RecoveryDecisionReceipt,
) -> Result<Option<String>, ObserverError> {
    validate_recovery_archive_manifest_binding(root, work_item_id, receipt)?;
    verify_archive_manifest_with_options(root, work_item_id, manifest, true)
}

pub fn generate_knowledge(root: &Path) -> Result<cockpit_knowledge::KnowledgeIndex, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let archive = root.join(".ai/work-items/archive");
    let knowledge = root.join(".ai/knowledge");
    let index_path = knowledge.join("index.json");
    let source_digest = knowledge_source_digest(&archive)?;
    if index_path.is_file() {
        // A derived cache is disposable.  An unreadable, malformed, or
        // schema-incompatible index is treated as stale and rebuilt through
        // this explicit query path; authority remains in the archive.
        if let Ok(cached) = read_json(&index_path)
            && let Ok(index) = serde_json::from_value::<cockpit_knowledge::KnowledgeIndex>(cached)
            && index.source_digest == source_digest
        {
            return Ok(index);
        }
    }
    let mut records = Vec::new();
    for entry in fs::read_dir(&archive).map_err(|source| ObserverError::Read {
        path: archive.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: archive.clone(),
            source,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(work_item_id) = name.strip_suffix(".archive.json") else {
            continue;
        };
        let contract_path = archive.join(format!("{work_item_id}.contract.json"));
        let contract = read_json(&contract_path)?;
        let intent = contract["intent"].as_str().unwrap_or("unknown");
        records.push(cockpit_knowledge::project_record(
            work_item_id,
            intent,
            "archived",
            &format!(".ai/work-items/archive/{work_item_id}.archive.json"),
        ));
    }
    let index =
        cockpit_knowledge::KnowledgeIndex::from_records_with_source_digest(records, source_digest);
    fs::create_dir_all(&knowledge).map_err(|source| ObserverError::Read {
        path: knowledge.clone(),
        source,
    })?;
    let encoded = serde_json::to_value(&index).map_err(|error| ObserverError::State {
        path: knowledge.clone(),
        message: error.to_string(),
    })?;
    atomic_json(&knowledge.join("index.json"), &encoded)?;
    for record in &index.records {
        let record_value = serde_json::to_value(record).map_err(|error| ObserverError::State {
            path: knowledge.clone(),
            message: error.to_string(),
        })?;
        atomic_json(&root.join(&record.knowledge_path), &record_value)?;
    }
    Ok(index)
}

/// Build and persist a request-scoped, provenance-aware implementation
/// approach.  This explicit command is the write boundary for the approach
/// projection; read-only callers should use [`implementation_approach_read_only`].
pub fn implementation_approach(
    root: &Path,
    work_item_id: &str,
) -> Result<ImplementationApproach, ObserverError> {
    implementation_approach_internal(root, work_item_id, true)
}

/// Build the same implementation approach projection without materializing an
/// `.approach.json` artifact.  `work-item inspect` uses this variant so that
/// inspection remains a truthful read-only operation.
pub fn implementation_approach_read_only(
    root: &Path,
    work_item_id: &str,
) -> Result<ImplementationApproach, ObserverError> {
    implementation_approach_internal(root, work_item_id, false)
}

fn implementation_approach_internal(
    root: &Path,
    work_item_id: &str,
    persist: bool,
) -> Result<ImplementationApproach, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    let observation = observe(&root, &snapshot)?;
    let snapshot_digest = snapshot_digest(&snapshot)?;
    let evidence_prefix = format!(".ai/work-items/active/{work_item_id}");
    let mut facts = vec![
        cockpit_protocol::TraceableFact {
            key: "repositoryId".into(),
            value: serde_json::Value::String(contract.repository_id.clone()),
            origin: FactOrigin::Observed,
            evidence_refs: vec![".ai/cockpit.toml".into()],
            confidence: "high".into(),
        },
        cockpit_protocol::TraceableFact {
            key: "baseRevision".into(),
            value: serde_json::Value::String(contract.base_revision.clone()),
            origin: FactOrigin::Observed,
            evidence_refs: vec!["git:HEAD".into()],
            confidence: "high".into(),
        },
        cockpit_protocol::TraceableFact {
            key: "languages".into(),
            value: serde_json::to_value(&observation.languages).map_err(|error| {
                ObserverError::State {
                    path: root.clone(),
                    message: error.to_string(),
                }
            })?,
            origin: FactOrigin::Observed,
            evidence_refs: vec!["repository-snapshot".into()],
            confidence: "high".into(),
        },
        cockpit_protocol::TraceableFact {
            key: "buildSystems".into(),
            value: serde_json::to_value(&observation.build_systems).map_err(|error| {
                ObserverError::State {
                    path: root.clone(),
                    message: error.to_string(),
                }
            })?,
            origin: FactOrigin::Observed,
            evidence_refs: vec!["repository-snapshot".into()],
            confidence: "high".into(),
        },
    ];
    facts.sort_by(|left, right| left.key.cmp(&right.key));
    let mut derivations = Vec::new();
    if !observation.quality_commands.is_empty() {
        derivations.push(cockpit_protocol::TraceableDerivation {
            key: "verificationCapability".into(),
            value: serde_json::to_value(&observation.quality_commands).map_err(|error| {
                ObserverError::State {
                    path: root.clone(),
                    message: error.to_string(),
                }
            })?,
            rule: "observer.quality_commands_from_detected_build_system".into(),
            input_fact_keys: vec!["buildSystems".into()],
            evidence_refs: vec!["repository-snapshot".into()],
            confidence: "medium".into(),
        });
    }
    let mut unknowns = Vec::new();
    if contract.intent.is_empty() {
        unknowns.push("intent".into());
    }
    if contract.scope.is_empty() {
        unknowns.push("scope".into());
    }
    if contract.acceptance_criteria.is_empty() {
        unknowns.push("acceptanceCriteria".into());
    }
    if contract.authority.trim().is_empty() || contract.authority == "unknown" {
        unknowns.push("authority".into());
    }
    unknowns.sort();
    unknowns.dedup();
    let mut evidence_refs = vec![evidence_prefix, "repository-snapshot".into()];
    evidence_refs.sort();
    let approach = ImplementationApproach {
        schema_version: 2,
        repository_id: contract.repository_id,
        work_item_id: work_item_id.into(),
        repository_snapshot_digest: snapshot_digest,
        facts,
        derivations,
        unknowns,
        evidence_refs,
    };
    if persist {
        atomic_json(
            &root
                .join(".ai/work-items/active")
                .join(format!("{work_item_id}.approach.json")),
            &serde_json::to_value(&approach).map_err(|error| ObserverError::State {
                path: root.clone(),
                message: error.to_string(),
            })?,
        )?;
    }
    Ok(approach)
}

/// Return knowledge v2 projections without replacing the legacy index.  The
/// projection is derived from archive contracts and bound to one snapshot.
pub fn generate_knowledge_v2(
    root: &Path,
) -> Result<Vec<cockpit_protocol::KnowledgeV2Record>, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let digest = snapshot_digest(&snapshot)?;
    let repository_id = repository_id(&root).to_string();
    let archive = root.join(".ai/work-items/archive");
    let mut records = Vec::new();
    for entry in fs::read_dir(&archive).map_err(|source| ObserverError::Read {
        path: archive.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: archive.clone(),
            source,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(work_item_id) = name.strip_suffix(".archive.json") else {
            continue;
        };
        let contract: serde_json::Value =
            read_json(&archive.join(format!("{work_item_id}.contract.json")))?;
        let intent = contract["intent"].as_str().unwrap_or("unknown");
        records.push(cockpit_knowledge::project_record_v2(
            &repository_id,
            work_item_id,
            intent,
            "archived",
            &format!(".ai/work-items/archive/{work_item_id}.archive.json"),
            digest.clone(),
        ));
    }
    records.sort_by(|left, right| left.work_item_id.cmp(&right.work_item_id));
    let path = root.join(".ai/knowledge/index.v2.json");
    atomic_json(
        &path,
        &serde_json::to_value(&records).map_err(|error| ObserverError::State {
            path: path.clone(),
            message: error.to_string(),
        })?,
    )?;
    Ok(records)
}

/// Build a human-benefit-aware outcome while preserving the distinction
/// between verified implementation evidence and a user-visible benefit claim.
pub fn outcome_v2(root: &Path, work_item_id: &str) -> Result<OutcomeV2, ObserverError> {
    outcome_v2_internal(root, work_item_id, None)
}

/// Runtime-bound outcome projection used by CLI/MCP.  A current Runtime may
/// render a legacy archived record, but it must explain that the record is
/// historical and not revalidated rather than presenting it as a current red
/// failure.
pub fn outcome_v2_with_runtime(
    root: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> Result<OutcomeV2, ObserverError> {
    outcome_v2_internal(root, work_item_id, Some(runtime))
}

/// Derive a request-scoped Work Item status without writing any repository
/// state.  This is intentionally a projection over the existing Contract,
/// Summary, Outcome, and evidence records; it is not a second scheduler or
/// governance authority.
pub fn work_item_status_snapshot_with_runtime(
    root: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> Result<WorkItemStatusSnapshot, ObserverError> {
    work_item_status_snapshot_with_snapshot(root, work_item_id, runtime, None)
}

/// Project one Work Item using a snapshot captured by the caller.  The
/// aggregate status path uses this to avoid recapturing the same Git snapshot
/// once per Work Item; the snapshot and its digest remain request-scoped and
/// are never shared across repositories or processes.
fn work_item_status_snapshot_with_snapshot(
    root: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
    snapshot_override: Option<(&RepositorySnapshot, &Digest)>,
) -> Result<WorkItemStatusSnapshot, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let active = root.join(".ai/work-items/active");
    let archive = root.join(".ai/work-items/archive");
    let (contract_path, archived) = [
        (active.join(format!("{work_item_id}.contract.json")), false),
        (archive.join(format!("{work_item_id}.contract.json")), true),
    ]
    .into_iter()
    .find(|(path, _)| path.is_file())
    .ok_or_else(|| ObserverError::State {
        path: active.join(format!("{work_item_id}.contract.json")),
        message: "work item contract not found".into(),
    })?;
    let contract = read_contract(&contract_path)?;
    let expected_repository_id = repository_id(&root).to_string();
    if contract.repository_id != expected_repository_id {
        return Err(ObserverError::State {
            path: contract_path,
            message: format!(
                "contract repository identity mismatch: expected {expected_repository_id}, found {}",
                contract.repository_id
            ),
        });
    }
    let base_commit = contract
        .base_commit
        .clone()
        .unwrap_or_else(|| contract.base_revision.clone());
    let branch = contract
        .resource_context
        .as_ref()
        .map(|context| context.branch.clone());
    let mut _owned_snapshot = None;
    let snapshot_digest_value;
    if let Some((_, provided_digest)) = snapshot_override {
        snapshot_digest_value = provided_digest.clone();
    } else {
        let git =
            cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
                path: root.clone(),
                message: error.to_string(),
            })?;
        let captured_snapshot = git.snapshot().map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
        snapshot_digest_value = snapshot_digest(&captured_snapshot)?;
        _owned_snapshot = Some(captured_snapshot);
    }
    let outcome =
        outcome_v2_internal_with_snapshot(&root, work_item_id, Some(runtime), snapshot_override)?;
    let summary_path = contract_path
        .parent()
        .unwrap_or(&active)
        .join(format!("{work_item_id}.summary.json"));
    let summary = read_json(&summary_path).unwrap_or_else(|_| serde_json::json!({}));
    let close_decision_path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    let close_decision_present = fs::symlink_metadata(&close_decision_path).is_ok();
    let close_decision_valid = archived
        && close_decision_is_valid_for_status(&root, work_item_id, &contract.repository_id);
    let lifecycle_phase = if close_decision_valid {
        "closed".to_string()
    } else if archived {
        "archived".to_string()
    } else {
        summary["state"]
            .as_str()
            .or_else(|| Some(outcome_state_name(&outcome.state)))
            .unwrap_or(if archived { "archived" } else { "unknown" })
            .to_string()
    };
    let governance_state = match outcome.decision_state {
        Some(DecisionState::Green) => "green",
        Some(DecisionState::Yellow) => "yellow",
        Some(DecisionState::Red) => "red",
        None => "unknown",
    }
    .to_string();
    let verification = match outcome.state {
        OutcomeState::Verified => "verified",
        OutcomeState::Partial => "partial",
        OutcomeState::NotReady => "not_ready",
        OutcomeState::Unknown => "unknown",
    }
    .to_string();
    let historical = legacy_verification_evidence(&root, work_item_id);
    let activity_health = if historical {
        "historical"
    } else if verification == "unknown" {
        "degraded"
    } else if verification == "not_ready" {
        "waiting"
    } else if archived {
        "inactive"
    } else {
        "active"
    }
    .to_string();

    let acceptance_total = contract.acceptance_criteria.len() as u64;
    let acceptance_evidence = summary["acceptanceEvidence"]
        .as_object()
        .map(|value| value.len() as u64)
        .unwrap_or_default();
    let mut progress_facts = BTreeMap::new();
    progress_facts.insert("acceptanceCriteriaDeclared".into(), acceptance_total);
    progress_facts.insert("acceptanceEvidenceEntries".into(), acceptance_evidence);
    progress_facts.insert(
        "checkpointCount".into(),
        summary["checkpointCount"].as_u64().unwrap_or_default(),
    );
    progress_facts.insert(
        "changedPathCount".into(),
        summary["changedPaths"]
            .as_array()
            .map(|value| value.len() as u64)
            .unwrap_or_default(),
    );

    let mut unknowns = outcome.unknowns.clone();
    if historical {
        unknowns.push("legacy_evidence_historical".into());
    }
    if archived && !close_decision_valid {
        if close_decision_present {
            unknowns.push("close_decision_invalid".into());
        } else {
            unknowns.push("close_decision_pending".into());
        }
    }
    unknowns.sort();
    unknowns.dedup();
    let mut blockers = Vec::new();
    if governance_state == "red" {
        blockers.push("governance_red".into());
    }
    if archived && !close_decision_valid {
        blockers.push("archived_work_item_pending_close".into());
    }
    let blocking = !blockers.is_empty();
    let human_decision_required =
        summary["preflightState"] == "yellow" && summary["decisionEvidence"].is_null();
    let missing_evidence = unknowns
        .iter()
        .filter(|value| value.contains("evidence") || value.contains("verification"))
        .cloned()
        .collect::<Vec<_>>();
    let dependencies = summary["dependencies"]
        .as_array()
        .or_else(|| summary["dependsOn"].as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let mut risks = Vec::new();
    if !contract.risk.trim().is_empty() {
        risks.push(contract.risk.clone());
    }
    if let Some(items) = summary["risks"].as_array() {
        risks.extend(
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned)),
        );
    }
    risks.sort();
    risks.dedup();
    let mut completion_domains = BTreeMap::new();
    completion_domains.insert(
        "implementation".into(),
        if matches!(
            lifecycle_phase.as_str(),
            "implementation_active" | "checkpointed"
        ) {
            "active"
        } else if archived {
            "recorded"
        } else {
            "not_started"
        }
        .into(),
    );
    completion_domains.insert("verification".into(), verification.clone());
    completion_domains.insert(
        "review".into(),
        if governance_state == "green" {
            "available"
        } else {
            "required"
        }
        .into(),
    );
    completion_domains.insert(
        "integration".into(),
        if archived {
            "recorded"
        } else {
            "not_applicable"
        }
        .into(),
    );
    completion_domains.insert(
        "closure".into(),
        if close_decision_valid {
            "closed"
        } else if archived {
            "archived"
        } else {
            "open"
        }
        .into(),
    );
    let mut governance_permissions = vec!["read_status".into(), "read_outcome".into()];
    if governance_state == "green" && !historical {
        governance_permissions.push("review_evidence".into());
    }
    let mut source_digests = BTreeMap::new();
    source_digests.insert("contract".into(), contract_digest(&contract_path)?);
    source_digests.insert("repositorySnapshot".into(), snapshot_digest_value.clone());
    if summary_path.is_file()
        && let Ok(digest) = cockpit_protocol::digest_json(&summary)
    {
        source_digests.insert("summary".into(), digest);
    }
    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    let evidence = read_json(&evidence_path).ok();
    if let Some(evidence) = &evidence
        && let Ok(digest) = cockpit_protocol::digest_json(&evidence)
    {
        source_digests.insert("verificationEvidence".into(), digest);
    }
    let last_verification_at = evidence
        .as_ref()
        .and_then(|value| value["createdAt"].as_str())
        .map(str::to_owned);
    let evidence_freshness = if historical {
        WorkItemEvidenceFreshness {
            state: "historical".into(),
            reason: "verification evidence is immutable historical input and was not revalidated"
                .into(),
        }
    } else if evidence.is_none() {
        WorkItemEvidenceFreshness {
            state: "missing".into(),
            reason: "verification evidence is missing".into(),
        }
    } else if verification == "verified" {
        WorkItemEvidenceFreshness {
            state: "fresh".into(),
            reason: "verification evidence matches the current repository and Runtime bindings"
                .into(),
        }
    } else {
        WorkItemEvidenceFreshness {
            state: "stale_or_invalid".into(),
            reason: "verification evidence exists but does not authorize the current status".into(),
        }
    };
    let updated_at = summary["updatedAt"]
        .as_str()
        .or_else(|| summary["createdAt"].as_str())
        .or(contract.created_at.as_deref())
        .map(str::to_owned);
    let human_decisions = if close_decision_valid {
        vec!["close_decision_recorded".into()]
    } else {
        Vec::new()
    };
    let mut diagnostics = Vec::new();
    if historical {
        diagnostics.push("historical_evidence_not_revalidated".into());
    }
    if archived && !close_decision_valid {
        diagnostics.push(if close_decision_present {
            "close_decision_not_accepted".into()
        } else {
            "lifecycle_cleanup_required".into()
        });
    }
    let mut safe_actions = if archived && !close_decision_valid {
        let mut actions = Vec::new();
        if contract.resource_context.is_some() {
            let finalization_path = resource_finalization_decision_path(&root, work_item_id);
            let finalization_state = if fs::symlink_metadata(&finalization_path).is_err() {
                "missing"
            } else {
                match verify_resource_finalization_internal(&root, work_item_id, Some(runtime)) {
                    Ok(value) if value["disposition"].as_str() == Some("deleted") => "deleted",
                    Ok(_) => "retained",
                    Err(_) => "invalid",
                }
            };
            match finalization_state {
                "deleted" => {
                    actions.extend(["finalize_verify", "close"].into_iter().map(str::to_owned))
                }
                "retained" => actions.extend(
                    [
                        "cleanup_resources",
                        "record_finalization",
                        "finalize_verify",
                        "close_after_cleanup",
                    ]
                    .into_iter()
                    .map(str::to_owned),
                ),
                "missing" => actions.extend(
                    [
                        "finalize_resources",
                        "record_finalization",
                        "finalize_verify",
                        "close_after_cleanup",
                    ]
                    .into_iter()
                    .map(str::to_owned),
                ),
                _ => actions.extend(
                    [
                        "repair_finalization",
                        "finalize_verify",
                        "close_after_cleanup",
                    ]
                    .into_iter()
                    .map(str::to_owned),
                ),
            }
        } else {
            actions.push("close_after_review".into());
        }
        actions
    } else if blocking {
        vec!["resolve_blockers".into(), "stop".into()]
    } else {
        match lifecycle_phase.as_str() {
            "implementation_active" => vec!["run_preflight".into()],
            "checkpointed" if verification != "verified" => vec!["run_verification".into()],
            "finish_ready" => vec!["archive_when_reviewed".into()],
            "archived" => vec!["read_outcome".into()],
            "closed" => Vec::new(),
            _ if verification != "verified" => vec!["run_verification".into()],
            _ => vec!["read_outcome".into()],
        }
    };
    safe_actions.push("refresh_status".into());
    safe_actions.sort();
    safe_actions.dedup();
    let status_digest = cockpit_protocol::digest_json(&serde_json::json!({
        "schemaVersion": 1,
        "repositoryId": contract.repository_id,
        "workItemId": work_item_id,
        "baseCommit": base_commit,
        "branch": branch,
        "lifecyclePhase": lifecycle_phase,
        "governanceState": governance_state,
        "activityHealth": activity_health,
        "blocking": blocking,
        "humanDecisionRequired": human_decision_required,
        "progressFacts": progress_facts,
        "blockers": blockers,
        "missingEvidence": missing_evidence,
        "dependencies": dependencies,
        "humanDecisions": human_decisions,
        "risks": risks,
        "verification": verification,
        "completionDomains": completion_domains,
        "governancePermissions": governance_permissions,
        "sourceDigests": source_digests,
        "unknowns": unknowns,
        "diagnostics": diagnostics,
        "snapshotDigest": snapshot_digest_value,
        "evidenceFreshness": evidence_freshness,
        "lastVerificationAt": last_verification_at,
        "updatedAt": updated_at,
        "safeActions": safe_actions,
        "historical": historical,
    }))
    .map_err(|error| ObserverError::State {
        path: contract_path.clone(),
        message: error.to_string(),
    })?;
    Ok(WorkItemStatusSnapshot {
        schema_version: 1,
        repository_id: contract.repository_id,
        work_item_id: work_item_id.into(),
        base_commit,
        branch,
        lifecycle_phase,
        governance_state,
        activity_health,
        blocking,
        human_decision_required,
        progress_facts,
        blockers,
        missing_evidence,
        dependencies,
        human_decisions,
        risks,
        verification,
        completion_domains,
        governance_permissions,
        source_digests,
        unknowns,
        diagnostics,
        snapshot_digest: snapshot_digest_value,
        evidence_freshness,
        last_verification_at,
        updated_at,
        safe_actions,
        status_digest,
        historical,
    })
}

/// Aggregate every active and archived Work Item into a stable read-only
/// projection. An unreadable member is retained as an explicit unknown entry;
/// it cannot hide other members or promote any count to green.
pub fn work_item_status_index_with_runtime(
    root: &Path,
    runtime: &RuntimeContext,
) -> Result<WorkItemStatusIndex, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let repository_snapshot_digest = snapshot_digest(&snapshot)?;
    let expected_repository_id = repository_id(&root).to_string();

    let mut work_item_ids = BTreeMap::<String, ()>::new();
    let mut index_unknowns = Vec::new();
    let mut index_diagnostics = Vec::new();
    for relative in [".ai/work-items/active", ".ai/work-items/archive"] {
        let directory = root.join(relative);
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                index_unknowns.push(format!("work_item_directory_unreadable:{relative}"));
                index_diagnostics
                    .push(format!("work_item_directory_unreadable:{relative}:{error}"));
                continue;
            }
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let Some(work_item_id) = name.strip_suffix(".contract.json") else {
                continue;
            };
            let is_regular = fs::symlink_metadata(entry.path())
                .ok()
                .is_some_and(|metadata| metadata.file_type().is_file());
            if is_regular {
                work_item_ids.insert(work_item_id.to_string(), ());
            } else {
                index_unknowns.push(format!("contract_not_regular:{work_item_id}"));
            }
        }
    }

    let mut counts = BTreeMap::from([
        ("green".into(), 0_u64),
        ("red".into(), 0_u64),
        ("unknown".into(), 0_u64),
        ("yellow".into(), 0_u64),
    ]);
    let mut items = Vec::with_capacity(work_item_ids.len());
    for work_item_id in work_item_ids.keys() {
        let entry = match work_item_status_snapshot_with_snapshot(
            &root,
            work_item_id,
            runtime,
            Some((&snapshot, &repository_snapshot_digest)),
        ) {
            Ok(status) => {
                let status_digest = status.status_digest.clone();
                WorkItemStatusIndexEntry {
                    work_item_id: work_item_id.clone(),
                    governance_state: status.governance_state.clone(),
                    status_digest,
                    unknowns: status.unknowns.clone(),
                    diagnostics: status.diagnostics.clone(),
                    status: Some(status),
                }
            }
            Err(error) => {
                let unknowns = vec!["status_projection_failed".into()];
                let diagnostics = vec![format!("status_projection_failed:{error}")];
                let stable = serde_json::json!({
                    "workItemId": work_item_id,
                    "governanceState": "unknown",
                    "unknowns": unknowns,
                    "diagnostics": diagnostics,
                });
                let status_digest = cockpit_protocol::digest_json(&stable).map_err(|error| {
                    ObserverError::State {
                        path: root.clone(),
                        message: error.to_string(),
                    }
                })?;
                WorkItemStatusIndexEntry {
                    work_item_id: work_item_id.clone(),
                    governance_state: "unknown".into(),
                    status_digest,
                    status: None,
                    unknowns,
                    diagnostics,
                }
            }
        };
        *counts.entry(entry.governance_state.clone()).or_default() += 1;
        items.push(entry);
    }
    index_unknowns.sort();
    index_unknowns.dedup();
    index_diagnostics.sort();
    index_diagnostics.dedup();
    index_diagnostics.push(format!("work_items_aggregated:{}", items.len()));
    let stable = serde_json::json!({
        "schemaVersion": 1,
        "repositoryId": expected_repository_id,
        "snapshotDigest": repository_snapshot_digest,
        "counts": counts,
        "items": items,
        "unknowns": index_unknowns,
        "diagnostics": index_diagnostics,
    });
    let index_digest =
        cockpit_protocol::digest_json(&stable).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;

    Ok(WorkItemStatusIndex {
        schema_version: 1,
        repository_id: expected_repository_id,
        snapshot_digest: repository_snapshot_digest,
        counts,
        items,
        unknowns: index_unknowns,
        diagnostics: index_diagnostics,
        index_digest,
    })
}

/// Validate the close receipt before exposing a terminal `closed` status.
/// Merely finding a decision file is not enough: the record must be a regular
/// repository-local file with the same Work Item identity, a confirmed closed
/// state, and a strict structured human decision whose summary agrees with
/// the structured value. Invalid records remain visible as unknowns and can
/// never promote an archived Work Item to `closed`.
pub(crate) fn close_decision_is_valid_for_status(
    root: &Path,
    work_item_id: &str,
    repository_id: &str,
) -> bool {
    let path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        return false;
    };
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return false;
    }
    let Ok(value) = read_json(&path) else {
        return false;
    };
    if value.get("workItemId").and_then(serde_json::Value::as_str) != Some(work_item_id)
        || value
            .get("repositoryId")
            .and_then(serde_json::Value::as_str)
            != Some(repository_id)
        || value.get("state").and_then(serde_json::Value::as_str) != Some("closed")
        || value
            .get("decisionState")
            .and_then(serde_json::Value::as_str)
            != Some("confirmed")
    {
        return false;
    }
    let Some(structured) = value.get("structuredDecision").cloned() else {
        return false;
    };
    let Ok(decision) = serde_json::from_value::<HumanDecision>(structured) else {
        return false;
    };
    if [
        decision.decision.as_str(),
        decision.actor.as_str(),
        decision.authority_source.as_str(),
        decision.reason.as_str(),
        decision.decided_at.as_str(),
    ]
    .iter()
    .any(|value| value.trim().is_empty())
    {
        return false;
    }
    value
        .get("humanDecision")
        .and_then(serde_json::Value::as_str)
        == Some(decision.decision.as_str())
}

fn outcome_state_name(state: &OutcomeState) -> &'static str {
    match state {
        OutcomeState::Verified => "verified",
        OutcomeState::Partial => "partial",
        OutcomeState::NotReady => "not_ready",
        OutcomeState::Unknown => "unknown",
    }
}

fn report_claim(text: impl Into<String>, evidence_refs: &[String]) -> OutcomeClaim {
    let evidence_refs = evidence_refs.to_vec();
    OutcomeClaim {
        text: text.into(),
        inference: evidence_refs.is_empty(),
        evidence_refs,
    }
}

fn repository_relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn archived_work_item_reference(work_item_id: &str, value: &str) -> String {
    value.replace(
        &format!(".ai/work-items/active/{work_item_id}"),
        &format!(".ai/work-items/archive/{work_item_id}"),
    )
}

fn normalize_archived_task_report_value(value: &mut serde_json::Value, work_item_id: &str) {
    match value {
        serde_json::Value::Object(object) => {
            for (key, child) in object.iter_mut() {
                match key.as_str() {
                    "evidenceRefs" => {
                        if let serde_json::Value::Array(items) = child {
                            for item in items {
                                if let serde_json::Value::String(reference) = item {
                                    *reference =
                                        archived_work_item_reference(work_item_id, reference);
                                }
                            }
                        }
                    }
                    "text" => {
                        if let serde_json::Value::String(text) = child {
                            *text = archived_work_item_reference(work_item_id, text);
                        }
                    }
                    _ => normalize_archived_task_report_value(child, work_item_id),
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                normalize_archived_task_report_value(item, work_item_id);
            }
        }
        _ => {}
    }
}

fn normalize_archived_summary_value(value: &mut serde_json::Value, work_item_id: &str) {
    let Some(changed_paths) = value
        .get_mut("changedPaths")
        .and_then(serde_json::Value::as_array_mut)
    else {
        return;
    };
    for path in changed_paths {
        if let serde_json::Value::String(path) = path {
            *path = archived_work_item_reference(work_item_id, path);
        }
    }
}

fn normalize_archived_events_bytes(
    bytes: &[u8],
    work_item_id: &str,
) -> Result<Vec<u8>, ObserverError> {
    let mut normalized = Vec::with_capacity(bytes.len());
    for line in bytes.split_inclusive(|byte| *byte == b'\n') {
        let (content, newline) = if line.last().is_some_and(|byte| *byte == b'\n') {
            (&line[..line.len() - 1], b'\n')
        } else {
            (line, 0)
        };
        if content.is_empty() {
            normalized.extend_from_slice(line);
            continue;
        }
        let mut value: serde_json::Value =
            serde_json::from_slice(content).map_err(|error| ObserverError::State {
                path: PathBuf::from(".ai/work-items/active"),
                message: format!("invalid Task Outcome event while archiving: {error}"),
            })?;
        normalize_archived_task_report_value(&mut value, work_item_id);
        let encoded = serde_json::to_vec(&value).map_err(|error| ObserverError::State {
            path: PathBuf::from(".ai/work-items/archive"),
            message: format!("serialize Task Outcome event while archiving: {error}"),
        })?;
        normalized.extend_from_slice(&encoded);
        if newline != 0 {
            normalized.push(newline);
        }
    }
    Ok(normalized)
}

fn normalized_archive_artifact_bytes(
    suffix: &str,
    bytes: &[u8],
    work_item_id: &str,
) -> Result<Vec<u8>, ObserverError> {
    let active_reference = format!(".ai/work-items/active/{work_item_id}");
    if !String::from_utf8_lossy(bytes).contains(&active_reference) {
        return Ok(bytes.to_vec());
    }
    match suffix {
        "outcome.json" => {
            let mut value: serde_json::Value =
                serde_json::from_slice(bytes).map_err(|error| ObserverError::State {
                    path: PathBuf::from(".ai/work-items/active"),
                    message: format!("invalid Outcome while archiving: {error}"),
                })?;
            if let Some(report) = value.get_mut("taskOutcomeReport") {
                normalize_archived_task_report_value(report, work_item_id);
            }
            serde_json::to_vec_pretty(&value).map_err(|error| ObserverError::State {
                path: PathBuf::from(".ai/work-items/archive"),
                message: format!("serialize Outcome while archiving: {error}"),
            })
        }
        "task-report.json" => {
            let mut value: serde_json::Value =
                serde_json::from_slice(bytes).map_err(|error| ObserverError::State {
                    path: PathBuf::from(".ai/work-items/active"),
                    message: format!("invalid Task Outcome report while archiving: {error}"),
                })?;
            normalize_archived_task_report_value(&mut value, work_item_id);
            serde_json::to_vec_pretty(&value).map_err(|error| ObserverError::State {
                path: PathBuf::from(".ai/work-items/archive"),
                message: format!("serialize Task Outcome report while archiving: {error}"),
            })
        }
        "summary.json" => {
            let mut value: serde_json::Value =
                serde_json::from_slice(bytes).map_err(|error| ObserverError::State {
                    path: PathBuf::from(".ai/work-items/active"),
                    message: format!("invalid Summary while archiving: {error}"),
                })?;
            normalize_archived_summary_value(&mut value, work_item_id);
            serde_json::to_vec_pretty(&value).map_err(|error| ObserverError::State {
                path: PathBuf::from(".ai/work-items/archive"),
                message: format!("serialize Summary while archiving: {error}"),
            })
        }
        "events.jsonl" => normalize_archived_events_bytes(bytes, work_item_id),
        "task-report.md" => String::from_utf8(bytes.to_vec())
            .map(|text| archived_work_item_reference(work_item_id, &text).into_bytes())
            .map_err(|error| ObserverError::State {
                path: PathBuf::from(".ai/work-items/active"),
                message: format!("invalid Task Outcome Markdown while archiving: {error}"),
            }),
        _ => Ok(bytes.to_vec()),
    }
}

/// Preserve a machine-readable recovery handoff when a lifecycle gate fails.
/// The original gate error remains authoritative, while any persisted
/// projection is identity-bound and never changes the lifecycle state to a
/// terminal success. A persistence failure is returned to the caller instead
/// of being silently discarded.
fn persist_blocked_lifecycle_outcome(
    root: &Path,
    work_item_id: &str,
    error: &ObserverError,
) -> Result<(), ObserverError> {
    if validate_work_item_id(work_item_id).is_err() {
        return Ok(());
    }
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let active = root.join(".ai/work-items/active");
    let contract_path = active.join(format!("{work_item_id}.contract.json"));
    if !is_regular_non_symlink(&contract_path)? {
        return Ok(());
    }
    let contract = read_contract(&contract_path)?;
    let summary_path = active.join(format!("{work_item_id}.summary.json"));
    let mut summary = if is_regular_non_symlink(&summary_path)? {
        Some(read_json(&summary_path)?)
    } else {
        None
    };
    if summary
        .as_ref()
        .and_then(|value| value["state"].as_str())
        .is_some_and(|state| matches!(state, "finish_ready" | "archived" | "closed"))
    {
        return Ok(());
    }
    let (failed_gate, recovery_condition) = lifecycle_failure_metadata(error);
    let evidence_ref = format!(".ai/evidence/{work_item_id}.verification.json");
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .ok()
        .and_then(|git| git.snapshot().ok());
    let snapshot_digest = snapshot
        .as_ref()
        .and_then(|value| snapshot_digest(value).ok());
    let unknowns = vec!["lifecycle_gate_failed".to_string()];
    let task_report = task_outcome_report(TaskOutcomeReportInput {
        root: &root,
        contract_path: &contract_path,
        contract: &contract,
        summary: summary.as_ref(),
        snapshot_digest,
        state: OutcomeState::Unknown,
        decision_state: DecisionState::Red,
        summary_text: "A lifecycle gate failed; completion is not claimed and the Work Item remains recoverable.",
        unknowns: &unknowns,
        evidence_ref: &evidence_ref,
        failed_gate_override: Some(&failed_gate),
        recovery_condition_override: Some(&recovery_condition),
        historical: false,
    });
    append_task_outcome_recovery_event(
        &root,
        &contract,
        &failed_gate,
        &recovery_condition,
        if root.join(&evidence_ref).is_file() {
            vec![evidence_ref.clone()]
        } else {
            Vec::new()
        },
    )?;
    let outcome_path = active.join(format!("{work_item_id}.outcome.json"));
    if !fs::symlink_metadata(&outcome_path).is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        let outcome_v2 = OutcomeV2 {
            schema_version: 2,
            repository_id: contract.repository_id.clone(),
            work_item_id: work_item_id.into(),
            state: OutcomeState::Unknown,
            decision_state: Some(DecisionState::Red),
            summary: "A lifecycle gate failed; completion is not claimed and the Work Item remains recoverable.".into(),
            acceptance_results: contract.acceptance_criteria.clone(),
            unknowns,
            evidence_refs: if root.join(&evidence_ref).is_file() {
                vec![evidence_ref.clone()]
            } else {
                Vec::new()
            },
            human_benefit_report: HumanBenefitReport {
                state: OutcomeState::Unknown,
                user_visible_changes: Vec::new(),
                affected_users: Vec::new(),
                unknowns: vec!["user_visible_benefit_not_declared".into()],
                evidence_refs: Vec::new(),
            },
            task_outcome_report: Some(task_report),
            failed_gate: Some(failed_gate.clone()),
            historical_status: None,
            recovery_condition: Some(recovery_condition.clone()),
            recovery_decision: None,
            governance_reasons: Vec::new(),
            finalization: None,
        };
        let mut value =
            serde_json::to_value(outcome_v2).map_err(|serialization| ObserverError::State {
                path: outcome_path.clone(),
                message: serialization.to_string(),
            })?;
        value["protocolVersion"] = serde_json::json!(1);
        value["workItemId"] = serde_json::json!(work_item_id);
        value["state"] = serde_json::json!("blocked");
        value["verification"] = serde_json::json!({
            "status": "blocked",
            "required": true,
            "evidencePath": evidence_ref,
        });
        atomic_json(&outcome_path, &value)?;
    }
    if let Some(summary) = summary.as_mut() {
        summary["outcomeState"] = "blocked".into();
        summary["failedGate"] = failed_gate.into();
        summary["recoveryCondition"] = recovery_condition.into();
        summary["updatedAt"] = now().into();
        atomic_json(&summary_path, summary)?;
    }
    Ok(())
}

fn lifecycle_failure_metadata(error: &ObserverError) -> (String, String) {
    let message = match error {
        ObserverError::State { message, .. } => message.as_str(),
        _ => "",
    };
    if matches!(
        message,
        "finish requires a recorded verification receipt"
            | "verification receipt is not a passed receipt for this work item"
            | "verification receipt is stale for the current repository snapshot"
            | "verification evidence is not a valid current receipt"
            | "finish requires a green preflight result for the current repository snapshot"
    ) || message.starts_with("verification evidence")
    {
        (
            "finish.verification".into(),
            "Record valid current verification evidence, rerun preflight, and retry finish.".into(),
        )
    } else if matches!(
        message,
        "finish requires a green preflight result after verification"
    ) {
        (
            "finish.preflight".into(),
            "Record a fresh non-red preflight result, then retry finish.".into(),
        )
    } else if message.starts_with("Contract/Summary governance controls are blocked:") {
        (
            "finish.governance".into(),
            "Repair the Contract or governance projection, rerun preflight, and retry finish."
                .into(),
        )
    } else if message.starts_with("finish requires a") && message.contains("finalization plan") {
        (
            "finish.finalization".into(),
            "Inspect the finalization plan and resource facts, then retry finish only under the Runtime rules."
                .into(),
        )
    } else {
        (
            "finish.lifecycle".into(),
            "Restore the required lifecycle state and retry finish after fresh checks.".into(),
        )
    }
}

struct TaskOutcomeReportInput<'a> {
    root: &'a Path,
    contract_path: &'a Path,
    contract: &'a Contract,
    summary: Option<&'a serde_json::Value>,
    snapshot_digest: Option<Digest>,
    state: OutcomeState,
    decision_state: DecisionState,
    summary_text: &'a str,
    unknowns: &'a [String],
    evidence_ref: &'a str,
    failed_gate_override: Option<&'a str>,
    recovery_condition_override: Option<&'a str>,
    historical: bool,
}

fn task_outcome_report(input: TaskOutcomeReportInput<'_>) -> TaskOutcomeReport {
    let TaskOutcomeReportInput {
        root,
        contract_path,
        contract,
        summary,
        snapshot_digest,
        state,
        decision_state,
        summary_text,
        unknowns,
        evidence_ref,
        failed_gate_override,
        recovery_condition_override,
        historical,
    } = input;
    let contract_ref = repository_relative_path(root, contract_path);
    let summary_ref = contract_path
        .parent()
        .map(|parent| {
            repository_relative_path(
                root,
                &parent.join(format!("{}.summary.json", contract.work_item_id)),
            )
        })
        .unwrap_or_else(|| {
            format!(
                ".ai/work-items/active/{}.summary.json",
                contract.work_item_id
            )
        });
    let evidence_refs = if root.join(evidence_ref).is_file() {
        vec![evidence_ref.to_string()]
    } else {
        Vec::new()
    };
    let mut sections = OutcomeReportSections {
        outcome_summary: vec![report_claim(summary_text, &evidence_refs)],
        task_overview: vec![report_claim(
            if contract.goal.trim().is_empty() {
                "Work Item goal is not declared."
            } else {
                contract.goal.as_str()
            },
            std::slice::from_ref(&contract_ref),
        )],
        forbidden_claims: vec![
            "merge_authorized_without_human_decision".into(),
            "release_published_without_release_evidence".into(),
            "provider_or_enterprise_approval_inferred_from_local_records".into(),
            "user_visible_benefit_invented_from_implementation_facts".into(),
        ],
        evidence: vec![report_claim(evidence_ref, &evidence_refs)],
        ..OutcomeReportSections::default()
    };

    if let Some(summary) = summary {
        if let Some(paths) = summary
            .get("changedPaths")
            .and_then(serde_json::Value::as_array)
        {
            sections
                .delivered_changes
                .extend(paths.iter().filter_map(|path| {
                    path.as_str().map(|path| {
                        report_claim(
                            format!("Changed path: {path}"),
                            std::slice::from_ref(&summary_ref),
                        )
                    })
                }));
        }
        if sections.delivered_changes.is_empty()
            && summary
                .get("changedPaths")
                .and_then(serde_json::Value::as_array)
                .is_some()
        {
            sections.non_risk_explanations.push(report_claim(
                "No repository paths were observed as changed by the current Summary.",
                std::slice::from_ref(&summary_ref),
            ));
        }
    }

    for unknown in unknowns {
        sections.residual_risks.push(report_claim(
            format!("Remaining unknown: {unknown}"),
            &evidence_refs,
        ));
    }
    sections.warnings.push(report_claim(
        "User-visible benefit is not declared by the Work Item owner.",
        std::slice::from_ref(&contract_ref),
    ));
    if historical {
        // Historical evidence is context, not a request for missing evidence
        // or human recovery input.
    } else if matches!(decision_state, DecisionState::Red) {
        sections.forced_stops.push(report_claim(
            "A required evidence or identity control failed; remain stopped.",
            &evidence_refs,
        ));
    } else if matches!(decision_state, DecisionState::Yellow) {
        sections.interventions.push(report_claim(
            "Additional evidence or human input is required before progression.",
            &evidence_refs,
        ));
    }
    if matches!(state, OutcomeState::Verified) {
        sections.resolutions.push(report_claim(
            "The current verification evidence is valid for this repository and Work Item.",
            &evidence_refs,
        ));
    }
    let failed_gate = if historical {
        None
    } else {
        failed_gate_override
            .map(str::to_owned)
            .or_else(|| match decision_state {
                DecisionState::Red => Some("evidence_or_identity_control".into()),
                DecisionState::Yellow => Some("verification_or_human_input".into()),
                DecisionState::Green => None,
            })
    };
    let recovery_condition = if historical {
        None
    } else {
        recovery_condition_override
            .map(str::to_owned)
            .or_else(|| match decision_state {
                DecisionState::Red => Some(
                    "Repair the invalid evidence or identity binding, then rerun verification."
                        .into(),
                ),
                DecisionState::Yellow => Some(
                    "Collect the missing evidence or human input, then rerun preflight/verification."
                        .into(),
                ),
                DecisionState::Green => None,
            })
    };

    TaskOutcomeReport {
        format: "ai-cockpit.task-outcome".into(),
        schema_version: 1,
        work_item_id: contract.work_item_id.clone(),
        status: state,
        human_status_color: decision_state,
        bindings: OutcomeReportBindings {
            repository_id: contract.repository_id.clone(),
            work_item_id: contract.work_item_id.clone(),
            evidence_refs,
            repository_snapshot_digest: snapshot_digest,
        },
        sections,
        failed_gate,
        recovery_condition,
    }
}

fn task_outcome_event_path(root: &Path, work_item_id: &str, archived: bool) -> PathBuf {
    let phase = if archived { "archive" } else { "active" };
    root.join(".ai/work-items")
        .join(phase)
        .join(format!("{work_item_id}.events.jsonl"))
}

fn event_detail_is_safe(detail: &str) -> bool {
    let lower = detail.to_ascii_lowercase();
    ![
        "api_key",
        "authorization:",
        "bearer ",
        "password=",
        "secret=",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn append_task_outcome_recovery_event(
    root: &Path,
    contract: &Contract,
    failed_gate: &str,
    recovery_condition: &str,
    evidence_refs: Vec<String>,
) -> Result<(), ObserverError> {
    let path = task_outcome_event_path(root, &contract.work_item_id, false);
    let mut events = if fs::symlink_metadata(&path).is_ok() {
        validate_task_outcome_events(root, &path, &contract.repository_id, &contract.work_item_id)?
    } else {
        Vec::new()
    };
    let detail = format!("Lifecycle gate blocked: {failed_gate}. {recovery_condition}");
    if events
        .iter()
        .any(|event| event.event_type == "blocked" && event.detail == detail)
    {
        return Ok(());
    }
    let timestamp = now();
    let event_id = format!(
        "{}-{}",
        event_id("blocked", &detail, &timestamp),
        events.len()
    );
    events.push(TaskOutcomeEvent {
        schema_version: 1,
        event_id,
        repository_id: contract.repository_id.clone(),
        work_item_id: contract.work_item_id.clone(),
        event_type: "blocked".into(),
        timestamp,
        detail,
        evidence_refs,
        related_event_ids: events
            .last()
            .map(|event| vec![event.event_id.clone()])
            .unwrap_or_default(),
        correction_of: None,
        finding_fingerprint: None,
    });
    let encoded = events
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| ObserverError::State {
            path: path.clone(),
            message: error.to_string(),
        })?
        .join("\n")
        + "\n";
    atomic_write(&path, encoded.as_bytes())
}

fn validate_task_outcome_events(
    root: &Path,
    path: &Path,
    expected_repository_id: &str,
    expected_work_item_id: &str,
) -> Result<Vec<TaskOutcomeEvent>, ObserverError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| ObserverError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(ObserverError::State {
            path: path.to_path_buf(),
            message: "Task Outcome event stream must be a regular non-symlink file".into(),
        });
    }
    let text = fs::read_to_string(path).map_err(|source| ObserverError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let mut events = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    let mut finding_fingerprints = BTreeSet::new();
    for (line_number, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let event: TaskOutcomeEvent =
            serde_json::from_str(line).map_err(|error| ObserverError::State {
                path: path.to_path_buf(),
                message: format!(
                    "invalid Task Outcome event at line {}: {error}",
                    line_number + 1
                ),
            })?;
        let is_finding_or_risk = matches!(event.event_type.as_str(), "finding" | "risk");
        let is_explicit_correction = matches!(
            event.event_type.as_str(),
            "event_corrected" | "event_superseded"
        ) && event.correction_of.is_some();
        let correction_relationship_valid = !matches!(
            event.event_type.as_str(),
            "event_corrected" | "event_superseded"
        ) || event.correction_of.is_some();
        let fingerprint_valid = match event.finding_fingerprint.as_deref() {
            Some(fingerprint) => valid_sha256_digest(fingerprint),
            None => !is_finding_or_risk,
        };
        let fingerprint_is_new = event
            .finding_fingerprint
            .as_ref()
            .is_none_or(|fingerprint| {
                !finding_fingerprints.contains(fingerprint) || is_explicit_correction
            });
        if event.schema_version != 1
            || event.repository_id != expected_repository_id
            || event.work_item_id != expected_work_item_id
            || event.event_id.trim().is_empty()
            || event.event_type.trim().is_empty()
            || event.timestamp.trim().is_empty()
            || event.detail.trim().is_empty()
            || !event_detail_is_safe(&event.detail)
            || ids.contains(&event.event_id)
            || event.related_event_ids.iter().any(|id| !ids.contains(id))
            || event
                .correction_of
                .as_ref()
                .is_some_and(|id| !ids.contains(id))
            || !matches!(
                event.event_type.as_str(),
                "blocked"
                    | "finding"
                    | "risk"
                    | "warning"
                    | "confirmation"
                    | "stop"
                    | "resume"
                    | "resolution"
                    | "risk-accepted"
                    | "check-pass-after-fix"
                    | "prevention"
                    | "completed"
                    | "cancelled"
                    | "recovered"
                    | "event_corrected"
                    | "event_superseded"
            )
            || !fingerprint_valid
            || !fingerprint_is_new
            || !correction_relationship_valid
        {
            return Err(ObserverError::State {
                path: path.to_path_buf(),
                message: format!(
                    "Task Outcome event identity or relationship is invalid at line {}",
                    line_number + 1
                ),
            });
        }
        if event
            .evidence_refs
            .iter()
            .any(|reference| reference.starts_with('/') || reference.contains(".."))
        {
            return Err(ObserverError::State {
                path: path.to_path_buf(),
                message: "Task Outcome event evidence reference must be repository-relative".into(),
            });
        }
        ids.insert(event.event_id.clone());
        if let Some(fingerprint) = &event.finding_fingerprint {
            finding_fingerprints.insert(fingerprint.clone());
        }
        events.push(event);
    }
    if events.is_empty() {
        return Err(ObserverError::State {
            path: path.to_path_buf(),
            message: "Task Outcome event stream must contain at least one event".into(),
        });
    }
    let _ = root;
    Ok(events)
}

fn event_id(event_type: &str, detail: &str, timestamp: &str) -> String {
    let input = format!("{event_type}\n{detail}\n{timestamp}");
    format!("event-{}", Digest::sha256_bytes(input.as_bytes()))
}

fn finding_fingerprint(event_type: &str, detail: &str, evidence_refs: &[String]) -> String {
    let normalized_detail = detail.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut refs = evidence_refs.to_vec();
    refs.sort();
    let input = format!("{event_type}\n{normalized_detail}\n{}", refs.join("\n"));
    Digest::sha256_bytes(input.as_bytes()).to_string()
}

fn append_task_outcome_events(
    root: &Path,
    contract: &Contract,
    report: &TaskOutcomeReport,
    allow_recovery_retry: bool,
) -> Result<(), ObserverError> {
    let path = task_outcome_event_path(root, &contract.work_item_id, false);
    let mut events = if fs::symlink_metadata(&path).is_ok() {
        let existing = validate_task_outcome_events(
            root,
            &path,
            &contract.repository_id,
            &contract.work_item_id,
        )?;
        let last_completed = existing
            .iter()
            .rposition(|event| event.event_type == "completed");
        let completed_then_blocked = last_completed.is_some_and(|index| {
            existing[index + 1..]
                .iter()
                .any(|event| event.event_type == "blocked")
        });
        if existing.iter().any(|event| event.event_type == "completed")
            && !(allow_recovery_retry || completed_then_blocked)
        {
            return Err(ObserverError::State {
                path,
                message: "Task Outcome event stream already contains a completion event".into(),
            });
        }
        existing
    } else {
        Vec::new()
    };
    let timestamp = now();
    let mut append = |event_type: &str, detail: &str, evidence_refs: Vec<String>| {
        let fingerprint = matches!(event_type, "finding" | "risk")
            .then(|| finding_fingerprint(event_type, detail, &evidence_refs));
        if fingerprint.as_ref().is_some_and(|fingerprint| {
            events
                .iter()
                .any(|event| event.finding_fingerprint.as_ref() == Some(fingerprint))
        }) {
            return;
        }
        let id = format!(
            "{}-{}",
            event_id(event_type, detail, &timestamp),
            events.len()
        );
        events.push(TaskOutcomeEvent {
            schema_version: 1,
            event_id: id,
            repository_id: contract.repository_id.clone(),
            work_item_id: contract.work_item_id.clone(),
            event_type: event_type.into(),
            timestamp: timestamp.clone(),
            detail: detail.into(),
            evidence_refs,
            related_event_ids: events
                .last()
                .map(|event: &TaskOutcomeEvent| vec![event.event_id.clone()])
                .unwrap_or_default(),
            correction_of: None,
            finding_fingerprint: fingerprint,
        });
    };
    append(
        "completed",
        report
            .sections
            .outcome_summary
            .first()
            .map(|claim| claim.text.as_str())
            .unwrap_or("Task Outcome report generated."),
        report.bindings.evidence_refs.clone(),
    );
    for claim in &report.sections.findings {
        append("finding", &claim.text, claim.evidence_refs.clone());
    }
    for claim in &report.sections.risks {
        append("risk", &claim.text, claim.evidence_refs.clone());
    }
    for claim in &report.sections.warnings {
        append("warning", &claim.text, claim.evidence_refs.clone());
    }
    for claim in &report.sections.forced_stops {
        append("stop", &claim.text, claim.evidence_refs.clone());
    }
    for claim in &report.sections.resolutions {
        append("resolution", &claim.text, claim.evidence_refs.clone());
    }
    for claim in &report.sections.recurrence_prevention {
        append("prevention", &claim.text, claim.evidence_refs.clone());
    }
    for claim in &report.sections.avoided_impact {
        if !claim.inference {
            append(
                "check-pass-after-fix",
                &claim.text,
                claim.evidence_refs.clone(),
            );
        }
    }
    for claim in &report.sections.residual_risks {
        append("risk", &claim.text, claim.evidence_refs.clone());
    }
    let encoded = events
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| ObserverError::State {
            path: path.clone(),
            message: error.to_string(),
        })?
        .join("\n")
        + "\n";
    fs::write(&path, encoded).map_err(|source| ObserverError::Read { path, source })
}

fn task_outcome_markdown(report: &TaskOutcomeReport) -> String {
    let mut output = format!(
        "# Task Outcome Report\n\n- Work Item: `{}`\n- Status: `{}`\n- Human status color: `{}`\n\n",
        report.work_item_id,
        outcome_state_name(&report.status),
        serde_json::to_string(&report.human_status_color)
            .unwrap_or_else(|_| "unknown".into())
            .trim_matches('"')
    );
    let sections = [
        ("Outcome summary", &report.sections.outcome_summary),
        ("Task overview", &report.sections.task_overview),
        ("Delivered changes", &report.sections.delivered_changes),
        ("Findings", &report.sections.findings),
        ("Risks", &report.sections.risks),
        ("Warnings", &report.sections.warnings),
        ("Limitations", &report.sections.limitations),
        ("Interventions", &report.sections.interventions),
        ("Forced stops", &report.sections.forced_stops),
        ("Resolutions", &report.sections.resolutions),
        (
            "Recurrence prevention",
            &report.sections.recurrence_prevention,
        ),
        ("Avoided impact", &report.sections.avoided_impact),
        ("Residual risks", &report.sections.residual_risks),
        ("Human decisions", &report.sections.human_decisions),
        ("Evidence", &report.sections.evidence),
    ];
    for (title, claims) in sections {
        output.push_str(&format!("## {title}\n\n"));
        if claims.is_empty() {
            output.push_str("- None\n\n");
            continue;
        }
        for claim in claims {
            let provenance = if claim.inference { " (inference)" } else { "" };
            output.push_str(&format!("- {}{}\n", claim.text, provenance));
        }
        output.push('\n');
    }
    if let Some(gate) = &report.failed_gate {
        output.push_str(&format!("## Failed gate\n\n- {gate}\n\n"));
    }
    if let Some(recovery) = &report.recovery_condition {
        output.push_str(&format!("## Recovery condition\n\n- {recovery}\n\n"));
    }
    output
}

fn write_task_outcome_artifacts(
    root: &Path,
    work_item_id: &str,
    report: &TaskOutcomeReport,
    replace_existing: bool,
) -> Result<(Digest, Digest), ObserverError> {
    let active = root.join(".ai/work-items/active");
    let report_value = serde_json::to_value(report).map_err(|error| ObserverError::State {
        path: active.clone(),
        message: error.to_string(),
    })?;
    let report_bytes =
        serde_json::to_vec_pretty(&report_value).map_err(|error| ObserverError::State {
            path: active.clone(),
            message: error.to_string(),
        })?;
    let markdown = task_outcome_markdown(report);
    let json_digest = Digest::sha256_bytes(&report_bytes);
    let markdown_digest = Digest::sha256_bytes(markdown.as_bytes());
    let json_path = active.join(format!("{work_item_id}.task-report.json"));
    let markdown_path = active.join(format!("{work_item_id}.task-report.md"));
    for path in [&json_path, &markdown_path] {
        if fs::symlink_metadata(path).is_ok()
            && (!replace_existing || !is_regular_non_symlink(path)?)
        {
            return Err(ObserverError::State {
                path: path.clone(),
                message: "Task Outcome report artifact already exists".into(),
            });
        }
    }
    atomic_write(&json_path, &report_bytes)?;
    if let Err(error) = atomic_write(&markdown_path, markdown.as_bytes()) {
        let _ = fs::remove_file(&json_path);
        return Err(error);
    }
    Ok((json_digest, markdown_digest))
}

fn outcome_v2_internal(
    root: &Path,
    work_item_id: &str,
    current_runtime: Option<&RuntimeContext>,
) -> Result<OutcomeV2, ObserverError> {
    outcome_v2_internal_with_snapshot(root, work_item_id, current_runtime, None)
}

fn outcome_v2_internal_with_snapshot(
    root: &Path,
    work_item_id: &str,
    current_runtime: Option<&RuntimeContext>,
    snapshot_override: Option<(&RepositorySnapshot, &Digest)>,
) -> Result<OutcomeV2, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let active = root.join(".ai/work-items/active");
    let archive = root.join(".ai/work-items/archive");
    let contract_path = [
        active.join(format!("{work_item_id}.contract.json")),
        archive.join(format!("{work_item_id}.contract.json")),
    ]
    .into_iter()
    .find(|path| path.is_file())
    .ok_or_else(|| ObserverError::State {
        path: active.join(format!("{work_item_id}.contract.json")),
        message: "work item contract not found".into(),
    })?;
    let contract = read_contract(&contract_path)?;
    let mut _owned_snapshot = None;
    let snapshot;
    let provided_snapshot_digest = snapshot_override.map(|(_, digest)| digest.clone());
    if let Some((provided_snapshot, _)) = snapshot_override {
        snapshot = provided_snapshot;
    } else {
        let git =
            cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
                path: root.clone(),
                message: error.to_string(),
            })?;
        let captured_snapshot = git.snapshot().map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
        _owned_snapshot = Some(captured_snapshot);
        snapshot = _owned_snapshot.as_ref().expect("captured snapshot");
    }
    let evidence_ref = format!(".ai/evidence/{work_item_id}.verification.json");
    let archived = contract_path
        .parent()
        .is_some_and(|path| path.ends_with("archive"));
    let legacy = legacy_verification_evidence(&root, work_item_id);
    // Archived v2 evidence is immutable historical truth. When the bytes are
    // otherwise valid but were produced by an older Runtime, the current
    // Runtime must not relabel that historical result as a current failure.
    // Active Work Items retain the strict foreign-runtime red path below.
    let historical_runtime = archived
        && current_runtime.is_some()
        && verification_evidence_state(&root, &contract, snapshot, true, None)?
            == EvidenceState::Complete
        && verification_evidence_state(&root, &contract, snapshot, true, current_runtime)?
            != EvidenceState::Complete;
    // A reviewed archive repair intentionally changes the archived Contract
    // digest while preserving the original verification bytes.  A valid
    // append-only revalidation receipt makes this a historical, actionable
    // yellow state instead of misclassifying the old evidence as tampering.
    let archived_revalidation = if archived {
        let decision = load_recovery_decision(&root, work_item_id, current_runtime)
            .ok()
            .flatten();
        decision.filter(|decision| {
            decision.successor_binding_mode.as_deref() == Some("contract_amendment_revalidation")
                && decision
                    .current_contract_digest
                    .as_ref()
                    .is_some_and(|digest| {
                        contract_digest(&contract_path).ok().as_ref() == Some(digest)
                    })
                && decision
                    .current_contract_digest
                    .as_ref()
                    .is_some_and(|digest| digest != &decision.predecessor_contract_digest)
                && decision
                    .predecessor_verification_evidence_digest
                    .as_ref()
                    .is_some_and(|evidence_digest| {
                        validate_archived_revalidation_evidence(
                            &root,
                            work_item_id,
                            &decision.predecessor_contract_digest,
                            evidence_digest,
                        )
                        .is_ok()
                    })
        })
    } else {
        None
    };
    let historical = legacy || historical_runtime || archived_revalidation.is_some();
    let evidence_state = if historical {
        None
    } else {
        Some(verification_evidence_state(
            &root,
            &contract,
            snapshot,
            archived,
            current_runtime,
        )?)
    };
    let (mut state, mut decision_state, mut summary, mut evidence_unknown) = if historical {
        (
            OutcomeState::NotReady,
            DecisionState::Yellow,
            if legacy {
                "Historical verification evidence uses a legacy schema and is not revalidated as a current result."
            } else if archived_revalidation.is_some() {
                "The archived Contract was reviewed and amended; historical evidence is preserved while a successor revalidates the current Contract."
            } else {
                "Historical verification evidence was produced by an older Runtime and is not revalidated as a current result."
            },
            Some(if legacy {
                "legacy_evidence_historical"
            } else if archived_revalidation.is_some() {
                "contract_amendment_revalidation_pending"
            } else {
                "historical_evidence_not_revalidated"
            }),
        )
    } else {
        match evidence_state.expect("non-legacy evidence state exists") {
            EvidenceState::Complete => (
                OutcomeState::Verified,
                DecisionState::Green,
                "Verification evidence is valid; user-visible benefit remains explicitly unknown.",
                None,
            ),
            EvidenceState::Missing => (
                OutcomeState::NotReady,
                DecisionState::Yellow,
                "No verification evidence is present; outcome is not ready.",
                Some("verification_evidence_missing"),
            ),
            EvidenceState::Stale => (
                OutcomeState::NotReady,
                DecisionState::Yellow,
                "Verification evidence is stale for the current repository snapshot; outcome is not ready.",
                Some("evidence_stale"),
            ),
            EvidenceState::Contradictory => (
                OutcomeState::Unknown,
                DecisionState::Red,
                "Verification evidence is contradictory or identity-bound to another context; outcome is stopped.",
                Some("evidence_contradictory"),
            ),
            EvidenceState::Unknown => (
                OutcomeState::Unknown,
                DecisionState::Red,
                "Verification evidence could not be validated; outcome is stopped.",
                Some("evidence_unknown"),
            ),
        }
    };
    let (recovery_decision, recovery_decision_invalid) =
        match load_recovery_decision(&root, work_item_id, current_runtime) {
            Ok(decision) => (decision, false),
            Err(_) => (None, true),
        };
    if archived {
        let archive_manifest = root
            .join(".ai/work-items/archive")
            .join(format!("{work_item_id}.archive.json"));
        let archived_outcome = root
            .join(".ai/work-items/archive")
            .join(format!("{work_item_id}.outcome.json"));
        let report_present = read_json(&archived_outcome)
            .ok()
            .and_then(|value| value.get("taskOutcomeReport").cloned())
            .is_some();
        let events_required = read_json(&archive_manifest)
            .ok()
            .and_then(|value| value.get("files").cloned())
            .and_then(|files| files.get("eventsDigest").cloned())
            .is_some();
        if report_present {
            let manifest = read_json(&archive_manifest).ok();
            let report_valid = manifest.as_ref().is_some_and(|manifest| {
                verify_archive_manifest(&root, work_item_id, manifest).is_ok()
                    || recovery_decision.as_ref().is_some_and(|decision| {
                        decision.decision == "supersede"
                            && decision.predecessor_archive_manifest_digest.is_some()
                            && verify_historical_supersede_manifest(
                                &root,
                                work_item_id,
                                manifest,
                                decision,
                            )
                            .is_ok()
                    })
            });
            let events_valid = !events_required
                || validate_task_outcome_events(
                    &root,
                    &task_outcome_event_path(&root, work_item_id, true),
                    &contract.repository_id,
                    work_item_id,
                )
                .is_ok();
            if !report_valid || !events_valid {
                state = OutcomeState::Unknown;
                decision_state = DecisionState::Red;
                summary = "Archived Task Outcome evidence is malformed or not bound to the archive manifest; outcome is stopped.";
                evidence_unknown = Some("outcome_report_invalid");
            }
        }
    }
    // A normal archived Work Item with a bound resource context is not a
    // terminal success until its provider-side finalization receipt is
    // present and valid.  Keep this as a yellow, actionable state rather
    // than allowing the archived verification receipt alone to appear green.
    // Superseded and historical records are handled by their explicit
    // recovery/compatibility projections below.
    let finalization_pending = archived
        && !historical
        && contract.resource_context.is_some()
        && verify_resource_finalization_internal(&root, work_item_id, current_runtime).is_err();
    if finalization_pending && state == OutcomeState::Verified {
        state = OutcomeState::NotReady;
        decision_state = DecisionState::Yellow;
        summary = "Archived verification is valid, but provider finalization evidence is missing or invalid; outcome is not ready.";
        evidence_unknown = Some("resource_finalization_pending");
    }
    // Archive is not a terminal handoff.  Even when the verification receipt
    // and provider cleanup are valid, the Work Item remains non-terminal until
    // an identity-bound human close decision is recorded.  Project this gap
    // explicitly so agents cannot mistake an archived item for a completed one.
    let close_decision_path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    let close_pending = archived
        && !historical
        && !close_decision_is_valid_for_status(&root, work_item_id, &contract.repository_id);
    if close_pending && state == OutcomeState::Verified {
        state = OutcomeState::NotReady;
        decision_state = DecisionState::Yellow;
        summary = "Archived verification is valid, but the required human close decision is missing or invalid; outcome is not ready.";
        evidence_unknown = Some(if fs::symlink_metadata(&close_decision_path).is_ok() {
            "close_decision_invalid"
        } else {
            "close_decision_pending"
        });
    }
    // A failed lifecycle gate is persisted as an active, repository-bound
    // blocked projection.  Prefer that projection over recomputing the
    // evidence-only view so a failed finish cannot be presented as merely
    // "not ready" (or, worse, as verified after a later evidence change).
    // A fresh verification performed through an explicit recovery retry may
    // replace a blocked active Outcome projection. Treat the projection as
    // reconciled only when the Summary marker matches the current evidence
    // digest and the strict evidence validator still passes.
    let current_summary_path = active.join(format!("{work_item_id}.summary.json"));
    let current_summary = read_json(&current_summary_path).ok();
    let recovery_projection_reconciled = if !archived {
        current_summary
            .as_ref()
            .and_then(|summary| summary["verificationRecoveryReconciled"].as_str())
            .is_some_and(|marker| {
                let evidence_path = root
                    .join(".ai/evidence")
                    .join(format!("{work_item_id}.verification.json"));
                let Some(evidence) = read_json(&evidence_path).ok() else {
                    return false;
                };
                let Ok(digest) = cockpit_protocol::digest_json(&evidence) else {
                    return false;
                };
                marker == digest.to_string()
                    && verification_evidence_state(
                        &root,
                        &contract,
                        snapshot,
                        false,
                        current_runtime,
                    )
                    .is_ok_and(|state| state == EvidenceState::Complete)
            })
    } else {
        false
    };
    let persisted_failure = if !archived && !recovery_projection_reconciled {
        let path = active.join(format!("{work_item_id}.outcome.json"));
        is_regular_non_symlink(&path)
            .ok()
            .filter(|valid| *valid)
            .and_then(|_| read_json(&path).ok())
            .and_then(|value| {
                if value.get("state").and_then(serde_json::Value::as_str) != Some("blocked")
                    || value.get("workItemId").and_then(serde_json::Value::as_str)
                        != Some(work_item_id)
                    || value
                        .get("repositoryId")
                        .and_then(serde_json::Value::as_str)
                        != Some(contract.repository_id.as_str())
                {
                    return None;
                }
                let gate = value
                    .get("failedGate")
                    .and_then(serde_json::Value::as_str)?;
                let recovery = value
                    .get("recoveryCondition")
                    .and_then(serde_json::Value::as_str)?;
                Some((gate.to_owned(), recovery.to_owned()))
            })
    } else {
        None
    };
    if persisted_failure.is_some() {
        state = OutcomeState::Unknown;
        decision_state = DecisionState::Red;
        summary = "A lifecycle gate failed; completion is not claimed and the Work Item remains recoverable.";
        evidence_unknown = Some("lifecycle_gate_failed");
    }
    if recovery_decision_invalid {
        state = OutcomeState::Unknown;
        decision_state = DecisionState::Red;
        summary = "Recovery decision evidence is malformed, foreign, stale, or not bound to the current predecessor; outcome is stopped.";
        evidence_unknown = Some(RECOVERY_DECISION_INVALID);
    }
    let historical_status = if historical {
        Some(if legacy {
            "legacy".to_owned()
        } else if archived_revalidation.is_some() {
            "contract_amendment_revalidation".to_owned()
        } else {
            "runtime_historical".to_owned()
        })
    } else {
        recovery_decision
            .as_ref()
            .filter(|decision| decision.decision == "supersede")
            .map(|_| "superseded".to_owned())
    };
    if historical_status.as_deref() == Some("superseded") {
        state = OutcomeState::Unknown;
        decision_state = DecisionState::Yellow;
        summary = "This Work Item was superseded as historical evidence; its original bytes were preserved and were not revalidated as a current result.";
        evidence_unknown = Some("historical_evidence_not_current");
    }
    let mut unknowns = vec!["user_visible_benefit_not_declared".into()];
    if let Some(code) = evidence_unknown {
        unknowns.push(code.into());
    }
    if contract.acceptance_criteria.is_empty() {
        unknowns.push("acceptanceCriteria".into());
    }
    unknowns.sort();
    unknowns.dedup();
    let report = HumanBenefitReport {
        state: OutcomeState::Unknown,
        user_visible_changes: Vec::new(),
        affected_users: Vec::new(),
        unknowns: vec!["user_visible_benefit_not_declared".into()],
        evidence_refs: vec![evidence_ref.clone()],
    };
    let summary_path = contract_path
        .parent()
        .map(|parent| parent.join(format!("{work_item_id}.summary.json")));
    let summary_value = summary_path
        .as_deref()
        .and_then(|path| read_json(path).ok());
    let task_report = task_outcome_report(TaskOutcomeReportInput {
        root: &root,
        contract_path: &contract_path,
        contract: &contract,
        summary: summary_value.as_ref(),
        snapshot_digest: provided_snapshot_digest.or_else(|| snapshot_digest(snapshot).ok()),
        state: state.clone(),
        decision_state: decision_state.clone(),
        summary_text: summary,
        unknowns: &unknowns,
        evidence_ref: &evidence_ref,
        failed_gate_override: persisted_failure.as_ref().map(|(gate, _)| gate.as_str()),
        recovery_condition_override: persisted_failure
            .as_ref()
            .map(|(_, recovery)| recovery.as_str()),
        historical,
    });
    let failed_gate = task_report.failed_gate.clone();
    let recovery_condition = task_report.recovery_condition.clone();
    Ok(OutcomeV2 {
        schema_version: 2,
        repository_id: contract.repository_id,
        work_item_id: work_item_id.into(),
        state,
        decision_state: Some(decision_state),
        summary: summary.into(),
        acceptance_results: contract.acceptance_criteria,
        unknowns,
        evidence_refs: vec![evidence_ref],
        human_benefit_report: report,
        task_outcome_report: Some(task_report),
        failed_gate,
        recovery_condition,
        recovery_decision,
        historical_status,
        governance_reasons: Vec::new(),
        finalization: None,
    })
}

fn recovery_decision_candidate_paths(
    root: &Path,
    work_item_id: &str,
    archived: bool,
) -> Result<(Vec<PathBuf>, bool), ObserverError> {
    if archived {
        let archive_manifest_path = root
            .join(".ai/work-items/archive")
            .join(format!("{work_item_id}.archive.json"));
        let manifest = read_json(&archive_manifest_path)?;
        if manifest["state"] == serde_json::json!("superseded") {
            let relative = manifest["supersessionDecisionPath"]
                .as_str()
                .ok_or_else(|| {
                    recovery_decision_error(
                        &archive_manifest_path,
                        "historical_binding_missing",
                        "superseded archive manifest has no recovery decision path",
                    )
                })?;
            let relative_path = Path::new(relative);
            let file_name = relative_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            let canonical = format!("{work_item_id}.recovery.json");
            let versioned_prefix = format!("{work_item_id}.recovery.");
            if relative_path.parent() != Some(Path::new(".ai/decisions"))
                || (file_name != canonical
                    && !(file_name.starts_with(&versioned_prefix) && file_name.ends_with(".json")))
            {
                return Err(recovery_decision_error(
                    archive_manifest_path,
                    "historical_binding_mismatch",
                    "superseded archive references a foreign recovery decision path",
                ));
            }
            return Ok((vec![root.join(relative)], true));
        }
    }

    let decisions_dir = root.join(".ai/decisions");
    let entries = match fs::read_dir(&decisions_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((Vec::new(), !archived));
        }
        Err(source) => {
            return Err(ObserverError::Read {
                path: decisions_dir,
                source,
            });
        }
    };
    let canonical = format!("{work_item_id}.recovery.json");
    let versioned_prefix = format!("{work_item_id}.recovery.");
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: decisions_dir.clone(),
            source,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == canonical || (name.starts_with(&versioned_prefix) && name.ends_with(".json")) {
            paths.push(entry.path());
        }
    }
    paths.sort();
    Ok((paths, !archived))
}

fn read_and_validate_recovery_decision(
    root: &Path,
    work_item_id: &str,
    path: &Path,
    current_runtime: Option<&RuntimeContext>,
    contract_path: &Path,
    summary_path: &Path,
) -> Result<RecoveryDecisionReceipt, ObserverError> {
    if !is_regular_non_symlink(path)? {
        return Err(recovery_decision_error(
            path,
            "candidate_not_regular",
            "recovery decision must be a regular non-symlink file",
        ));
    }
    let bytes = fs::read(path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })?;
    if bytes.len() > MAX_EXTERNAL_EVIDENCE_BYTES {
        return Err(recovery_decision_error(
            path,
            "candidate_too_large",
            "recovery decision exceeds the bounded size limit",
        ));
    }
    reject_duplicate_json_keys(&bytes)
        .map_err(|detail| recovery_decision_error(path, "candidate_json_invalid", detail))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| recovery_decision_error(path, "candidate_json_invalid", error))?;
    let receipt: RecoveryDecisionReceipt = serde_json::from_value(value.clone())
        .map_err(|error| recovery_decision_error(path, "candidate_schema_invalid", error))?;

    let canonical = format!("{work_item_id}.recovery.json");
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if name != canonical {
        let digest = cockpit_protocol::digest_json(&value)
            .map_err(|error| recovery_decision_error(path, "candidate_digest_invalid", error))?;
        let digest = digest.to_string();
        let expected = format!(
            "{work_item_id}.recovery.{}.json",
            digest.strip_prefix("sha256:").unwrap_or(&digest)
        );
        if name != expected {
            return Err(recovery_decision_error(
                path,
                "candidate_digest_mismatch",
                "versioned recovery decision filename does not match its content digest",
            ));
        }
    }

    validate_recovery_predecessor_bindings(
        root,
        work_item_id,
        &receipt,
        current_runtime,
        contract_path,
        summary_path,
        Some(path),
    )?;
    let _ = validate_recovery_successor_binding(root, work_item_id, &receipt)?;
    Ok(receipt)
}

fn is_stale_recovery_binding_error(error: &ObserverError) -> bool {
    [
        "predecessor_contract_mismatch",
        "predecessor_summary_mismatch",
        "predecessor_outcome_mismatch",
        "predecessor_events_mismatch",
        "runtime_mismatch",
    ]
    .iter()
    .any(|code| error.to_string().contains(code))
}

/// An archived append-only chain may contain an older successor receipt whose
/// target was never bound (for example, a failed recovery attempt from an
/// older Runtime).  Once a newer, valid `supersede` receipt exists, that
/// historical binding failure must not prevent the predecessor from being
/// projected as superseded.  This exception is intentionally narrow: it only
/// applies to archived records and only when a later valid candidate wins;
/// malformed, foreign, or otherwise untrusted receipts still fail closed.
fn is_historical_successor_binding_error(error: &ObserverError) -> bool {
    let message = error.to_string();
    ["successor_binding_missing", "successor_binding_mismatch"]
        .iter()
        .any(|code| message.contains(code))
        // A pre-strict Runtime could persist the legacy marker even after a
        // successor Contract had gained the mandatory predecessor binding.
        // Treat only this exact, deterministic compatibility error as stale;
        // other successor-binding errors remain fail-closed.
        || message.contains(
            "successor_binding_mode_invalid: legacy successorBindingMode cannot be used by a strictly bound successor",
        )
}

fn load_recovery_decision(
    root: &Path,
    work_item_id: &str,
    current_runtime: Option<&RuntimeContext>,
) -> Result<Option<RecoveryDecisionReceipt>, ObserverError> {
    let contract_path = work_item_artifact_path(root, work_item_id, "contract.json")?;
    let summary_path = work_item_artifact_path(root, work_item_id, "summary.json")?;
    let archived = contract_path
        .parent()
        .is_some_and(|parent| parent.ends_with("archive"));
    let (paths, strict) = recovery_decision_candidate_paths(root, work_item_id, archived)?;
    let mut candidates = Vec::new();
    let mut stale_candidates = Vec::new();
    for path in paths {
        let receipt = match read_and_validate_recovery_decision(
            root,
            work_item_id,
            &path,
            if archived { None } else { current_runtime },
            &contract_path,
            &summary_path,
        ) {
            Ok(receipt) => receipt,
            Err(error) if !strict => {
                // Archived recovery records are historical inputs, but they
                // are still repository-local evidence.  Do not silently
                // ignore malformed, foreign, or tampered candidates: a
                // caller must see the stable invalid-recovery boundary rather
                // than falling through to a weaker finalization path.
                if archived {
                    if !is_stale_recovery_binding_error(&error)
                        && !is_historical_successor_binding_error(&error)
                    {
                        return Err(error);
                    }
                    let retry = read_json(&path).ok().and_then(|value| {
                        serde_json::from_value::<RecoveryDecisionReceipt>(value).ok()
                    });
                    let Some(retry) = retry.filter(|receipt| {
                        matches!(
                            receipt.decision.as_str(),
                            "retry" | "successor" | "supersede"
                        )
                    }) else {
                        return Err(error);
                    };
                    let decided_at = DateTime::parse_from_rfc3339(&retry.decided_at)
                        .expect("recovery validator accepted RFC3339")
                        .timestamp_millis();
                    stale_candidates.push((Some(decided_at), Some(retry.decision), error));
                    continue;
                }
                continue;
            }
            Err(error) => {
                // An append-only recovery chain may contain an older retry
                // receipt whose predecessor bindings became stale after a
                // Contract amendment or Runtime upgrade.  Preserve that
                // historical byte, but allow a newer valid receipt to become
                // the current projection.  Malformed, misnamed, foreign, or
                // otherwise untrusted candidates still fail closed.
                if !is_stale_recovery_binding_error(&error) {
                    return Err(error);
                }
                let parsed = read_json(&path).ok().map(|value| {
                    let decision = value["decision"].as_str().map(str::to_owned);
                    let timestamp = value["decidedAt"]
                        .as_str()
                        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
                        .map(|value| value.timestamp_millis());
                    (timestamp, decision)
                });
                let (timestamp, decision) = parsed.unwrap_or((None, None));
                stale_candidates.push((timestamp, decision, error));
                continue;
            }
        };
        let decided_at = DateTime::parse_from_rfc3339(&receipt.decided_at)
            .expect("recovery validator accepted RFC3339")
            .timestamp_millis();
        candidates.push((decided_at, path, receipt));
    }
    if let Some(latest_valid) = candidates.iter().map(|item| item.0).max() {
        if let Some((_, _, error)) = stale_candidates
            .into_iter()
            .find(|(timestamp, _, _)| timestamp.is_none_or(|value| value >= latest_valid))
        {
            return Err(error);
        }
    } else if stale_candidates
        .iter()
        .all(|(_, decision, _)| decision.as_deref() == Some("retry"))
    {
        // Retry receipts bind the pre-retry Summary by design. Once fresh
        // verification advances that Summary, retain the bytes as history
        // without projecting them as a current recovery decision. A pending
        // marker, however, still requires a matching current receipt.
        let summary = read_json(&summary_path)?;
        if summary["recoveryRetryPending"] == serde_json::json!(true) {
            return Err(recovery_decision_error(
                summary_path,
                "retry_binding_missing",
                "pending retry marker has no valid current recovery receipt",
            ));
        }
        return Ok(None);
    } else if let Some((_, _, error)) = stale_candidates.into_iter().next() {
        return Err(error);
    }
    candidates.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    Ok(candidates.pop().map(|(_, _, receipt)| receipt))
}

/// Return true only for a readable, regular legacy evidence file.  Malformed
/// v2 JSON, symlinks, and v2 records with missing nested identity remain
/// contradictory/red; this predicate is intentionally narrow so current
/// corruption cannot hide behind the historical projection.
fn legacy_verification_evidence(root: &Path, work_item_id: &str) -> bool {
    let path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        return false;
    };
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return false;
    }
    let Ok(value) = read_json(&path) else {
        return false;
    };
    let Some(object) = value.as_object() else {
        return false;
    };
    // A schema-2 envelope with a deleted repositoryId is current corruption,
    // not historical evidence.  Only the absence of the v2 discriminator
    // qualifies for the legacy projection.
    object.get("evidenceSchemaVersion").is_none()
}

/// Derive a repository-local capability truth registry from Observer facts and
/// profile evidence. Detection is not treated as verification unless a
/// repository profile explicitly confirmed the command.
pub fn capability_truth_registry(root: &Path) -> Result<CapabilityTruthRegistry, ObserverError> {
    capability_truth_registry_internal(root, None)
}

pub fn capability_truth_registry_with_runtime(
    root: &Path,
    runtime: &RuntimeContext,
) -> Result<CapabilityTruthRegistry, ObserverError> {
    capability_truth_registry_internal(root, Some(runtime))
}

fn capability_truth_registry_internal(
    root: &Path,
    runtime: Option<&RuntimeContext>,
) -> Result<CapabilityTruthRegistry, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let observation = observe(&root, &snapshot)?;
    let profile_path = root.join(".ai/project.json");
    let profile_is_regular = fs::symlink_metadata(&profile_path)
        .ok()
        .is_some_and(|metadata| metadata.file_type().is_file());
    let expected_repository_id = repository_id(&root).to_string();
    let (profile, profile_unknown) = if !profile_is_regular {
        (None, Some("project_profile_missing".to_string()))
    } else {
        match read_json(&profile_path)
            .ok()
            .and_then(|value| serde_json::from_value::<AttachedProfile>(value).ok())
        {
            Some(profile) if profile.repository_id == expected_repository_id => {
                (Some(profile), None)
            }
            Some(_) => (None, Some("project_profile_repository_mismatch".into())),
            None => (None, Some("project_profile_invalid".into())),
        }
    };
    let snapshot_ref = snapshot_digest(&snapshot)?.to_string();
    let mut capabilities = Vec::new();
    for language in &observation.languages {
        let capability = format!("language:{language:?}").to_ascii_lowercase();
        capabilities.push(CapabilityTruth {
            capability,
            state: TruthState::Observed,
            confidence: CapabilityConfidence::High,
            source: FactOrigin::Observed,
            evidence_refs: vec![format!("repository-snapshot:{snapshot_ref}")],
            verification: None,
            unknowns: Vec::new(),
        });
    }
    for build_system in &observation.build_systems {
        let capability = format!("build:{build_system:?}").to_ascii_lowercase();
        capabilities.push(CapabilityTruth {
            capability,
            state: TruthState::Observed,
            confidence: CapabilityConfidence::High,
            source: FactOrigin::Observed,
            evidence_refs: vec![format!("repository-snapshot:{snapshot_ref}")],
            verification: None,
            unknowns: Vec::new(),
        });
    }
    for command in &observation.quality_commands {
        let key = format!(
            "verification:{} {}",
            command.program,
            command.args.join(" ")
        );
        let confirmed = profile
            .as_ref()
            .is_some_and(|profile| profile.tests.iter().any(|test| test == command));
        capabilities.push(CapabilityTruth {
            capability: key,
            state: if confirmed {
                TruthState::Verified
            } else {
                TruthState::Observed
            },
            confidence: if confirmed {
                CapabilityConfidence::High
            } else {
                CapabilityConfidence::Medium
            },
            source: if confirmed {
                FactOrigin::Declared
            } else {
                FactOrigin::Observed
            },
            evidence_refs: vec![
                ".ai/project.json".into(),
                format!("repository-snapshot:{snapshot_ref}"),
            ],
            verification: confirmed.then(|| "project-profile-confirmed".into()),
            unknowns: if confirmed {
                Vec::new()
            } else if let Some(unknown) = &profile_unknown {
                vec![unknown.clone()]
            } else {
                vec!["command_not_profile_confirmed".into()]
            },
        });
    }
    capabilities.sort_by(|left, right| left.capability.cmp(&right.capability));
    capabilities.dedup_by(|left, right| left.capability == right.capability);
    let unknown_runtime_digest = Digest::sha256_bytes(b"runtime_identity_not_supplied");
    let runtime_version = runtime
        .map(|value| value.runtime_version.clone())
        .unwrap_or_else(|| "unknown".into());
    let runtime_digest = runtime
        .map(|value| value.runtime_digest.clone())
        .unwrap_or_else(|| unknown_runtime_digest.clone());
    let interface_path = root.join(".ai/agent-interface.json");
    let interface_is_regular = fs::symlink_metadata(&interface_path)
        .ok()
        .is_some_and(|metadata| metadata.file_type().is_file());
    let (interface_valid, interface_unknown) = if !interface_is_regular {
        (false, Some("agent_interface_missing".to_string()))
    } else {
        match read_json(&interface_path)
            .ok()
            .and_then(|value| serde_json::from_value::<AgentInterfaceManifest>(value).ok())
        {
            Some(manifest) if manifest.repository_id != expected_repository_id => {
                (false, Some("agent_interface_repository_mismatch".into()))
            }
            Some(manifest)
                if manifest.protocol_version != cockpit_protocol::PROTOCOL_VERSION
                    || manifest.root_binding.binding_type != "manifest-parent"
                    || !manifest.interfaces.cli.available =>
            {
                (false, Some("agent_interface_invalid".into()))
            }
            Some(_) => (true, None),
            None => (false, Some("agent_interface_invalid".into())),
        }
    };
    let state = if runtime.is_some() && interface_valid {
        AdopterCapabilityState::RepositoryBound
    } else {
        AdopterCapabilityState::Unknown
    };
    let runtime_ref = format!("runtime:{runtime_digest}");
    let mut adopter_unknowns = if runtime.is_some() {
        Vec::new()
    } else {
        vec!["runtime_identity_not_supplied".into()]
    };
    if let Some(unknown) = interface_unknown {
        adopter_unknowns.push(unknown);
    }
    adopter_unknowns.sort();
    adopter_unknowns.dedup();
    let mut registry_unknowns = adopter_unknowns.clone();
    if let Some(unknown) = profile_unknown {
        registry_unknowns.push(unknown);
    }
    registry_unknowns.sort();
    registry_unknowns.dedup();
    let mut adopter_capabilities = [
        "capability_manifest",
        "governance_cost_metrics",
        "implementation_knowledge_query",
        "implementation_knowledge_reports",
        "repository_observe",
        "repository_status",
        "work_item_status_aggregation",
        "work_item_status_interface",
    ]
    .into_iter()
    .map(|id| AdopterCapabilityTruth {
        id: id.into(),
        state: state.clone(),
        ownership: CapabilityOwnership::Runtime,
        adopter_facing: true,
        evidence_refs: vec![runtime_ref.clone(), ".ai/agent-interface.json".into()],
        unknowns: adopter_unknowns.clone(),
    })
    .collect::<Vec<_>>();
    adopter_capabilities.sort_by(|left, right| left.id.cmp(&right.id));
    let mut exclusions = [
        ("codeql", CapabilityOwnership::ExternalProvider),
        (
            "digital_signing",
            CapabilityOwnership::AdopterOrReleaseDomain,
        ),
        (
            "enterprise_iam",
            CapabilityOwnership::AdopterOrReleaseDomain,
        ),
        ("external_audit", CapabilityOwnership::ExternalProvider),
        ("hosted_ci", CapabilityOwnership::ExternalProvider),
        (
            "production_sandbox",
            CapabilityOwnership::AdopterOrReleaseDomain,
        ),
        ("provenance", CapabilityOwnership::AdopterOrReleaseDomain),
        ("sbom", CapabilityOwnership::AdopterOrReleaseDomain),
    ]
    .into_iter()
    .map(|(id, ownership)| CapabilityExclusion {
        id: id.into(),
        ownership,
        reason: "External evidence is not proven by this repository-local Runtime projection."
            .into(),
    })
    .collect::<Vec<_>>();
    exclusions.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(CapabilityTruthRegistry {
        schema_version: 1,
        repository_id: repository_id(&root).to_string(),
        snapshot_digest: snapshot_digest(&snapshot)?,
        runtime_version,
        runtime_digest,
        capabilities,
        adopter_capabilities,
        exclusions,
        unknowns: registry_unknowns,
        project_governance: Some(project_governance_projection(&root, &snapshot)?),
    })
}

/// Summarize measurable governance cost from one fresh snapshot and, when
/// requested, one bound verification receipt. Missing measurements remain
/// unknown instead of being replaced with benchmark assumptions.
pub fn performance_diagnosis(
    root: &Path,
    work_item_id: Option<&str>,
) -> Result<PerformanceDiagnosis, ObserverError> {
    let total_start = Instant::now();
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let mut phases = Vec::new();
    let identity_start = Instant::now();
    let repository = repository_id(&root);
    phases.push(PerformancePhase {
        name: "identity".into(),
        elapsed_ns: identity_start.elapsed().as_nanos(),
        measurement: "runtime_internal".into(),
        parent: None,
    });
    let git_start = Instant::now();
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    phases.push(PerformancePhase {
        name: "git_snapshot".into(),
        elapsed_ns: git_start.elapsed().as_nanos(),
        measurement: "runtime_internal".into(),
        parent: None,
    });
    let mut cost = GovernanceCost {
        snapshot_git_calls: snapshot.git_calls,
        snapshot_files_read: snapshot.files_read,
        snapshot_files_hashed: snapshot.files_hashed,
        verification_runs: 0,
        verification_nodes_executed: 0,
        verification_nodes_reused: 0,
        elapsed_ms: 0,
    };
    let mut evidence_refs = vec!["repository-snapshot".into()];
    let mut unknowns = Vec::new();
    let mut read_bytes = snapshot.bytes_read;
    let mut hashed_bytes = snapshot.bytes_hashed;
    let read_hash_start = Instant::now();
    let mut evidence_bytes = None;
    if let Some(work_item_id) = work_item_id {
        let path = root
            .join(".ai/evidence")
            .join(format!("{work_item_id}.verification.json"));
        match fs::read(&path) {
            Ok(bytes) => {
                let digest = Digest::sha256_bytes(&bytes);
                read_bytes = read_bytes.saturating_add(bytes.len() as u64);
                hashed_bytes = hashed_bytes.saturating_add(bytes.len() as u64);
                evidence_bytes = Some((bytes, digest));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                unknowns.push("verification_receipt_missing".into())
            }
            Err(_) => unknowns.push("verification_receipt_unreadable".into()),
        }
    } else {
        unknowns.push("work_item_not_selected".into());
    }
    phases.push(PerformancePhase {
        name: "read_hash".into(),
        elapsed_ns: read_hash_start.elapsed().as_nanos(),
        measurement: "runtime_internal".into(),
        parent: None,
    });
    let parse_start = Instant::now();
    if let (Some((bytes, _digest)), Some(work_item_id)) = (evidence_bytes.as_ref(), work_item_id) {
        match serde_json::from_slice::<serde_json::Value>(bytes) {
            Ok(evidence) => {
                cost.verification_runs = 1;
                let receipt = evidence.get("receipt").unwrap_or(&evidence);
                cost.verification_nodes_executed =
                    receipt["nodesExecuted"].as_u64().unwrap_or(0) as usize;
                cost.verification_nodes_reused =
                    receipt["nodesReused"].as_u64().unwrap_or(0) as usize;
                cost.elapsed_ms = receipt["elapsedMs"].as_u64().unwrap_or(0) as u128;
                evidence_refs.push(format!(".ai/evidence/{work_item_id}.verification.json"));
            }
            Err(_) => unknowns.push("verification_receipt_invalid".into()),
        }
    }
    phases.push(PerformancePhase {
        name: "parse".into(),
        elapsed_ns: parse_start.elapsed().as_nanos(),
        measurement: "runtime_internal".into(),
        parent: None,
    });
    let governance_start = Instant::now();
    let mut projected_outcome = None;
    if let Some(work_item_id) = work_item_id {
        if validate_work_item_governance_controls(&root, work_item_id).is_err() {
            unknowns.push("governance_projection_unavailable".into());
        }
        if let Ok(snapshot_digest) = snapshot_digest(&snapshot) {
            match outcome_v2_internal_with_snapshot(
                &root,
                work_item_id,
                None,
                Some((&snapshot, &snapshot_digest)),
            ) {
                Ok(outcome) => projected_outcome = Some(outcome),
                Err(_) => unknowns.push("outcome_projection_unavailable".into()),
            }
        } else {
            unknowns.push("snapshot_digest_unavailable".into());
        }
    }
    phases.push(PerformancePhase {
        name: "governance".into(),
        elapsed_ns: governance_start.elapsed().as_nanos(),
        measurement: "runtime_internal".into(),
        parent: None,
    });
    let projection_start = Instant::now();
    if let Some(outcome) = projected_outcome.as_ref()
        && serde_json::to_vec(outcome).is_err()
    {
        unknowns.push("outcome_serialization_failed".into());
    }
    if serde_json::to_vec(&snapshot).is_err() {
        unknowns.push("snapshot_serialization_failed".into());
    }
    phases.push(PerformancePhase {
        name: "projection_serialization".into(),
        elapsed_ns: projection_start.elapsed().as_nanos(),
        measurement: "runtime_internal".into(),
        parent: None,
    });
    // The Runtime does not own a process supervisor for this read-only route,
    // so a child-process count is explicitly unavailable rather than zero.
    unknowns.push("child_process_count_unavailable_runtime_internal".into());
    let total_elapsed_ns = total_start.elapsed().as_nanos();
    for phase in &mut phases {
        phase.parent = Some("runtime_total".into());
    }
    phases.push(PerformancePhase {
        name: "runtime_total".into(),
        elapsed_ns: total_elapsed_ns,
        measurement: "runtime_internal".into(),
        parent: None,
    });
    let mut bottlenecks = Vec::new();
    if cost.snapshot_files_hashed > 1000 {
        bottlenecks.push("snapshot_hashing".into());
    }
    if cost.verification_nodes_executed > 0 && cost.verification_nodes_reused == 0 {
        bottlenecks.push("verification_reuse_not_observed".into());
    }
    let state = if unknowns.is_empty() {
        DiagnosisState::Known
    } else {
        DiagnosisState::Unknown
    };
    Ok(PerformanceDiagnosis {
        schema_version: 1,
        repository_id: repository.to_string(),
        work_item_id: work_item_id.map(str::to_owned),
        state,
        cost,
        bottlenecks,
        unknowns,
        evidence_refs,
        phases,
        counters: PerformanceCounters {
            read_bytes: Some(read_bytes),
            hashed_bytes: Some(hashed_bytes),
            git_calls: Some(snapshot.git_calls.saturating_add(1)),
            child_processes: None,
        },
        measurement_scope: format!(
            "runtime_internal; total_elapsed_ms={}",
            total_elapsed_ns / 1_000_000
        ),
    })
}

fn read_work_item_intelligence(
    root: &Path,
    work_item_id: &str,
) -> Result<Option<WorkItemIntelligence>, ObserverError> {
    let path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.intelligence.json"));
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(ObserverError::State {
                path,
                message: "Work Item intelligence sidecar must not be a symlink".into(),
            });
        }
        Ok(metadata) if !metadata.is_file() => {
            return Err(ObserverError::State {
                path,
                message: "Work Item intelligence sidecar must be a regular file".into(),
            });
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(ObserverError::Read { path, source }),
    }
    let value = read_json(&path)?;
    serde_json::from_value(value)
        .map(Some)
        .map_err(|error| ObserverError::State {
            path,
            message: error.to_string(),
        })
}

/// Persist an explicit, repository-bound parallelism declaration next to an
/// active Contract.  This is deliberately separate from the Runtime process:
/// two repositories can declare identically named Work Items independently.
pub fn set_work_item_intelligence(
    root: &Path,
    work_item_id: &str,
    depends_on: Vec<String>,
    conflicts_with: Vec<String>,
    parallelizable: bool,
) -> Result<WorkItemIntelligence, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let contract = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    if !contract.is_file() {
        return Err(ObserverError::State {
            path: contract,
            message: "active work item contract not found".into(),
        });
    }
    let intelligence = WorkItemIntelligence {
        schema_version: 1,
        repository_id: repository_id(&root).to_string(),
        work_item_id: work_item_id.into(),
        depends_on: sorted_unique(depends_on),
        conflicts_with: sorted_unique(conflicts_with),
        parallelizable,
        unknowns: Vec::new(),
    };
    let path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.intelligence.json"));
    atomic_json(
        &path,
        &serde_json::to_value(&intelligence).map_err(|error| ObserverError::State {
            path: path.clone(),
            message: error.to_string(),
        })?,
    )?;
    Ok(intelligence)
}

/// Bind an explicit parallelism boundary to the active Contract.  The
/// Contract is the authority; the legacy intelligence sidecar remains the
/// compatibility projection for dependency/conflict declarations.
pub fn set_work_item_concurrency_boundary(
    root: &Path,
    work_item_id: &str,
    boundary: ConcurrencyBoundary,
) -> Result<ConcurrencyBoundary, ObserverError> {
    validate_work_item_id(work_item_id)?;
    validate_boundary_for_parallel_use(&boundary).map_err(|message| ObserverError::State {
        path: root.join(".ai/work-items/active"),
        message,
    })?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    if !is_regular_non_symlink(&path)? {
        return Err(ObserverError::State {
            path,
            message: "active work item contract not found or is not a regular file".into(),
        });
    }
    let mut value = read_json(&path)?;
    let object = value.as_object_mut().ok_or_else(|| ObserverError::State {
        path: path.clone(),
        message: "work item contract must be a JSON object".into(),
    })?;
    let stored_repository_id = object
        .get("repositoryId")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if stored_repository_id != repository_id(&root).to_string() {
        return Err(ObserverError::State {
            path,
            message: "work item contract repository identity does not match repository".into(),
        });
    }
    object.insert(
        "concurrencyBoundary".into(),
        serde_json::to_value(&boundary).map_err(|error| ObserverError::State {
            path: path.clone(),
            message: error.to_string(),
        })?,
    );
    atomic_json(&path, &value)?;
    Ok(boundary)
}

fn read_contract_boundary(
    root: &Path,
    work_item_id: &str,
) -> Result<Option<ConcurrencyBoundary>, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    if !is_regular_non_symlink(&path)? {
        return Err(ObserverError::State {
            path,
            message: "active work item contract not found or is not a regular file".into(),
        });
    }
    let value = read_json(&path)?;
    let Some(boundary) = value.get("concurrencyBoundary") else {
        return Ok(None);
    };
    let boundary: ConcurrencyBoundary =
        serde_json::from_value(boundary.clone()).map_err(|error| ObserverError::State {
            path: path.clone(),
            message: format!("invalid concurrencyBoundary: {error}"),
        })?;
    validate_boundary_for_parallel_use(&boundary)
        .map_err(|message| ObserverError::State { path, message })?;
    Ok(Some(boundary))
}

fn validate_boundary_for_parallel_use(boundary: &ConcurrencyBoundary) -> Result<(), String> {
    boundary.validate()?;
    for (kind, raw_path) in boundary.all_paths() {
        let normalized = normalized_scope_pattern(raw_path);
        if normalized.is_empty() || scope_pattern_is_unsafe(raw_path) {
            return Err(format!(
                "concurrency boundary {kind} contains an unsafe path"
            ));
        }
        if scope_pattern_has_glob(&normalized)
            && normalized != "*"
            && normalized != "**"
            && simple_scope_prefix(&normalized).is_none()
        {
            return Err(format!(
                "concurrency boundary {kind} contains an unsupported glob"
            ));
        }
    }
    Ok(())
}

fn is_regular_non_symlink(path: &Path) -> Result<bool, ObserverError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            ObserverError::State {
                path: path.into(),
                message: "path does not exist".into(),
            }
        } else {
            ObserverError::Read {
                path: path.into(),
                source,
            }
        }
    })?;
    Ok(metadata.file_type().is_file())
}

fn optional_regular_artifact(path: &Path, label: &str) -> Result<bool, ObserverError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(ObserverError::State {
            path: path.into(),
            message: format!("{label} must be a regular non-symlink file"),
        }),
        Ok(metadata) if !metadata.is_file() => Err(ObserverError::State {
            path: path.into(),
            message: format!("{label} must be a regular non-symlink file"),
        }),
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(ObserverError::Read {
            path: path.into(),
            source,
        }),
    }
}

fn parallel_state_root(root: &Path) -> PathBuf {
    root.join(".ai/parallel")
}

fn parallel_leases_root(root: &Path) -> PathBuf {
    parallel_state_root(root).join("leases")
}

fn ensure_parallel_directories(root: &Path) -> Result<PathBuf, ObserverError> {
    let state = parallel_state_root(root);
    let leases = parallel_leases_root(root);
    for path in [&state, &leases] {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(ObserverError::State {
                    path: path.clone(),
                    message: "parallel state path must not be a symlink".into(),
                });
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(ObserverError::State {
                    path: path.clone(),
                    message: "parallel state path must be a directory".into(),
                });
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir_all(path).map_err(|source| ObserverError::Read {
                    path: path.clone(),
                    source,
                })?;
            }
            Err(source) => {
                return Err(ObserverError::Read {
                    path: path.clone(),
                    source,
                });
            }
        }
    }
    // Re-check after create_dir_all: another first-use acquirer may have
    // created the path between metadata and creation, and a symlink must
    // never become an accepted parallel-state root.
    for path in [&state, &leases] {
        let metadata = fs::symlink_metadata(path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(ObserverError::State {
                path: path.clone(),
                message: "parallel state path must be a non-symlink directory".into(),
            });
        }
    }
    Ok(leases)
}

fn slot_lease_path(root: &Path, slot_id: u32) -> PathBuf {
    parallel_leases_root(root).join(format!("slot-{slot_id}.json"))
}

fn parallel_slot_lease_id() -> String {
    let sequence = NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    format!("{}-{timestamp}-{sequence}", std::process::id())
}

/// Publish a newly acquired lease without exposing partially written JSON.
///
/// The final slot path is installed with a hard link only after the temporary
/// file has been fully written and synced.  A hard link is used instead of a
/// rename so a competing Work Item cannot overwrite the winner's lease bytes.
fn publish_parallel_slot_lease(path: &Path, bytes: &[u8]) -> Result<bool, ObserverError> {
    let sequence = NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed);
    let temporary = path.with_extension(format!("tmp-{}-{sequence}", std::process::id()));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|source| ObserverError::Read {
                path: temporary.clone(),
                source,
            })?;
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|source| ObserverError::Read {
                path: temporary.clone(),
                source,
            })?;
        drop(file);
        match fs::hard_link(&temporary, path) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
            Err(source) => Err(ObserverError::Read {
                path: path.into(),
                source,
            }),
        }
    })();
    let cleanup = fs::remove_file(&temporary);
    match cleanup {
        Ok(()) => result,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => result,
        Err(source) => match result {
            Ok(_) => Err(ObserverError::Read {
                path: temporary,
                source,
            }),
            Err(error) => Err(error),
        },
    }
}

fn read_parallel_slot_lease(path: &Path) -> Result<ParallelSlotLease, ObserverError> {
    if !is_regular_non_symlink(path)? {
        return Err(ObserverError::State {
            path: path.into(),
            message: "parallel slot lease must be a regular non-symlink file".into(),
        });
    }
    let value = read_json(path)?;
    let lease: ParallelSlotLease =
        serde_json::from_value(value).map_err(|error| ObserverError::State {
            path: path.into(),
            message: format!("invalid parallel slot lease: {error}"),
        })?;
    if lease.schema_version != PARALLEL_SLOT_LEASE_SCHEMA_VERSION
        || lease.work_item_id.trim().is_empty()
        || lease.lease_id.trim().is_empty()
        || lease.max_workers == 0
        || lease.slot_id >= lease.max_workers
    {
        return Err(ObserverError::State {
            path: path.into(),
            message: "invalid parallel slot lease identity or capacity".into(),
        });
    }
    Ok(lease)
}

/// Acquire exactly one repository-local parallel execution slot.  A Work
/// Item may hold only one lease and stale/malformed leases are fail-closed;
/// there is no implicit expiry that could create a concurrent write window.
pub fn acquire_parallel_slot(
    root: &Path,
    work_item_id: &str,
) -> Result<ParallelSlotLease, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let boundary =
        read_contract_boundary(&root, work_item_id)?.ok_or_else(|| ObserverError::State {
            path: root
                .join(".ai/work-items/active")
                .join(format!("{work_item_id}.contract.json")),
            message:
                "concurrency boundary is not declared; parallel slot acquisition is serialized"
                    .into(),
        })?;
    let intelligence =
        read_work_item_intelligence(&root, work_item_id)?.ok_or_else(|| ObserverError::State {
            path: root
                .join(".ai/work-items/active")
                .join(format!("{work_item_id}.intelligence.json")),
            message: "parallel compatibility declaration is missing".into(),
        })?;
    if !intelligence.parallelizable {
        return Err(ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: "Work Item is not declared parallelizable".into(),
        });
    }
    let leases = ensure_parallel_directories(&root)?;
    let reservation_path = parallel_state_root(&root).join(format!(".{work_item_id}.slot.reserve"));
    let mut reservation = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&reservation_path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(ObserverError::State {
                path: reservation_path,
                message: "parallel slot reservation is already active".into(),
            });
        }
        Err(source) => {
            return Err(ObserverError::Read {
                path: reservation_path,
                source,
            });
        }
    };
    let reservation_result = reservation
        .write_all(b"ai-cockpit parallel slot reservation\n")
        .and_then(|()| reservation.sync_all());
    if let Err(source) = reservation_result {
        drop(reservation);
        let _ = fs::remove_file(&reservation_path);
        return Err(ObserverError::Read {
            path: reservation_path,
            source,
        });
    }
    drop(reservation);

    let result = (|| {
        for slot_id in 0..boundary.max_workers {
            let path = slot_lease_path(&root, slot_id);
            if path.exists() {
                let existing = read_parallel_slot_lease(&path)?;
                if existing.repository_id != repository_id(&root).to_string() {
                    return Err(ObserverError::State {
                        path,
                        message: "parallel slot lease repository identity mismatch".into(),
                    });
                }
                if existing.max_workers != boundary.max_workers {
                    return Err(ObserverError::State {
                        path,
                        message: "parallel slot lease capacity conflicts with Contract".into(),
                    });
                }
                if existing.work_item_id == work_item_id {
                    return Err(ObserverError::State {
                        path,
                        message: "Work Item already owns a parallel slot".into(),
                    });
                }
                continue;
            }
            let lease = ParallelSlotLease {
                schema_version: PARALLEL_SLOT_LEASE_SCHEMA_VERSION,
                repository_id: repository_id(&root).to_string(),
                work_item_id: work_item_id.into(),
                slot_id,
                lease_id: parallel_slot_lease_id(),
                max_workers: boundary.max_workers,
                acquired_at: now(),
            };
            let bytes =
                serde_json::to_vec_pretty(&lease).map_err(|error| ObserverError::State {
                    path: path.clone(),
                    message: error.to_string(),
                })?;
            if publish_parallel_slot_lease(&path, &bytes)? {
                return Ok(lease);
            }
        }
        Err(ObserverError::State {
            path: leases,
            message: "no parallel slots available".into(),
        })
    })();
    let _ = fs::remove_file(&reservation_path);
    result
}

/// Release a lease only when both Work Item and lease identity match.  A
/// caller cannot release another Work Item's slot by guessing a slot number.
pub fn release_parallel_slot(
    root: &Path,
    work_item_id: &str,
    lease_id: &str,
) -> Result<ParallelSlotLease, ObserverError> {
    validate_work_item_id(work_item_id)?;
    if lease_id.trim().is_empty() {
        return Err(ObserverError::State {
            path: root.join(".ai/parallel/leases"),
            message: "lease id must not be empty".into(),
        });
    }
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let leases = ensure_parallel_directories(&root)?;
    let entries = fs::read_dir(&leases).map_err(|source| ObserverError::Read {
        path: leases.clone(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: leases.clone(),
            source,
        })?;
        let path = entry.path();
        if !path.file_name().is_some_and(|name| {
            name.to_string_lossy().starts_with("slot-") && name.to_string_lossy().ends_with(".json")
        }) {
            continue;
        }
        let lease = read_parallel_slot_lease(&path)?;
        if lease.repository_id != repository_id(&root).to_string() {
            return Err(ObserverError::State {
                path,
                message: "parallel slot lease repository identity mismatch".into(),
            });
        }
        if lease.work_item_id == work_item_id && lease.lease_id == lease_id {
            fs::remove_file(&path).map_err(|source| ObserverError::Read {
                path: path.clone(),
                source,
            })?;
            return Ok(lease);
        }
    }
    Err(ObserverError::State {
        path: leases,
        message: "matching parallel slot lease not found".into(),
    })
}

/// Read all repository-local leases in deterministic slot order.  Any
/// malformed or symlink lease is an error rather than an ignored slot.
pub fn list_parallel_slots(root: &Path) -> Result<Vec<ParallelSlotLease>, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let state = parallel_state_root(&root);
    match fs::symlink_metadata(&state) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(ObserverError::State {
                path: state,
                message: "parallel state path must not be a symlink".into(),
            });
        }
        Ok(metadata) if !metadata.is_dir() => {
            return Err(ObserverError::State {
                path: state,
                message: "parallel state path must be a directory".into(),
            });
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(ObserverError::Read {
                path: state,
                source,
            });
        }
    }
    let leases = parallel_leases_root(&root);
    match fs::symlink_metadata(&leases) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(ObserverError::State {
                path: leases,
                message: "parallel leases path must not be a symlink".into(),
            });
        }
        Ok(metadata) if !metadata.is_dir() => {
            return Err(ObserverError::State {
                path: leases,
                message: "parallel leases path must be a directory".into(),
            });
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(ObserverError::Read {
                path: leases,
                source,
            });
        }
    }
    let entries = fs::read_dir(&leases).map_err(|source| ObserverError::Read {
        path: leases.clone(),
        source,
    })?;
    let mut result = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: leases.clone(),
            source,
        })?;
        let path = entry.path();
        if !path.file_name().is_some_and(|name| {
            name.to_string_lossy().starts_with("slot-") && name.to_string_lossy().ends_with(".json")
        }) {
            continue;
        }
        let lease = read_parallel_slot_lease(&path)?;
        if lease.repository_id != repository_id(&root).to_string() {
            return Err(ObserverError::State {
                path,
                message: "parallel slot lease repository identity mismatch".into(),
            });
        }
        result.push(lease);
    }
    result.sort_by_key(|lease| lease.slot_id);
    Ok(result)
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScopeRelation {
    Overlap,
    Disjoint,
    Unknown,
}

fn normalized_scope_pattern(value: &str) -> String {
    value
        .trim()
        .replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect::<Vec<_>>()
        .join("/")
}

fn scope_pattern_has_glob(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character, '*' | '?' | '[' | ']' | '{' | '}'))
}

fn scope_pattern_is_unsafe(value: &str) -> bool {
    value.starts_with('/')
        || (value.len() >= 2 && value.as_bytes()[1] == b':')
        || value.split('/').any(|part| part == "..")
}

fn simple_scope_prefix(value: &str) -> Option<&str> {
    if value == "**" {
        return Some("");
    }
    let prefix = value.strip_suffix("/**")?;
    (!prefix.is_empty() && !scope_pattern_has_glob(prefix)).then_some(prefix)
}

fn exact_path_is_under_prefix(path: &str, prefix: &str) -> bool {
    path == prefix || path.starts_with(&format!("{prefix}/"))
}

fn scope_pattern_relation(left: &str, right: &str) -> ScopeRelation {
    let left_raw = left.trim().replace('\\', "/");
    let right_raw = right.trim().replace('\\', "/");
    if scope_pattern_is_unsafe(&left_raw) || scope_pattern_is_unsafe(&right_raw) {
        return ScopeRelation::Unknown;
    }
    let left = normalized_scope_pattern(left);
    let right = normalized_scope_pattern(right);
    if left.is_empty() || right.is_empty() {
        return ScopeRelation::Unknown;
    }
    if left == right {
        return ScopeRelation::Overlap;
    }
    if matches!(left.as_str(), "*" | "**") || matches!(right.as_str(), "*" | "**") {
        return ScopeRelation::Overlap;
    }

    let left_exact = !scope_pattern_has_glob(&left);
    let right_exact = !scope_pattern_has_glob(&right);
    if left_exact && right_exact {
        return ScopeRelation::Disjoint;
    }

    let left_prefix = simple_scope_prefix(&left);
    let right_prefix = simple_scope_prefix(&right);
    match (left_prefix, right_prefix, left_exact, right_exact) {
        (Some(left), Some(right), _, _) => {
            if left.is_empty()
                || right.is_empty()
                || left == right
                || left.starts_with(&format!("{right}/"))
                || right.starts_with(&format!("{left}/"))
            {
                ScopeRelation::Overlap
            } else {
                ScopeRelation::Disjoint
            }
        }
        (Some(prefix), _, _, true) => {
            if exact_path_is_under_prefix(&right, prefix) {
                ScopeRelation::Overlap
            } else {
                ScopeRelation::Disjoint
            }
        }
        (_, Some(prefix), true, _) => {
            if exact_path_is_under_prefix(&left, prefix) {
                ScopeRelation::Overlap
            } else {
                ScopeRelation::Disjoint
            }
        }
        _ => ScopeRelation::Unknown,
    }
}

fn scope_list_relation(left: &[String], right: &[String]) -> ScopeRelation {
    if left.is_empty() || right.is_empty() {
        return ScopeRelation::Unknown;
    }
    let mut unknown = false;
    for left_pattern in left {
        for right_pattern in right {
            match scope_pattern_relation(left_pattern, right_pattern) {
                ScopeRelation::Overlap => return ScopeRelation::Overlap,
                ScopeRelation::Unknown => unknown = true,
                ScopeRelation::Disjoint => {}
            }
        }
    }
    if unknown {
        ScopeRelation::Unknown
    } else {
        ScopeRelation::Disjoint
    }
}

fn concurrency_boundary_relation(
    left: &ConcurrencyBoundary,
    right: &ConcurrencyBoundary,
) -> (ScopeRelation, Option<String>) {
    let mut unknown = false;
    for (left_kind, left_path) in left.all_paths() {
        for (right_kind, right_path) in right.all_paths() {
            match scope_pattern_relation(left_path, right_path) {
                ScopeRelation::Overlap => {
                    return (
                        ScopeRelation::Overlap,
                        Some(format!("{left_kind}/{left_path}↔{right_kind}/{right_path}")),
                    );
                }
                ScopeRelation::Unknown => unknown = true,
                ScopeRelation::Disjoint => {}
            }
        }
    }
    if unknown {
        (ScopeRelation::Unknown, None)
    } else {
        (ScopeRelation::Disjoint, None)
    }
}

/// Compare one active Work Item against other active Work Items using only
/// explicit sidecar dependencies/conflicts and declared scopes. Missing
/// intelligence is reported as unknown and cannot silently authorize parallel
/// execution.
pub fn work_item_compatibility(
    root: &Path,
    work_item_id: &str,
) -> Result<WorkItemCompatibility, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let active = root.join(".ai/work-items/active");
    let contract_path = active.join(format!("{work_item_id}.contract.json"));
    if !is_regular_non_symlink(&contract_path)? {
        return Err(ObserverError::State {
            path: contract_path,
            message: "active work item contract not found".into(),
        });
    }
    let target: serde_json::Value = read_json(&contract_path)?;
    let target_scope = target["scope"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let target_boundary = read_contract_boundary(&root, work_item_id)?;
    let intelligence = read_work_item_intelligence(&root, work_item_id)?;
    let mut reasons = Vec::new();
    let mut conflicts = Vec::new();
    let mut dependencies_satisfied = true;
    let mut unknowns = Vec::new();
    if target_scope.is_empty() && target_boundary.is_none() {
        reasons.push("scope_overlap_unknown:empty_target_scope".into());
    }
    let Some(intelligence) = intelligence else {
        return Ok(WorkItemCompatibility {
            repository_id: repository_id(&root).to_string(),
            work_item_id: work_item_id.into(),
            compatible: false,
            dependencies_satisfied: false,
            conflicts,
            reasons: vec!["parallel_compatibility_not_declared".into()],
        });
    };
    for dependency in &intelligence.depends_on {
        let path = active.join(format!("{dependency}.contract.json"));
        if path.is_file() {
            dependencies_satisfied = false;
            reasons.push(format!("dependency_active:{dependency}"));
        } else {
            unknowns.push(format!("dependency_not_observed:{dependency}"));
        }
    }
    let entries = fs::read_dir(&active).map_err(|source| ObserverError::Read {
        path: active.clone(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: active.clone(),
            source,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(other_id) = name.strip_suffix(".contract.json") else {
            continue;
        };
        if other_id == work_item_id {
            continue;
        }
        if !is_regular_non_symlink(&entry.path())? {
            return Err(ObserverError::State {
                path: entry.path(),
                message: "active Work Item contract must be a regular non-symlink file".into(),
            });
        }
        let other: serde_json::Value = read_json(&entry.path())?;
        let other_scope = other["scope"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let other_intelligence = read_work_item_intelligence(&root, other_id)?;
        let other_boundary = read_contract_boundary(&root, other_id)?;
        let declared_conflict = intelligence.conflicts_with.iter().any(|id| id == other_id);
        let reciprocal_conflict = other_intelligence
            .as_ref()
            .is_some_and(|item| item.conflicts_with.iter().any(|id| id == work_item_id));
        if declared_conflict {
            conflicts.push(other_id.to_string());
            reasons.push(format!("explicit_conflict:{other_id}"));
            continue;
        }
        if reciprocal_conflict {
            conflicts.push(other_id.to_string());
            reasons.push(format!("explicit_conflict:{other_id}"));
            continue;
        }
        match (&target_boundary, &other_boundary) {
            (Some(left), Some(right)) => match concurrency_boundary_relation(left, right) {
                (ScopeRelation::Overlap, Some(detail)) => {
                    conflicts.push(other_id.to_string());
                    reasons.push(format!("concurrency_boundary_overlap:{other_id}:{detail}"));
                }
                (ScopeRelation::Unknown, _) => {
                    reasons.push(format!("concurrency_boundary_unknown:{other_id}"));
                }
                (ScopeRelation::Disjoint, _) | (ScopeRelation::Overlap, None) => {}
            },
            (Some(_), None) | (None, Some(_)) => {
                reasons.push(format!("concurrency_boundary_unknown:{other_id}"));
            }
            (None, None) => match scope_list_relation(&target_scope, &other_scope) {
                ScopeRelation::Overlap => {
                    conflicts.push(other_id.to_string());
                    reasons.push(format!("scope_overlap:{other_id}"));
                }
                ScopeRelation::Unknown => {
                    reasons.push(format!("scope_overlap_unknown:{other_id}"));
                }
                ScopeRelation::Disjoint => {}
            },
        }
        if (target_boundary.is_some() || other_boundary.is_some())
            && other_intelligence
                .as_ref()
                .is_none_or(|item| !item.parallelizable)
        {
            reasons.push(format!("parallel_compatibility_not_declared:{other_id}"));
        }
    }
    conflicts.sort();
    conflicts.dedup();
    if !unknowns.is_empty() {
        reasons.extend(unknowns);
    }
    let compatible = intelligence.parallelizable
        && dependencies_satisfied
        && conflicts.is_empty()
        && reasons.iter().all(|reason| {
            !reason.starts_with("dependency_not_observed")
                && !reason.starts_with("scope_overlap_unknown")
                && !reason.starts_with("concurrency_boundary_unknown")
                && !reason.starts_with("parallel_compatibility_not_declared")
        });
    Ok(WorkItemCompatibility {
        repository_id: repository_id(&root).to_string(),
        work_item_id: work_item_id.into(),
        compatible,
        dependencies_satisfied,
        conflicts,
        reasons,
    })
}

fn knowledge_source_digest(archive: &Path) -> Result<String, ObserverError> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(archive).map_err(|source| ObserverError::Read {
        path: archive.into(),
        source,
    })? {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: archive.into(),
            source,
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let relative = path
            .strip_prefix(archive)
            .map_err(|error| ObserverError::State {
                path: path.clone(),
                message: error.to_string(),
            })?
            .to_string_lossy()
            .replace('\\', "/");
        let metadata = fs::metadata(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        if metadata.len() > MAX_RECEIPT_INDEX_BYTES {
            return Err(ObserverError::State {
                path,
                message: "knowledge source file exceeds bounded cache input".into(),
            });
        }
        let bytes = fs::read(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        entries.push((relative, Digest::sha256_bytes(&bytes).to_string()));
    }
    entries.sort();
    Ok(
        Digest::sha256_bytes(&serde_json::to_vec(&entries).map_err(|error| {
            ObserverError::State {
                path: archive.into(),
                message: error.to_string(),
            }
        })?)
        .to_string(),
    )
}

fn validate_work_item_id(id: &str) -> Result<(), ObserverError> {
    if id.is_empty()
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(ObserverError::State {
            path: PathBuf::from(id),
            message: "invalid work item id".into(),
        });
    }
    Ok(())
}

fn read_json(path: &Path) -> Result<serde_json::Value, ObserverError> {
    let bytes = fs::read(path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })?;
    reject_duplicate_json_keys(&bytes).map_err(|message| ObserverError::State {
        path: path.into(),
        message: format!("invalid JSON: {message}"),
    })?;
    serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
        path: path.into(),
        message: error.to_string(),
    })
}

fn attached_profile_digest(
    profile: &AttachedProfile,
    path: &Path,
) -> Result<Digest, ObserverError> {
    let computed = cockpit_protocol::digest_json(&cockpit_protocol::ProjectProfile {
        profile_version: profile.profile_version,
        repository_id: profile.repository_id.clone(),
        tests: profile.tests.clone(),
        build_systems: profile.build_systems.clone(),
    })
    .map_err(|error| ObserverError::State {
        path: path.into(),
        message: error.to_string(),
    })?;
    if let Some(stored) = &profile.profile_digest
        && stored != &computed
    {
        return Err(ObserverError::State {
            path: path.into(),
            message: "attached profile digest does not match profile fields".into(),
        });
    }
    Ok(computed)
}

fn atomic_json(path: &Path, value: &serde_json::Value) -> Result<(), ObserverError> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| ObserverError::State {
        path: path.into(),
        message: error.to_string(),
    })?;
    atomic_write(path, &bytes)
}

/// Serialize lifecycle transitions for one Work Item across threads and
/// processes. The lock file is retained under the ignored `.ai/locks`
/// runtime directory so a later caller cannot replace the inode while an
/// older waiter still holds it. The operating-system lock is released when
/// the handle is dropped, including after a process crash.
fn acquire_lifecycle_lock(root: &Path, work_item_id: &str) -> Result<fs::File, ObserverError> {
    let root_dir = Dir::open_ambient_dir(root, cap_std::ambient_authority()).map_err(|source| {
        ObserverError::Read {
            path: root.to_path_buf(),
            source,
        }
    })?;
    let ai_path = root.join(".ai");
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &ai_path)?;
    let locks_path = ai_path.join("locks");
    let locks = create_and_open_cap_directory(&ai, "locks", &locks_path)?;
    let lock_name = format!("{work_item_id}.lifecycle.lock");
    let lock_path = locks_path.join(&lock_name);
    let lock = open_or_create_cap_nofollow(&locks, &lock_name, &lock_path)?;
    lock.lock().map_err(|source| ObserverError::Read {
        path: lock_path,
        source,
    })?;
    Ok(lock)
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ObserverError> {
    let sequence = NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed);
    let temporary = path.with_extension(format!("tmp-{}-{sequence}", std::process::id()));
    fs::write(&temporary, bytes).map_err(|source| ObserverError::Read {
        path: temporary.clone(),
        source,
    })?;
    fs::rename(&temporary, path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })
}

pub fn observe(
    root: &Path,
    snapshot: &RepositorySnapshot,
) -> Result<RepositoryObservation, ObserverError> {
    let canonical_root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let canonical_snapshot_root =
        fs::canonicalize(&snapshot.root).map_err(|source| ObserverError::Read {
            path: snapshot.root.clone(),
            source,
        })?;
    if canonical_root != canonical_snapshot_root {
        return Err(ObserverError::SnapshotRootMismatch);
    }
    let mut files = Vec::new();
    collect_files(&canonical_root, &canonical_root, &mut files)?;
    // Sort once after the walk.  Sorting at every recursive directory level
    // repeatedly reorders the same prefix and becomes quadratic for large
    // repositories while producing the same deterministic order.
    files.sort_unstable();
    let mut languages = Vec::new();
    let mut build_systems = Vec::new();
    let mut test_roots = Vec::new();
    let mut quality_commands = Vec::new();
    let mut ci_surfaces = Vec::new();
    let mut critical_domains = Vec::new();
    for relative in &files {
        match relative
            .extension()
            .and_then(|extension| extension.to_str())
        {
            Some("rs") => languages.push(LanguageSignal::Rust),
            Some("py") => languages.push(LanguageSignal::Python),
            Some("js") | Some("jsx") => languages.push(LanguageSignal::JavaScript),
            Some("ts") | Some("tsx") => languages.push(LanguageSignal::TypeScript),
            Some("go") => languages.push(LanguageSignal::Go),
            _ => {}
        }
        let path = relative.to_string_lossy();
        let path_string = path.to_string();
        if path.starts_with("tests/") || path == "tests" {
            test_roots.push("tests/**".into());
        }
        if path.starts_with(".github/workflows/") {
            ci_surfaces.push(path_string.clone());
        }
        for (needle, domain) in [
            ("security", "security"),
            ("payment", "payment"),
            ("auth", "identity"),
            ("production", "production"),
            ("release", "release"),
        ] {
            if path.contains(needle) {
                critical_domains.push(domain.into());
            }
        }
        if path == "Cargo.toml" {
            build_systems.push(BuildSystem::Cargo);
            quality_commands.push(QualityCommand {
                program: "cargo".into(),
                args: vec!["test".into(), "--workspace".into()],
                state: "detected".into(),
            });
        } else if path == "package.json" {
            build_systems.push(BuildSystem::Npm);
        } else if path == "pyproject.toml" {
            build_systems.push(BuildSystem::Poetry);
        } else if path == "go.mod" {
            build_systems.push(BuildSystem::Go);
        } else if path == "Makefile" {
            build_systems.push(BuildSystem::Make);
        }
    }
    languages.sort_by_key(|value| format!("{value:?}"));
    languages.dedup();
    build_systems.sort_by_key(|value| format!("{value:?}"));
    build_systems.dedup();
    test_roots.sort();
    test_roots.dedup();
    ci_surfaces.sort();
    ci_surfaces.dedup();
    critical_domains.sort();
    critical_domains.dedup();
    let mut hasher = Sha256::new();
    for path in &files {
        hasher.update(path.to_string_lossy().as_bytes());
        hasher.update([0]);
    }
    let snapshot_digest = Digest::sha256_bytes(&hasher.finalize());
    Ok(RepositoryObservation {
        snapshot_digest,
        languages,
        build_systems,
        test_roots,
        quality_commands,
        ci_surfaces,
        critical_domains,
        dependency_fingerprint: snapshot
            .dependency_fingerprint
            .parse()
            .unwrap_or_else(|_| Digest::sha256_bytes(b"invalid-dependency-fingerprint")),
        files_read: files.len() + snapshot.files_read,
        cache_hit: false,
    })
}

pub fn observe_cached(
    root: &Path,
    snapshot: &RepositorySnapshot,
) -> Result<RepositoryObservation, ObserverError> {
    let canonical_root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    if !canonical_root.join(".ai/cockpit.toml").is_file() {
        return observe(&canonical_root, snapshot);
    }
    let cache_path = canonical_root.join(".ai/decisions/observer-snapshot.json");
    if let Ok(cache) = read_json(&cache_path) {
        let matches = cache["treeDigest"] == snapshot.tree_digest
            && cache["diffDigest"] == snapshot.diff_digest
            && cache["dependencyFingerprint"] == snapshot.dependency_fingerprint;
        if matches
            && let Ok(mut observation) =
                serde_json::from_value::<RepositoryObservation>(cache["observation"].clone())
        {
            observation.cache_hit = true;
            return Ok(observation);
        }
    }
    let observation = observe(&canonical_root, snapshot)?;
    let cache = serde_json::json!({
        "treeDigest": snapshot.tree_digest,
        "diffDigest": snapshot.diff_digest,
        "dependencyFingerprint": snapshot.dependency_fingerprint,
        "observation": observation,
    });
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|source| ObserverError::Read {
            path: parent.into(),
            source,
        })?;
    }
    atomic_json(&cache_path, &cache)?;
    Ok(observation)
}

pub fn classify_evolution(
    _profile: &cockpit_protocol::ProjectProfile,
    observation: &RepositoryObservation,
    snapshot: &RepositorySnapshot,
) -> Vec<EvolutionEvent> {
    snapshot
        .changed_paths
        .iter()
        .map(|path| {
            let normalized = path.replace('\\', "/");
            let is_governance = normalized.starts_with(".github/")
                || normalized.contains("security")
                || normalized.contains("release")
                || normalized.contains("branch-protection")
                || normalized.contains("production");
            let is_known_test = observation.test_roots.iter().any(|root| {
                root.strip_suffix("/**")
                    .is_some_and(|prefix| normalized.starts_with(&format!("{prefix}/")))
            });
            let is_new_capability = normalized.contains("playwright")
                || normalized.contains("cypress")
                || normalized.contains("nextest")
                || normalized.ends_with("playwright.config.ts")
                || normalized.ends_with("playwright.config.js");
            if is_governance {
                EvolutionEvent {
                    class: EvolutionClass::L3,
                    event_type: "governance_change".into(),
                    path: normalized,
                    action: "needs_human_decision".into(),
                }
            } else if is_new_capability {
                EvolutionEvent {
                    class: EvolutionClass::L2,
                    event_type: "new_test_framework".into(),
                    path: normalized,
                    action: "needs_confirmation".into(),
                }
            } else if is_known_test {
                EvolutionEvent {
                    class: EvolutionClass::L1,
                    event_type: "new_test".into(),
                    path: normalized,
                    action: "auto_absorb".into(),
                }
            } else {
                EvolutionEvent {
                    class: EvolutionClass::L0,
                    event_type: "content_change".into(),
                    path: normalized,
                    action: "auto_absorb".into(),
                }
            }
        })
        .collect()
}

pub fn profile_update_proposal(
    profile: &cockpit_protocol::ProjectProfile,
    events: &[EvolutionEvent],
) -> Option<ProfileUpdateProposal> {
    let candidate = events
        .iter()
        .find(|event| matches!(&event.class, EvolutionClass::L2 | EvolutionClass::L3))?;
    Some(ProfileUpdateProposal {
        from_profile_version: profile.profile_version,
        candidate: candidate.path.clone(),
        reason: candidate.event_type.clone(),
        requires_human_confirmation: true,
    })
}

pub fn confirm_profile_update(
    root: &Path,
    program: &str,
    args: &[String],
) -> Result<AttachedProfile, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let project_path = root.join(".ai/project.json");
    let current: AttachedProfile = read_json(&project_path).and_then(|value| {
        serde_json::from_value(value).map_err(|error| ObserverError::State {
            path: project_path.clone(),
            message: error.to_string(),
        })
    })?;
    let mut tests = current.tests.clone();
    let candidate = QualityCommand {
        program: program.into(),
        args: args.to_vec(),
        state: "verified".into(),
    };
    if !tests.contains(&candidate) {
        tests.push(candidate);
    }
    tests.sort_by(|left, right| {
        (left.program.as_str(), &left.args).cmp(&(right.program.as_str(), &right.args))
    });
    let profile_version = current.profile_version + 1;
    let profile = cockpit_protocol::ProjectProfile {
        profile_version,
        repository_id: current.repository_id.clone(),
        tests: tests.clone(),
        build_systems: current.build_systems.clone(),
    };
    let profile_digest =
        cockpit_protocol::digest_json(&profile).map_err(|error| ObserverError::State {
            path: project_path.clone(),
            message: error.to_string(),
        })?;
    let updated = AttachedProfile {
        profile_version,
        repository_id: current.repository_id,
        repository_schema_version: current.repository_schema_version,
        state: "calibrated".into(),
        profile_digest: Some(profile_digest.clone()),
        tests,
        build_systems: current.build_systems,
    };
    let value = serde_json::to_value(&updated).map_err(|error| ObserverError::State {
        path: project_path.clone(),
        message: error.to_string(),
    })?;
    atomic_json(&project_path, &value)?;
    let decision = serde_json::json!({
        "kind": "project_profile_confirmation",
        "profileVersion": profile_version,
        "profileDigest": profile_digest,
        "candidate": {"program": program, "args": args},
        "state": "confirmed",
        "createdAt": now(),
    });
    atomic_json(
        &root
            .join(".ai/decisions")
            .join(format!("profile-v{profile_version}.json")),
        &decision,
    )?;
    Ok(updated)
}

fn collect_files(
    root: &Path,
    current: &Path,
    output: &mut Vec<PathBuf>,
) -> Result<(), ObserverError> {
    let entries = fs::read_dir(current).map_err(|source| ObserverError::Read {
        path: current.into(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: current.into(),
            source,
        })?;
        let path = entry.path();
        if path
            .file_name()
            .is_some_and(|name| name == ".git" || name == "target")
        {
            continue;
        }
        if path.is_dir() {
            collect_files(root, &path, output)?;
        } else if path.is_file() {
            output.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
        }
    }
    Ok(())
}

#[cfg(test)]
mod environment_identity_tests {
    use super::execution_environment_digest_from_values;
    use std::{ffi::OsString, path::Path};

    fn digest(values: &[(&str, &str)]) -> String {
        execution_environment_digest_from_values(
            values
                .iter()
                .map(|(name, value)| (OsString::from(name), OsString::from(value))),
            Path::new("/tmp/ai-cockpit-environment-test"),
        )
        .expect("environment digest")
        .to_string()
    }

    #[test]
    fn mise_session_metadata_does_not_invalidate_exact_reuse() {
        let first = digest(&[
            ("PATH", "/usr/bin"),
            ("PWD", "/repo"),
            ("_", "/usr/bin/time"),
            ("OLDPWD", "/repo-a"),
            ("SHLVL", "2"),
            ("__MISE_SESSION", "session-a"),
            ("__MISE_ORIG_PATH", "/usr/bin"),
            ("CODEX_SESSION_ID", "turn-a"),
            ("CODEX_THREAD_ID", "thread-a"),
        ]);
        let second = digest(&[
            ("PATH", "/usr/bin"),
            ("PWD", "/repo"),
            ("_", "/usr/bin/env"),
            ("OLDPWD", "/repo-b"),
            ("SHLVL", "3"),
            ("__MISE_SESSION", "session-b"),
            ("__MISE_ORIG_PATH", "/opt/bin"),
            ("CODEX_SESSION_ID", "turn-b"),
            ("CODEX_THREAD_ID", "thread-b"),
        ]);
        assert_eq!(first, second);
    }

    #[test]
    fn command_environment_changes_still_invalidate_exact_reuse() {
        let first = digest(&[("PATH", "/usr/bin"), ("PWD", "/repo")]);
        let second = digest(&[("PATH", "/opt/toolchain"), ("PWD", "/repo")]);
        assert_ne!(first, second);
    }
}
