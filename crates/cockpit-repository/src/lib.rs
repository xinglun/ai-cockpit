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
    GovernanceCost, GovernancePolicy, GovernancePolicyDocument, HumanBenefitReport, HumanDecision,
    ImplementationApproach, OutcomeClaim, OutcomeReportBindings, OutcomeReportSections,
    OutcomeState, OutcomeV2, PARALLEL_SLOT_LEASE_SCHEMA_VERSION, ParallelSlotLease,
    PerformanceDiagnosis, PolicyLayer, QualityCommand, RecoveryDecisionReceipt, RepositoryConfig,
    ResourceFinalizationContext, ResourceFinalizationDisposition, ResourceFinalizationReceipt,
    ResourceFinalizationTransitionReceipt, RuntimeContext, SchemaMigrationStep, TaskOutcomeEvent,
    TaskOutcomeReport, TruthState, VerificationStage, VerificationTier, WorkItemCompatibility,
    WorkItemEvidenceFreshness, WorkItemIntelligence, WorkItemStatusIndex, WorkItemStatusIndexEntry,
    WorkItemStatusSnapshot, default_repository_schema_version, merge_policy_layers,
    repository_schema_migration_chain, validate_evidence_retention, validate_protocol_version,
    validate_resource_finalization_receipt_for, validate_resource_finalization_replay,
    validate_resource_finalization_transition,
};
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer as _, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::BTreeMap;
#[cfg(unix)]
use std::collections::BTreeSet;
#[cfg(unix)]
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;

mod governance_controls;
mod outcome_render;
mod project_governance;

pub use governance_controls::*;
pub use outcome_render::render_human_outcome;
pub use project_governance::*;

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
pub struct VerificationContextInput {
    pub program: String,
    pub args: Vec<String>,
    pub command_digest: String,
    pub scope: Vec<String>,
    pub stage: String,
    pub runner: String,
    pub runtime_digest: String,
    pub base_commit: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerificationReuseAssessment {
    Authorized(Box<VerificationReuseAuthorization>),
    Denied { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationReuseAuthorization {
    pub context: cockpit_evidence::EvidenceContext,
    config_file_digest: String,
    profile_file_digest: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct VerificationIdentityCost {
    files_read: usize,
    files_hashed: usize,
}

struct ResolvedExecutableIdentity {
    path: String,
    digest: String,
    components: Vec<ExecutableComponent>,
    #[cfg(unix)]
    _staging: tempfile::TempDir,
}

struct ExecutableComponent {
    path: String,
    #[cfg(unix)]
    execution_path: String,
    digest: String,
    #[cfg(unix)]
    first_line: Option<Vec<u8>>,
    _file: fs::File,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceiptStoreBinding {
    pub repository_id: String,
    pub profile_digest: String,
    pub node_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReceiptStoreLoad {
    Candidate {
        receipt: Box<cockpit_evidence::ReusableReceipt>,
        files_read: usize,
    },
    Unavailable {
        reason: String,
        files_read: usize,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReceiptStoreWrite {
    pub files_read: usize,
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

/// Request-scoped repository state.  A context captures one immutable Git
/// snapshot and memoizes the derived observation for the lifetime of the
/// request.  Callers that need fresh facts must create a new context instead
/// of mutating or globally replacing this one.
pub struct RepositoryExecutionContext {
    root: PathBuf,
    repository_id: Digest,
    snapshot: RepositorySnapshot,
    observation: OnceLock<RepositoryObservation>,
    observation_guard: Mutex<()>,
}

impl std::fmt::Debug for RepositoryExecutionContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RepositoryExecutionContext")
            .field("root", &self.root)
            .field("repository_id", &self.repository_id)
            .field("snapshot", &self.snapshot)
            .finish_non_exhaustive()
    }
}

impl RepositoryExecutionContext {
    pub fn capture(root: &Path) -> Result<Self, ObserverError> {
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
        let repository_id = repository_id(&root);
        Ok(Self {
            root,
            repository_id,
            snapshot,
            observation: OnceLock::new(),
            observation_guard: Mutex::new(()),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn repository_id(&self) -> &Digest {
        &self.repository_id
    }

    pub fn snapshot(&self) -> &RepositorySnapshot {
        &self.snapshot
    }

    pub fn observe(&self) -> Result<&RepositoryObservation, ObserverError> {
        if let Some(observation) = self.observation.get() {
            return Ok(observation);
        }
        let _guard = self
            .observation_guard
            .lock()
            .map_err(|_| ObserverError::State {
                path: self.root.join(".ai"),
                message: "repository observation mutex was poisoned".into(),
            })?;
        if let Some(observation) = self.observation.get() {
            return Ok(observation);
        }
        let observation = observe_cached(&self.root, &self.snapshot)?;
        let _ = self.observation.set(observation);
        self.observation.get().ok_or_else(|| ObserverError::State {
            path: self.root.join(".ai"),
            message: "repository observation was not initialized".into(),
        })
    }
}

/// Explicitly owned process session for repeated requests. It is not a
/// global current-repository slot: every lookup receives an explicit path,
/// and each entry contains an isolated request context. The caller chooses
/// when to refresh a context after a repository mutation.
#[derive(Default)]
pub struct RuntimeSession {
    contexts: Mutex<BTreeMap<PathBuf, Arc<RepositoryExecutionContext>>>,
}

impl std::fmt::Debug for RuntimeSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeSession")
            .field(
                "active_repositories",
                &self.active_repositories().unwrap_or_default(),
            )
            .finish()
    }
}

impl RuntimeSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&self, root: &Path) -> Result<Arc<RepositoryExecutionContext>, ObserverError> {
        let canonical = fs::canonicalize(root).map_err(|source| ObserverError::Read {
            path: root.into(),
            source,
        })?;
        let mut contexts = self.contexts.lock().map_err(|_| ObserverError::State {
            path: canonical.clone(),
            message: "runtime session mutex was poisoned".into(),
        })?;
        if let Some(context) = contexts.get(&canonical) {
            return Ok(Arc::clone(context));
        }
        let context = Arc::new(RepositoryExecutionContext::capture(&canonical)?);
        contexts.insert(canonical, Arc::clone(&context));
        Ok(context)
    }

    pub fn refresh(&self, root: &Path) -> Result<Arc<RepositoryExecutionContext>, ObserverError> {
        let canonical = fs::canonicalize(root).map_err(|source| ObserverError::Read {
            path: root.into(),
            source,
        })?;
        let context = Arc::new(RepositoryExecutionContext::capture(&canonical)?);
        let mut contexts = self.contexts.lock().map_err(|_| ObserverError::State {
            path: canonical.clone(),
            message: "runtime session mutex was poisoned".into(),
        })?;
        contexts.insert(canonical, Arc::clone(&context));
        Ok(context)
    }

    pub fn unbind(&self, root: &Path) -> Result<bool, ObserverError> {
        let canonical = fs::canonicalize(root).map_err(|source| ObserverError::Read {
            path: root.into(),
            source,
        })?;
        let mut contexts = self.contexts.lock().map_err(|_| ObserverError::State {
            path: canonical.clone(),
            message: "runtime session mutex was poisoned".into(),
        })?;
        Ok(contexts.remove(&canonical).is_some())
    }

    pub fn active_repositories(&self) -> Result<Vec<PathBuf>, ObserverError> {
        self.contexts
            .lock()
            .map(|contexts| contexts.keys().cloned().collect())
            .map_err(|_| ObserverError::State {
                path: PathBuf::from(".ai"),
                message: "runtime session mutex was poisoned".into(),
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReceiptStoreIndex {
    schema_version: u32,
    repository_id: String,
    profile_digest: String,
    receipts: BTreeMap<String, String>,
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
    pub readiness: RepositoryReadiness,
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

pub fn assess_verification_reuse(
    root: &Path,
    snapshot: &RepositorySnapshot,
    input: &VerificationContextInput,
) -> Result<VerificationReuseAssessment, ObserverError> {
    assess_verification_reuse_measured(
        root,
        snapshot,
        input,
        None,
        &mut VerificationIdentityCost::default(),
    )
}

fn assess_verification_reuse_measured(
    root: &Path,
    snapshot: &RepositorySnapshot,
    input: &VerificationContextInput,
    pre_resolved: Option<&ResolvedExecutableIdentity>,
    cost: &mut VerificationIdentityCost,
) -> Result<VerificationReuseAssessment, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let snapshot_root = fs::canonicalize(&snapshot.root).map_err(|source| ObserverError::Read {
        path: snapshot.root.clone(),
        source,
    })?;
    if root != snapshot_root {
        return Err(ObserverError::SnapshotRootMismatch);
    }

    let Some(head) = snapshot
        .head
        .as_ref()
        .filter(|head| valid_git_object_id(head))
    else {
        return Ok(denied_reuse("source_revision_unknown"));
    };
    let Ok(stage) = VerificationStage::parse(&input.stage) else {
        return Ok(denied_reuse("verification_context_invalid"));
    };
    if input.program.is_empty()
        || input.scope.is_empty()
        || !matches!(input.runner.as_str(), "local" | "hosted")
    {
        return Ok(denied_reuse("verification_context_invalid"));
    }
    let base_commit = if !stage.requires_base_revision() {
        head.clone()
    } else {
        let Some(base) = input
            .base_commit
            .as_ref()
            .filter(|base| valid_git_object_id(base))
        else {
            return Ok(denied_reuse("base_revision_unknown"));
        };
        base.clone()
    };
    let owned_executable_identity;
    let executable_identity = if let Some(identity) = pre_resolved {
        identity
    } else {
        let Some(identity) = resolved_executable_identity(&root, &input.program) else {
            return Ok(denied_reuse("toolchain_identity_unknown"));
        };
        owned_executable_identity = identity;
        &owned_executable_identity
    };
    if pre_resolved.is_none() {
        cost.files_read = cost
            .files_read
            .saturating_add(executable_identity.components.len());
        cost.files_hashed = cost
            .files_hashed
            .saturating_add(executable_identity.components.len());
    }
    let execution_environment = execution_environment_digest(&snapshot.root)?;

    let ai = root.join(".ai");
    let config_path = ai.join("cockpit.toml");
    let profile_path = ai.join("project.json");
    let (config_bytes, profile_bytes) = read_verification_identity_files(&root)?;
    cost.files_read = cost.files_read.saturating_add(2);
    cost.files_hashed = cost.files_hashed.saturating_add(2);
    let config_text = std::str::from_utf8(&config_bytes).map_err(|error| ObserverError::State {
        path: config_path.clone(),
        message: error.to_string(),
    })?;
    let config: RepositoryConfig =
        toml::from_str(config_text).map_err(|error| ObserverError::State {
            path: config_path.clone(),
            message: error.to_string(),
        })?;
    validate_protocol_version(config.protocol_version).map_err(|error| ObserverError::State {
        path: config_path,
        message: error.to_string(),
    })?;
    let expected_repository_id = repository_id(&root).to_string();
    if config.repository_id != expected_repository_id {
        return Ok(denied_reuse("repository_identity_mismatch"));
    }

    let profile: AttachedProfile =
        match serde_json::from_slice(&profile_bytes).map_err(|error| ObserverError::State {
            path: profile_path.clone(),
            message: error.to_string(),
        }) {
            Ok(profile) => profile,
            Err(_) => return Ok(denied_reuse("profile_untrusted")),
        };
    if profile.repository_id != expected_repository_id
        || profile.repository_id != config.repository_id
    {
        return Ok(denied_reuse("repository_identity_mismatch"));
    }
    let Some(stored_profile_digest) = profile.profile_digest.as_ref() else {
        return Ok(denied_reuse("profile_digest_missing"));
    };
    let computed_profile_digest = digest_value(
        &cockpit_protocol::ProjectProfile {
            profile_version: profile.profile_version,
            repository_id: profile.repository_id.clone(),
            tests: profile.tests.clone(),
            build_systems: profile.build_systems.clone(),
        },
        &profile_path,
    )?;
    if stored_profile_digest.to_string() != computed_profile_digest {
        return Ok(denied_reuse("profile_digest_mismatch"));
    }
    if profile.state != "calibrated" {
        return Ok(denied_reuse("profile_not_calibrated"));
    }
    if !profile.tests.iter().any(|command| {
        command.program == input.program
            && command.args == input.args
            && command.state == "verified"
    }) {
        return Ok(denied_reuse("command_not_profile_verified"));
    }

    let mut changed_paths: Vec<String> = snapshot
        .changed_paths
        .iter()
        .map(|path| path.replace('\\', "/"))
        .filter(|path| path != ".ai" && !path.starts_with(".ai/"))
        .collect();
    changed_paths.sort();
    changed_paths.dedup();
    let mut scope = input.scope.clone();
    scope.sort();
    scope.dedup();

    let context = cockpit_evidence::EvidenceContext {
        content_digest: digest_value(
            &(snapshot.tree_digest.as_str(), snapshot.diff_digest.as_str()),
            &snapshot.root,
        )?,
        diff: cockpit_evidence::DiffIdentity {
            base_commit,
            head_commit: head.clone(),
            changed_paths_digest: digest_value(&changed_paths, &snapshot.root)?,
        },
        environment_digest: digest_value(
            &(
                input.runtime_digest.as_str(),
                std::env::consts::OS,
                std::env::consts::ARCH,
                input.runner.as_str(),
                execution_environment.as_str(),
            ),
            &snapshot.root,
        )?,
        command_digest: input.command_digest.clone(),
        scope_digest: digest_value(&scope, &snapshot.root)?,
        governance_digest: digest_value(
            &(config.protocol_version, config.repository_id.as_str()),
            &snapshot.root,
        )?,
        toolchain_digest: digest_value(
            &(
                input.program.as_str(),
                &input.args,
                executable_identity.path.as_str(),
                executable_identity.digest.as_str(),
                snapshot.dependency_fingerprint.as_str(),
            ),
            &snapshot.root,
        )?,
        policy_digest: digest_value(
            &(
                "verification-reuse-v1",
                input.stage.as_str(),
                profile.state.as_str(),
            ),
            &snapshot.root,
        )?,
        profile_digest: stored_profile_digest.to_string(),
        stage: input.stage.clone(),
        runner: input.runner.clone(),
    };
    if context.validate().is_err() {
        return Ok(denied_reuse("verification_context_invalid"));
    }
    Ok(VerificationReuseAssessment::Authorized(Box::new(
        VerificationReuseAuthorization {
            context,
            config_file_digest: digest_raw_bytes(&config_bytes),
            profile_file_digest: digest_raw_bytes(&profile_bytes),
        },
    )))
}

fn refresh_verification_context(
    root: &Path,
    snapshot: &RepositorySnapshot,
    input: &VerificationContextInput,
    executable_identity: Option<&ResolvedExecutableIdentity>,
    authorization: &VerificationReuseAuthorization,
    cost: &mut VerificationIdentityCost,
) -> Result<Option<cockpit_evidence::EvidenceContext>, ObserverError> {
    let snapshot_root = fs::canonicalize(&snapshot.root).map_err(|source| ObserverError::Read {
        path: snapshot.root.clone(),
        source,
    })?;
    if root != snapshot_root {
        return Err(ObserverError::SnapshotRootMismatch);
    }
    let Some(head) = snapshot
        .head
        .as_ref()
        .filter(|head| valid_git_object_id(head))
    else {
        return Ok(None);
    };
    let stage = VerificationStage::parse(&input.stage).ok();
    let base_commit = if stage.is_some_and(|value| !value.requires_base_revision()) {
        head.clone()
    } else {
        let Some(base) = input
            .base_commit
            .as_ref()
            .filter(|base| valid_git_object_id(base))
        else {
            return Ok(None);
        };
        base.clone()
    };
    let Some(executable_identity) = executable_identity else {
        return Ok(None);
    };
    let execution_environment = execution_environment_digest(&snapshot.root)?;
    let mut changed_paths = snapshot
        .changed_paths
        .iter()
        .map(|path| path.replace('\\', "/"))
        .filter(|path| path != ".ai" && !path.starts_with(".ai/"))
        .collect::<Vec<_>>();
    changed_paths.sort();
    changed_paths.dedup();
    let mut scope = input.scope.clone();
    scope.sort();
    scope.dedup();
    let Ok((config_bytes, profile_bytes)) = read_verification_identity_files(root) else {
        return Ok(None);
    };
    cost.files_read = cost.files_read.saturating_add(2);
    cost.files_hashed = cost.files_hashed.saturating_add(2);
    if digest_raw_bytes(&config_bytes) != authorization.config_file_digest
        || digest_raw_bytes(&profile_bytes) != authorization.profile_file_digest
    {
        return Ok(None);
    }
    let mut context = authorization.context.clone();
    context.content_digest = digest_value(
        &(snapshot.tree_digest.as_str(), snapshot.diff_digest.as_str()),
        &snapshot.root,
    )?;
    context.diff = cockpit_evidence::DiffIdentity {
        base_commit,
        head_commit: head.clone(),
        changed_paths_digest: digest_value(&changed_paths, &snapshot.root)?,
    };
    context.environment_digest = digest_value(
        &(
            input.runtime_digest.as_str(),
            std::env::consts::OS,
            std::env::consts::ARCH,
            input.runner.as_str(),
            execution_environment.as_str(),
        ),
        &snapshot.root,
    )?;
    context.command_digest = input.command_digest.clone();
    context.scope_digest = digest_value(&scope, &snapshot.root)?;
    context.toolchain_digest = digest_value(
        &(
            input.program.as_str(),
            &input.args,
            executable_identity.path.as_str(),
            executable_identity.digest.as_str(),
            snapshot.dependency_fingerprint.as_str(),
        ),
        &snapshot.root,
    )?;
    context.stage = input.stage.clone();
    context.runner = input.runner.clone();
    Ok(context.validate().is_ok().then_some(context))
}

fn denied_reuse(reason: &str) -> VerificationReuseAssessment {
    VerificationReuseAssessment::Denied {
        reason: reason.into(),
    }
}

fn valid_git_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn digest_value<T: Serialize>(value: &T, path: &Path) -> Result<String, ObserverError> {
    cockpit_protocol::digest_json(value)
        .map(|digest| digest.to_string())
        .map_err(|error| ObserverError::State {
            path: path.to_path_buf(),
            message: error.to_string(),
        })
}

fn digest_raw_bytes(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn read_verification_identity_files(root: &Path) -> Result<(Vec<u8>, Vec<u8>), ObserverError> {
    let root_dir = Dir::open_ambient_dir(root, cap_std::ambient_authority()).map_err(|source| {
        ObserverError::Read {
            path: root.to_path_buf(),
            source,
        }
    })?;
    let ai_path = root.join(".ai");
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &ai_path)?;
    let config_path = ai_path.join("cockpit.toml");
    let profile_path = ai_path.join("project.json");
    Ok((
        read_cap_file_nofollow_bounded(
            &ai,
            "cockpit.toml",
            &config_path,
            MAX_VERIFICATION_IDENTITY_FILE_BYTES,
        )?,
        read_cap_file_nofollow_bounded(
            &ai,
            "project.json",
            &profile_path,
            MAX_VERIFICATION_IDENTITY_FILE_BYTES,
        )?,
    ))
}

fn resolved_executable_identity(root: &Path, program: &str) -> Option<ResolvedExecutableIdentity> {
    let executable = resolve_executable(root, program)?;
    #[cfg(windows)]
    if executable
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("bat") || extension.eq_ignore_ascii_case("cmd")
        })
    {
        // Command delegates batch files through cmd.exe, whose identity (and any
        // further runtime selected by the batch file) is not represented here.
        return None;
    }
    #[cfg(unix)]
    let staging = tempfile::Builder::new()
        .prefix("ai-cockpit-executable-")
        .tempdir()
        .ok()?;
    #[cfg(unix)]
    let staging_path = Some(staging.path());
    #[cfg(not(unix))]
    let staging_path = None;
    let mut components = Vec::new();
    collect_executable_identities(root, &executable, &mut components, staging_path, 0)?;
    #[cfg(unix)]
    if !supports_pinned_execution(&components) {
        return None;
    }
    let identities = components
        .iter()
        .map(|component| (&component.path, &component.digest))
        .collect::<Vec<_>>();
    let identity_bytes = serde_json::to_vec(&identities).ok()?;
    Some(ResolvedExecutableIdentity {
        path: executable.to_string_lossy().into_owned(),
        digest: digest_raw_bytes(&identity_bytes),
        components,
        #[cfg(unix)]
        _staging: staging,
    })
}

#[cfg(unix)]
fn supports_pinned_execution(components: &[ExecutableComponent]) -> bool {
    let Some(primary) = components.first() else {
        return false;
    };
    let Some(first_line) = primary.first_line.as_deref() else {
        return true;
    };
    let Some(shebang) = first_line.strip_prefix(b"#!") else {
        return true;
    };
    let Ok(shebang) = std::str::from_utf8(shebang) else {
        return false;
    };
    let words = shebang.split_whitespace().collect::<Vec<_>>();
    let Some(interpreter) = components.get(1) else {
        return false;
    };
    let is_env = Path::new(&interpreter.path)
        .file_name()
        .is_some_and(|name| name == "env" || name == "env.exe");
    let effective_index = if is_env { 2 } else { 1 };
    !components
        .get(effective_index)
        .and_then(|component| component.first_line.as_deref())
        .is_some_and(|line| line.starts_with(b"#!"))
        && (!is_env || words.len() >= 2)
}

impl ResolvedExecutableIdentity {
    #[cfg(target_os = "macos")]
    fn execution_environment(&self) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
        let mut library_paths = self
            .components
            .iter()
            .filter_map(|component| Path::new(&component.path).parent())
            .flat_map(|parent| {
                let prefix = parent.parent().unwrap_or(parent);
                [
                    parent.join("lib"),
                    prefix.join("lib"),
                    prefix.join("Frameworks"),
                ]
            })
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        if let Some(existing) = std::env::var_os("DYLD_LIBRARY_PATH") {
            library_paths.extend(std::env::split_paths(&existing));
        }
        library_paths.sort();
        library_paths.dedup();
        std::env::join_paths(library_paths)
            .ok()
            .map(|value| vec![("DYLD_LIBRARY_PATH".into(), value)])
            .unwrap_or_default()
    }

    #[cfg(not(target_os = "macos"))]
    fn execution_environment(&self) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
        Vec::new()
    }

    fn execution(&self, original_args: &[String]) -> Option<(String, Vec<String>)> {
        #[cfg(windows)]
        {
            return Some((self.path.clone(), original_args.to_vec()));
        }
        #[cfg(unix)]
        {
            let primary = self.components.first()?;
            let primary_path = primary.execution_path.clone();
            let Some(first_line) = primary.first_line.as_deref() else {
                return Some((primary_path, original_args.to_vec()));
            };
            let Some(shebang) = first_line.strip_prefix(b"#!") else {
                return Some((primary_path, original_args.to_vec()));
            };
            let shebang = std::str::from_utf8(shebang).ok()?.trim();
            let words = shebang.split_whitespace().collect::<Vec<_>>();
            let interpreter = self.components.get(1)?;
            let is_env = Path::new(&interpreter.path)
                .file_name()
                .is_some_and(|name| name == "env" || name == "env.exe");
            let executable_index = if is_env { 2 } else { 1 };
            self.components.get(executable_index)?;
            let mut args = Vec::new();
            if !is_env && words.len() > 1 {
                args.push(words[1..].join(" "));
            }
            args.push(primary_path);
            args.extend_from_slice(original_args);
            Some((
                self.components[executable_index].execution_path.clone(),
                args,
            ))
        }
    }
}

fn build_repository_verification_command(
    root: &Path,
    request: &RepositoryVerificationRequest,
    execution_identity: Option<&ResolvedExecutableIdentity>,
    policy: cockpit_verification::VerificationReusePolicy,
) -> cockpit_verification::VerificationCommand {
    let (execution_program, execution_args) = execution_identity
        .and_then(|identity| identity.execution(&request.args))
        .unwrap_or_else(|| (request.program.clone(), request.args.clone()));
    let command = if execution_identity.is_some() {
        cockpit_verification::VerificationCommand::new_pinned(
            &request.node_id,
            &execution_program,
            execution_args,
            &request.program,
            request.args.clone(),
            policy,
        )
    } else {
        cockpit_verification::VerificationCommand::new(
            &request.node_id,
            &execution_program,
            execution_args,
            policy,
        )
    };
    command.with_current_dir(root).with_environment(
        execution_identity.map_or_else(Vec::new, ResolvedExecutableIdentity::execution_environment),
    )
}

fn resolve_executable(root: &Path, program: &str) -> Option<PathBuf> {
    let requested = Path::new(program);
    let executable = if requested.is_absolute() {
        resolve_platform_executable(requested)?
    } else if requested.components().count() > 1 {
        resolve_platform_executable(&root.join(requested))?
    } else {
        std::env::var_os("PATH")
            .into_iter()
            .flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
            .find_map(|directory| resolve_platform_executable(&directory.join(requested)))?
    };
    let executable_name = executable.file_name()?.to_owned();
    let executable_parent = fs::canonicalize(executable.parent()?).ok()?;
    let executable = executable_parent.join(executable_name);
    if !is_executable_file(&executable) {
        return None;
    }
    Some(executable)
}

#[cfg(not(windows))]
fn resolve_platform_executable(requested: &Path) -> Option<PathBuf> {
    is_executable_file(requested).then(|| requested.to_path_buf())
}

#[cfg(windows)]
fn resolve_platform_executable(requested: &Path) -> Option<PathBuf> {
    if requested.extension().is_some() && is_executable_file(requested) {
        return Some(requested.to_path_buf());
    }
    let path_ext = std::env::var_os("PATHEXT").unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".into());
    path_ext
        .to_string_lossy()
        .split(';')
        .filter(|extension| !extension.is_empty())
        .map(|extension| {
            let extension = extension.trim_start_matches('.');
            requested.with_extension(extension)
        })
        .find(|candidate| is_executable_file(candidate))
        .or_else(|| is_executable_file(requested).then(|| requested.to_path_buf()))
}

fn collect_executable_identities(
    root: &Path,
    executable: &Path,
    components: &mut Vec<ExecutableComponent>,
    staging: Option<&Path>,
    depth: usize,
) -> Option<()> {
    if depth > 4 {
        return None;
    }
    let key = executable.to_string_lossy().into_owned();
    if components.iter().any(|component| component.path == key) {
        return None;
    }
    let file = open_pinned_executable(executable)?;
    let (mut file, resolved_execution_path) =
        if staging.is_some_and(|_| should_stage_executable(executable)) {
            stage_pinned_executable(file, executable, staging?, components.len())?
        } else {
            (file, executable.to_string_lossy().into_owned())
        };
    #[cfg(not(unix))]
    let _ = &resolved_execution_path;
    let (digest, first_line) = hash_executable_and_first_line(&mut file)?;
    components.push(ExecutableComponent {
        path: key,
        #[cfg(unix)]
        execution_path: resolved_execution_path,
        digest,
        #[cfg(unix)]
        first_line: first_line.clone(),
        _file: file,
    });
    let Some(first_line) = first_line.as_deref() else {
        return Some(());
    };
    let Some(shebang) = first_line.strip_prefix(b"#!") else {
        return Some(());
    };
    let shebang = std::str::from_utf8(shebang).ok()?.trim();
    if shebang.is_empty() {
        return Some(());
    }
    let words = shebang.split_whitespace().collect::<Vec<_>>();
    let interpreter_program = *words.first()?;
    let interpreter = resolve_executable(root, interpreter_program)?;
    collect_executable_identities(root, &interpreter, components, staging, depth + 1)?;
    if interpreter
        .file_name()
        .is_some_and(|name| name == "env" || name == "env.exe")
        && let Some(command) = simple_env_command(&words[1..])
    {
        let effective = resolve_executable(root, command)?;
        collect_executable_identities(root, &effective, components, staging, depth + 1)?;
    } else if interpreter
        .file_name()
        .is_some_and(|name| name == "env" || name == "env.exe")
    {
        return None;
    }
    Some(())
}

fn stage_pinned_executable(
    mut source: fs::File,
    original: &Path,
    staging: &Path,
    index: usize,
) -> Option<(fs::File, String)> {
    let original_parent = original.parent()?;
    let prefix = original_parent.parent()?;
    let logical_relative = original.strip_prefix(prefix).ok()?;
    let canonical_target = fs::canonicalize(original).ok()?;
    let target_relative = canonical_target.strip_prefix(prefix).ok()?;
    let component_root = staging.join(format!("component-{index}"));
    let staged_target = component_root.join(target_relative);
    fs::create_dir_all(staged_target.parent()?).ok()?;
    let mut destination = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&staged_target)
        .ok()?;
    source.seek(std::io::SeekFrom::Start(0)).ok()?;
    std::io::copy(&mut source, &mut destination).ok()?;
    destination.sync_all().ok()?;
    fs::set_permissions(&staged_target, source.metadata().ok()?.permissions()).ok()?;
    // Some Linux filesystems reject execve while a writable descriptor for
    // the executable is still open (ETXTBSY). Close the copy-on-write handle
    // and retain only a read-only pin for the staged bytes.
    drop(destination);
    let pinned = fs::File::open(&staged_target).ok()?;
    let staged_logical = component_root.join(logical_relative);
    if staged_logical != staged_target {
        fs::create_dir_all(staged_logical.parent()?).ok()?;
        #[cfg(unix)]
        std::os::unix::fs::symlink(
            relative_path(staged_logical.parent()?, &staged_target)?,
            &staged_logical,
        )
        .ok()?;
    }
    #[cfg(unix)]
    mirror_relative_runtime_layout(
        prefix,
        &component_root,
        logical_relative.parent()?,
        target_relative.parent()?,
    )?;
    let execution_path = staged_logical.to_string_lossy().into_owned();
    Some((pinned, execution_path))
}

#[cfg(unix)]
fn mirror_relative_runtime_layout(
    prefix: &Path,
    component_root: &Path,
    logical_parent: &Path,
    target_parent: &Path,
) -> Option<()> {
    use std::os::unix::fs::symlink;

    let mut directories = BTreeSet::from([PathBuf::new()]);
    for parent in [logical_parent, target_parent] {
        let mut relative = PathBuf::new();
        for component in parent.components() {
            relative.push(component.as_os_str());
            directories.insert(relative.clone());
        }
    }
    for relative in directories {
        let original_directory = prefix.join(&relative);
        let staged_directory = component_root.join(&relative);
        for entry in fs::read_dir(original_directory).ok()?.flatten() {
            let destination = staged_directory.join(entry.file_name());
            if fs::symlink_metadata(&destination).is_ok() {
                continue;
            }
            symlink(entry.path(), destination).ok()?;
        }
    }
    Some(())
}

#[cfg(unix)]
fn relative_path(from: &Path, to: &Path) -> Option<PathBuf> {
    let from = from.components().collect::<Vec<_>>();
    let to = to.components().collect::<Vec<_>>();
    let common = from
        .iter()
        .zip(&to)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = PathBuf::new();
    for _ in common..from.len() {
        relative.push(OsStr::new(".."));
    }
    for component in &to[common..] {
        relative.push(component.as_os_str());
    }
    (!relative.as_os_str().is_empty()).then_some(relative)
}

#[cfg(target_os = "macos")]
fn should_stage_executable(executable: &Path) -> bool {
    ![
        Path::new("/System"),
        Path::new("/usr/bin"),
        Path::new("/usr/sbin"),
        Path::new("/bin"),
        Path::new("/sbin"),
    ]
    .iter()
    .any(|protected| executable.starts_with(protected))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn should_stage_executable(_executable: &Path) -> bool {
    true
}

#[cfg(not(unix))]
fn should_stage_executable(_executable: &Path) -> bool {
    false
}

fn simple_env_command<'a>(arguments: &'a [&'a str]) -> Option<&'a str> {
    let command = *arguments.first()?;
    (!command.starts_with('-') && !command.contains('=')).then_some(command)
}

fn hash_executable_and_first_line(file: &mut fs::File) -> Option<(String, Option<Vec<u8>>)> {
    const MAX_SHEBANG_BYTES: usize = 4096;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut first_line = Vec::with_capacity(MAX_SHEBANG_BYTES);
    let mut first_line_complete = false;
    loop {
        let count = file.read(&mut buffer).ok()?;
        if count == 0 {
            break;
        }
        let chunk = &buffer[..count];
        hasher.update(chunk);
        if !first_line_complete && first_line.len() < MAX_SHEBANG_BYTES {
            let remaining = MAX_SHEBANG_BYTES - first_line.len();
            let prefix = &chunk[..chunk.len().min(remaining)];
            if let Some(newline) = prefix.iter().position(|byte| *byte == b'\n') {
                first_line.extend_from_slice(&prefix[..newline]);
                first_line_complete = true;
            } else {
                first_line.extend_from_slice(prefix);
            }
        }
    }
    let first_line = if first_line.starts_with(b"#!") && !first_line_complete {
        return None;
    } else if first_line.is_empty() {
        None
    } else {
        Some(first_line)
    };
    file.seek(std::io::SeekFrom::Start(0)).ok()?;
    Some((
        format!("sha256:{}", hex::encode(hasher.finalize())),
        first_line,
    ))
}

#[cfg(unix)]
fn open_pinned_executable(executable: &Path) -> Option<fs::File> {
    fs::File::open(executable).ok()
}

#[cfg(windows)]
fn open_pinned_executable(executable: &Path) -> Option<fs::File> {
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ,
    };

    let file = fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(executable)
        .ok()?;
    let metadata = file.metadata().ok()?;
    if !metadata.is_file() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return None;
    }
    Some(file)
}

fn is_executable_file(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn execution_environment_digest(path: &Path) -> Result<String, ObserverError> {
    let mut values = std::env::vars_os()
        .map(|(name, value)| {
            (
                name.as_encoded_bytes().to_vec(),
                value.as_encoded_bytes().to_vec(),
            )
        })
        .collect::<Vec<_>>();
    values.sort();
    digest_value(&values, path)
}

pub fn load_reusable_receipt(
    root: &Path,
    binding: &ReceiptStoreBinding,
) -> Result<ReceiptStoreLoad, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    if binding.repository_id != repository_id(&root).to_string() {
        return Ok(unavailable_receipt("repository_identity_mismatch", 0));
    }
    if !valid_sha256_digest(&binding.profile_digest) || !valid_node_id(&binding.node_id) {
        return Ok(unavailable_receipt("store_binding_invalid", 0));
    }

    let root_dir =
        Dir::open_ambient_dir(&root, cap_std::ambient_authority()).map_err(|source| {
            ObserverError::Read {
                path: root.clone(),
                source,
            }
        })?;
    let ai = match open_cap_directory_nofollow(&root_dir, ".ai", &root.join(".ai")) {
        Ok(directory) => directory,
        Err(reason) => return Ok(unavailable_receipt(reason, 0)),
    };
    let evidence = match open_cap_directory_nofollow(&ai, "evidence", &root.join(".ai/evidence")) {
        Ok(directory) => directory,
        Err(reason) => return Ok(unavailable_receipt(reason, 0)),
    };
    let reuse_path = root.join(".ai/evidence/reuse");
    let reuse = match open_cap_directory_nofollow(&evidence, "reuse", &reuse_path) {
        Ok(directory) => directory,
        Err(reason) => return Ok(unavailable_receipt(reason, 0)),
    };
    let lock_path = reuse_path.join("index.lock");
    let lock = match open_cap_existing_nofollow(&reuse, "index.lock", &lock_path) {
        Ok(lock) => lock,
        Err(_) => return Ok(unavailable_receipt("index_invalid", 0)),
    };
    if lock.lock_shared().is_err() {
        return Ok(unavailable_receipt("index_unreadable", 0));
    }
    #[cfg(windows)]
    match read_optional_cap_file_nofollow_bounded(
        &reuse,
        "index.pending",
        &reuse_path.join("index.pending"),
        1024,
    ) {
        Ok(Some(_)) => return Ok(unavailable_receipt("index_commit_uncertain", 0)),
        Ok(None) => {}
        Err(ObserverError::State { message, .. }) if message.contains("symlink") => {
            return Ok(unavailable_receipt("symlink_rejected", 0));
        }
        Err(_) => return Ok(unavailable_receipt("index_unreadable", 0)),
    }
    #[cfg(not(windows))]
    match reuse.symlink_metadata("index.pending") {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            return Ok(unavailable_receipt("index_commit_uncertain", 0));
        }
        Ok(_) => return Ok(unavailable_receipt("index_invalid", 0)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return Ok(unavailable_receipt("index_unreadable", 0)),
    }
    let index_path = reuse_path.join("index.json");
    #[cfg(windows)]
    let index_bytes = match read_optional_cap_file_nofollow_bounded(
        &reuse,
        "index.json",
        &index_path,
        MAX_RECEIPT_INDEX_BYTES,
    ) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(unavailable_receipt("evidence_missing", 0)),
        Err(ObserverError::State { message, .. }) if message.contains("symlink") => {
            return Ok(unavailable_receipt("symlink_rejected", 0));
        }
        Err(_) => return Ok(unavailable_receipt("index_unreadable", 0)),
    };
    #[cfg(not(windows))]
    let index_bytes = {
        let index_metadata = match reuse.symlink_metadata("index.json") {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(unavailable_receipt("evidence_missing", 0));
            }
            Err(_) => return Ok(unavailable_receipt("index_unreadable", 0)),
        };
        if index_metadata.file_type().is_symlink() {
            return Ok(unavailable_receipt("symlink_rejected", 0));
        }
        if !index_metadata.is_file() {
            return Ok(unavailable_receipt("index_invalid", 0));
        }
        match read_cap_file_nofollow_bounded(
            &reuse,
            "index.json",
            &index_path,
            MAX_RECEIPT_INDEX_BYTES,
        ) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(unavailable_receipt("index_unreadable", 0)),
        }
    };
    let index: ReceiptStoreIndex = match serde_json::from_slice(&index_bytes) {
        Ok(index) => index,
        Err(_) => return Ok(unavailable_receipt("index_invalid", 1)),
    };
    if index.schema_version != 1 {
        return Ok(unavailable_receipt("index_invalid", 1));
    }
    if index.repository_id != binding.repository_id {
        return Ok(unavailable_receipt("repository_identity_mismatch", 1));
    }
    if index.profile_digest != binding.profile_digest {
        return Ok(unavailable_receipt("profile_identity_mismatch", 1));
    }
    let Some(receipt_id) = index.receipts.get(&binding.node_id) else {
        return Ok(unavailable_receipt("evidence_missing", 1));
    };
    if !valid_sha256_digest(receipt_id) {
        return Ok(unavailable_receipt("index_invalid", 1));
    }

    let receipts_path = reuse_path.join("receipts");
    let receipts = match open_cap_directory_nofollow(&reuse, "receipts", &receipts_path) {
        Ok(directory) => directory,
        Err(reason) => return Ok(unavailable_receipt(reason, 1)),
    };
    let receipt_name = receipt_file_name(receipt_id);
    let receipt_path = receipts_path.join(&receipt_name);
    #[cfg(windows)]
    let receipt_bytes = match read_optional_cap_file_nofollow_bounded(
        &receipts,
        &receipt_name,
        &receipt_path,
        MAX_REUSABLE_RECEIPT_BYTES,
    ) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(unavailable_receipt("receipt_missing", 1)),
        Err(ObserverError::State { message, .. }) if message.contains("symlink") => {
            return Ok(unavailable_receipt("symlink_rejected", 1));
        }
        Err(_) => return Ok(unavailable_receipt("receipt_unreadable", 1)),
    };
    #[cfg(not(windows))]
    let receipt_bytes = {
        let receipt_metadata = match receipts.symlink_metadata(&receipt_name) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(unavailable_receipt("receipt_missing", 1));
            }
            Err(_) => return Ok(unavailable_receipt("receipt_unreadable", 1)),
        };
        if receipt_metadata.file_type().is_symlink() {
            return Ok(unavailable_receipt("symlink_rejected", 1));
        }
        if !receipt_metadata.is_file() {
            return Ok(unavailable_receipt("receipt_invalid", 1));
        }
        match read_cap_file_nofollow_bounded(
            &receipts,
            &receipt_name,
            &receipt_path,
            MAX_REUSABLE_RECEIPT_BYTES,
        ) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(unavailable_receipt("receipt_unreadable", 1)),
        }
    };
    let receipt: cockpit_evidence::ReusableReceipt = match serde_json::from_slice(&receipt_bytes) {
        Ok(receipt) => receipt,
        Err(_) => return Ok(unavailable_receipt("receipt_invalid", 2)),
    };
    if receipt.validate().is_err()
        || receipt.receipt_id != *receipt_id
        || receipt.node_id != binding.node_id
        || receipt.context.profile_digest != binding.profile_digest
    {
        return Ok(unavailable_receipt("receipt_invalid", 2));
    }
    Ok(ReceiptStoreLoad::Candidate {
        receipt: Box::new(receipt),
        files_read: 2,
    })
}

pub fn persist_reusable_receipt(
    root: &Path,
    binding: &ReceiptStoreBinding,
    receipt: &cockpit_evidence::ReusableReceipt,
) -> Result<ReceiptStoreWrite, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let expected_repository_id = repository_id(&root).to_string();
    if binding.repository_id != expected_repository_id
        || !valid_sha256_digest(&binding.profile_digest)
        || !valid_node_id(&binding.node_id)
        || receipt.validate().is_err()
        || !receipt.passed
        || receipt.node_id != binding.node_id
        || receipt.context.profile_digest != binding.profile_digest
    {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence/reuse"),
            message: "invalid reusable receipt store binding".into(),
        });
    }

    let root_dir =
        Dir::open_ambient_dir(&root, cap_std::ambient_authority()).map_err(|source| {
            ObserverError::Read {
                path: root.clone(),
                source,
            }
        })?;
    let ai_path = root.join(".ai");
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &ai_path)?;
    let evidence_path = ai_path.join("evidence");
    let evidence = create_and_open_cap_directory(&ai, "evidence", &evidence_path)?;
    let reuse_path = evidence_path.join("reuse");
    let reuse = create_and_open_cap_directory(&evidence, "reuse", &reuse_path)?;
    let receipts_path = reuse_path.join("receipts");
    let receipts = create_and_open_cap_directory(&reuse, "receipts", &receipts_path)?;

    let lock_path = reuse_path.join("index.lock");
    let lock = open_or_create_cap_nofollow(&reuse, "index.lock", &lock_path)?;
    lock.lock().map_err(|source| ObserverError::Read {
        path: lock_path,
        source,
    })?;

    let receipt_name = receipt_file_name(&receipt.receipt_id);
    let receipt_path = receipts_path.join(&receipt_name);
    let receipt_bytes =
        serde_json::to_vec_pretty(receipt).map_err(|error| ObserverError::State {
            path: receipt_path.clone(),
            message: error.to_string(),
        })?;
    let mut files_read =
        write_cap_immutable(&receipts, &receipt_name, &receipt_path, &receipt_bytes)?;

    let index_path = reuse_path.join("index.json");
    let mut index = if let Some(bytes) = read_optional_cap_file_nofollow_bounded(
        &reuse,
        "index.json",
        &index_path,
        MAX_RECEIPT_INDEX_BYTES,
    )? {
        files_read += 1;
        let existing: ReceiptStoreIndex =
            serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
                path: index_path.clone(),
                message: format!("invalid reusable receipt index: {error}"),
            })?;
        if existing.schema_version != 1 || existing.repository_id != binding.repository_id {
            return Err(ObserverError::State {
                path: index_path,
                message: "reusable receipt index binding mismatch".into(),
            });
        }
        if existing.profile_digest == binding.profile_digest {
            existing
        } else {
            ReceiptStoreIndex {
                schema_version: 1,
                repository_id: binding.repository_id.clone(),
                profile_digest: binding.profile_digest.clone(),
                receipts: BTreeMap::new(),
            }
        }
    } else {
        ReceiptStoreIndex {
            schema_version: 1,
            repository_id: binding.repository_id.clone(),
            profile_digest: binding.profile_digest.clone(),
            receipts: BTreeMap::new(),
        }
    };
    index
        .receipts
        .insert(binding.node_id.clone(), receipt.receipt_id.clone());
    let index_bytes = serde_json::to_vec_pretty(&index).map_err(|error| ObserverError::State {
        path: index_path.clone(),
        message: error.to_string(),
    })?;
    write_cap_commit_marker(&reuse, &reuse_path)?;
    atomic_replace_cap_strict(&reuse, "index.json", &index_path, &index_bytes)?;
    reuse
        .remove_file("index.pending")
        .map_err(|source| ObserverError::Read {
            path: reuse_path.join("index.pending"),
            source,
        })?;
    // The index rename is the logical commit. If syncing marker removal fails,
    // a crash can only resurrect the marker, which makes future reads fail closed.
    let _ = sync_cap_directory(&reuse, &reuse_path.join("index.pending"));
    Ok(ReceiptStoreWrite { files_read })
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

fn unavailable_receipt(reason: &str, files_read: usize) -> ReceiptStoreLoad {
    ReceiptStoreLoad::Unavailable {
        reason: reason.into(),
        files_read,
    }
}

fn valid_sha256_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn receipt_file_name(receipt_id: &str) -> String {
    format!("{}.json", &receipt_id["sha256:".len()..])
}

fn valid_node_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn open_cap_directory_nofollow(
    parent: &Dir,
    name: &str,
    _display_path: &Path,
) -> Result<Dir, &'static str> {
    #[cfg(windows)]
    {
        // cap-std's Windows `symlink_metadata` implementation asks
        // `CreateFileAtW` for a zero-access handle.  On the hosted Windows
        // runner that relative metadata probe returns `ERROR_ACCESS_DENIED`
        // even for an ordinary child directory.  Open the directory handle
        // directly instead, then inspect the handle metadata; this also
        // avoids a metadata/open TOCTOU window.
        let directory = match parent.open_dir_nofollow(name) {
            Ok(directory) => directory,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err("evidence_missing");
            }
            Err(_) => return Err("store_path_unreadable"),
        };
        let metadata = directory
            .dir_metadata()
            .map_err(|_| "store_path_unreadable")?;
        if metadata.file_type().is_symlink() {
            return Err("symlink_rejected");
        }
        if !metadata.is_dir() {
            return Err("store_path_invalid");
        }
        return Ok(directory);
    }

    #[cfg(not(windows))]
    match parent.symlink_metadata(name) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err("symlink_rejected"),
        Ok(metadata) if !metadata.is_dir() => Err("store_path_invalid"),
        Ok(_) => parent
            .open_dir_nofollow(name)
            .map_err(|_| "store_path_unreadable"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err("evidence_missing"),
        Err(_) => Err("store_path_unreadable"),
    }
}

fn open_cap_directory_nofollow_strict(
    parent: &Dir,
    name: &str,
    display_path: &Path,
) -> Result<Dir, ObserverError> {
    open_cap_directory_nofollow(parent, name, display_path).map_err(|reason| ObserverError::State {
        path: display_path.to_path_buf(),
        message: format!("receipt store directory rejected: {reason}"),
    })
}

fn create_and_open_cap_directory(
    parent: &Dir,
    name: &str,
    display_path: &Path,
) -> Result<Dir, ObserverError> {
    match parent.create_dir(name) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(source) => {
            return Err(ObserverError::Read {
                path: display_path.to_path_buf(),
                source,
            });
        }
    }
    open_cap_directory_nofollow_strict(parent, name, display_path)
}

fn cap_read_options() -> CapOpenOptions {
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    options
}

fn open_cap_existing_nofollow(
    parent: &Dir,
    name: &str,
    display_path: &Path,
) -> Result<fs::File, ObserverError> {
    let file = parent
        .open_with(name, &cap_read_options())
        .map_err(|source| ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        })?
        .into_std();
    if !file
        .metadata()
        .map_err(|source| ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        })?
        .is_file()
    {
        return Err(ObserverError::State {
            path: display_path.to_path_buf(),
            message: "receipt store entry must be a real file".into(),
        });
    }
    Ok(file)
}

fn open_or_create_cap_nofollow(
    parent: &Dir,
    name: &str,
    display_path: &Path,
) -> Result<fs::File, ObserverError> {
    let mut existing = CapOpenOptions::new();
    existing.read(true).write(true).follow(FollowSymlinks::No);
    let file = match parent.open_with(name, &existing) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut create = CapOpenOptions::new();
            create
                .read(true)
                .write(true)
                .create_new(true)
                .follow(FollowSymlinks::No);
            match parent.open_with(name, &create) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => parent
                    .open_with(name, &existing)
                    .map_err(|source| ObserverError::Read {
                        path: display_path.to_path_buf(),
                        source,
                    })?,
                Err(source) => {
                    return Err(ObserverError::Read {
                        path: display_path.to_path_buf(),
                        source,
                    });
                }
            }
        }
        Err(source) => {
            return Err(ObserverError::Read {
                path: display_path.to_path_buf(),
                source,
            });
        }
    }
    .into_std();
    if !file
        .metadata()
        .map_err(|source| ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        })?
        .is_file()
    {
        return Err(ObserverError::State {
            path: display_path.to_path_buf(),
            message: "receipt store lock must be a real file".into(),
        });
    }
    Ok(file)
}

fn read_cap_file_nofollow_bounded(
    parent: &Dir,
    name: &str,
    display_path: &Path,
    maximum_bytes: u64,
) -> Result<Vec<u8>, ObserverError> {
    let mut file = open_cap_existing_nofollow(parent, name, display_path)?;
    let mut bytes = Vec::new();
    std::io::Read::by_ref(&mut file)
        .take(maximum_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        })?;
    if bytes.len() as u64 > maximum_bytes {
        return Err(ObserverError::State {
            path: display_path.to_path_buf(),
            message: "receipt store entry exceeds the bounded read limit".into(),
        });
    }
    Ok(bytes)
}

fn read_optional_cap_file_nofollow_bounded(
    parent: &Dir,
    name: &str,
    display_path: &Path,
    maximum_bytes: u64,
) -> Result<Option<Vec<u8>>, ObserverError> {
    #[cfg(windows)]
    {
        // See `open_cap_directory_nofollow`: probing a child with
        // `symlink_metadata` is not usable with the Windows capability
        // handle implementation on the hosted runner.  Open once with
        // no-follow semantics and read from that pinned handle.
        //
        // cap-std currently reports `ERROR_ACCESS_DENIED` for a missing leaf
        // when it performs that relative open.  Use the canonical display
        // path only to distinguish the absent case; any existing entry is
        // still opened and read through the capability handle below, so the
        // ambient probe cannot authorize a substituted file.
        match fs::symlink_metadata(display_path) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(ObserverError::Read {
                    path: display_path.to_path_buf(),
                    source,
                });
            }
        }
        let mut file = match parent.open_with(name, &cap_read_options()) {
            Ok(file) => file.into_std(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(ObserverError::Read {
                    path: display_path.to_path_buf(),
                    source,
                });
            }
        };
        let metadata = file.metadata().map_err(|source| ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        })?;
        if metadata.file_type().is_symlink() {
            return Err(ObserverError::State {
                path: display_path.to_path_buf(),
                message: "receipt store entry must not be a symlink".into(),
            });
        }
        if !metadata.is_file() {
            return Err(ObserverError::State {
                path: display_path.to_path_buf(),
                message: "receipt store entry must be a real file".into(),
            });
        }
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(maximum_bytes.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|source| ObserverError::Read {
                path: display_path.to_path_buf(),
                source,
            })?;
        if bytes.len() as u64 > maximum_bytes {
            return Err(ObserverError::State {
                path: display_path.to_path_buf(),
                message: "receipt store entry exceeds the bounded read limit".into(),
            });
        }
        return Ok(Some(bytes));
    }

    #[cfg(not(windows))]
    match parent.symlink_metadata(name) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(ObserverError::State {
                path: display_path.to_path_buf(),
                message: "receipt store entry must be a real file".into(),
            })
        }
        Ok(_) => {
            read_cap_file_nofollow_bounded(parent, name, display_path, maximum_bytes).map(Some)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        }),
    }
}

fn write_cap_immutable(
    parent: &Dir,
    name: &str,
    display_path: &Path,
    bytes: &[u8],
) -> Result<usize, ObserverError> {
    let sequence = NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed);
    let temporary = format!("{name}.receipt-tmp-{}-{sequence}", std::process::id());
    let mut options = CapOpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = parent
        .open_with(&temporary, &options)
        .map_err(|source| ObserverError::Read {
            path: display_path.with_file_name(&temporary),
            source,
        })?
        .into_std();
    if let Err(source) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = parent.remove_file(&temporary);
        return Err(ObserverError::Read {
            path: display_path.with_file_name(&temporary),
            source,
        });
    }
    drop(file);
    let installed = match parent.hard_link(&temporary, parent, name) {
        Ok(()) => Ok(0),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if read_cap_file_nofollow_bounded(parent, name, display_path, bytes.len() as u64)?
                == bytes
            {
                Ok(1)
            } else {
                Err(ObserverError::State {
                    path: display_path.to_path_buf(),
                    message: "immutable receipt already exists with different content".into(),
                })
            }
        }
        Err(source) => Err(ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        }),
    };
    let _ = parent.remove_file(&temporary);
    let files_read = installed?;
    sync_cap_directory(parent, display_path)?;
    Ok(files_read)
}

fn write_cap_commit_marker(parent: &Dir, display_path: &Path) -> Result<(), ObserverError> {
    let marker_path = display_path.join("index.pending");
    let mut options = CapOpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut marker = parent
        .open_with("index.pending", &options)
        .map_err(|source| ObserverError::Read {
            path: marker_path.clone(),
            source,
        })?
        .into_std();
    marker
        .write_all(b"pending-index-commit-v1\n")
        .and_then(|()| marker.sync_all())
        .map_err(|source| ObserverError::Read {
            path: marker_path.clone(),
            source,
        })?;
    sync_cap_directory(parent, &marker_path)
}

fn atomic_replace_cap_strict(
    parent: &Dir,
    name: &str,
    display_path: &Path,
    bytes: &[u8],
) -> Result<(), ObserverError> {
    if let Ok(metadata) = parent.symlink_metadata(name)
        && (metadata.file_type().is_symlink() || !metadata.is_file())
    {
        return Err(ObserverError::State {
            path: display_path.to_path_buf(),
            message: "receipt store entry must be a real file".into(),
        });
    }
    let sequence = NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed);
    let temporary = format!("{name}.tmp-{}-{sequence}", std::process::id());
    let mut options = CapOpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    #[cfg(windows)]
    {
        use cap_std::fs::OpenOptionsExt;
        use windows_sys::Win32::{
            Foundation::{GENERIC_READ, GENERIC_WRITE},
            Storage::FileSystem::DELETE,
        };
        // FileRenameInfoEx requires DELETE access on the source handle.  A
        // write-only temporary file is sufficient on Unix but is rejected by
        // Windows with ERROR_ACCESS_DENIED during the first index publish.
        options.access_mode(GENERIC_READ | GENERIC_WRITE | DELETE);
    }
    let mut file = parent
        .open_with(&temporary, &options)
        .map_err(|source| ObserverError::Read {
            path: display_path.with_file_name(&temporary),
            source,
        })?
        .into_std();
    if let Err(source) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = parent.remove_file(&temporary);
        return Err(ObserverError::Read {
            path: display_path.with_file_name(&temporary),
            source,
        });
    }
    let replace_result = replace_cap_entry(parent, &file, &temporary, name, display_path);
    drop(file);
    if let Err(source) = replace_result {
        let _ = parent.remove_file(&temporary);
        return Err(ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        });
    }
    sync_cap_directory(parent, display_path)?;
    Ok(())
}

fn sync_cap_directory(parent: &Dir, display_path: &Path) -> Result<(), ObserverError> {
    #[cfg(windows)]
    {
        let _ = (parent, display_path);
        return Ok(());
    }
    #[cfg(not(windows))]
    #[cfg(not(windows))]
    {
        // `cap-std` intentionally opens read-only directories with `O_PATH`
        // on Linux. `fsync(O_PATH)` returns EBADF, so reopen `.` relative to
        // the already-pinned directory capability with ordinary read access
        // before syncing. This remains handle-relative and avoids an ambient
        // path race.
        let mut options = CapOpenOptions::new();
        options.read(true);
        parent
            .open_with(".", &options)
            .map(|directory| directory.into_std().sync_all())
            .and_then(|result| result)
            .map_err(|source| ObserverError::Read {
                path: display_path.parent().unwrap_or(display_path).to_path_buf(),
                source,
            })
    }
}

#[cfg(not(windows))]
fn replace_cap_entry(
    parent: &Dir,
    _temporary_file: &std::fs::File,
    temporary: &str,
    name: &str,
    _display_path: &Path,
) -> std::io::Result<()> {
    parent.rename(temporary, parent, name)
}

#[cfg(windows)]
fn replace_cap_entry(
    _parent: &Dir,
    temporary_file: &std::fs::File,
    _temporary: &str,
    _name: &str,
    display_path: &Path,
) -> std::io::Result<()> {
    use std::os::windows::{ffi::OsStrExt, io::AsRawHandle};
    use windows_sys::Win32::{
        Storage::FileSystem::{
            FILE_RENAME_INFO, FILE_RENAME_INFO_0, FileRenameInfoEx, SetFileInformationByHandle,
        },
        System::WindowsProgramming::{
            FILE_RENAME_FLAG_POSIX_SEMANTICS, FILE_RENAME_FLAG_REPLACE_IF_EXISTS,
        },
    };

    // FileRenameInfoEx rejects a relative name paired with RootDirectory on
    // the Windows runners (ERROR_INVALID_PARAMETER). The parent capability
    // remains open with delete sharing disabled, so its absolute path cannot
    // be redirected while this operation is in flight. Keeping the Ex class
    // preserves replace/POSIX semantics for an existing target.
    let display_path = display_path.to_string_lossy();
    // cap-std canonical paths on Windows use the extended-length `\\?\\`
    // prefix. FileRenameInfoEx accepts the ordinary DOS spelling for these
    // short local paths, while treating the extended spelling as an invalid
    // rename target on some runner images.
    let display_path = display_path.strip_prefix(r"\\?\").unwrap_or(&display_path);
    let name = std::ffi::OsStr::new(display_path)
        .encode_wide()
        .collect::<Vec<_>>();
    let header_size = std::mem::offset_of!(FILE_RENAME_INFO, FileName);
    // `FILE_RENAME_INFO` declares `FileName[1]`.  Even though
    // `FileNameLength` excludes a terminator, the buffer passed to
    // `SetFileInformationByHandle` must still include that inline slot.
    let byte_len = header_size + (name.len() + 1) * std::mem::size_of::<u16>();
    let word_len = byte_len.div_ceil(std::mem::size_of::<usize>());
    let mut storage = vec![0_usize; word_len];
    let information = storage.as_mut_ptr().cast::<FILE_RENAME_INFO>();
    unsafe {
        (*information).Anonymous = FILE_RENAME_INFO_0 {
            Flags: FILE_RENAME_FLAG_REPLACE_IF_EXISTS | FILE_RENAME_FLAG_POSIX_SEMANTICS,
        };
        (*information).RootDirectory = std::ptr::null_mut();
        (*information).FileNameLength = (name.len() * std::mem::size_of::<u16>()) as u32;
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            (*information).FileName.as_mut_ptr(),
            name.len(),
        );
    }
    let result = unsafe {
        SetFileInformationByHandle(
            temporary_file.as_raw_handle().cast(),
            FileRenameInfoEx,
            information.cast(),
            byte_len as u32,
        )
    };
    if result == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
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
        capabilities: vec![
            "inspect".into(),
            "observe".into(),
            "status".into(),
            "preflight".into(),
            "verify".into(),
            "work-item-scaffold".into(),
            "profile-propose".into(),
            "knowledge".into(),
            "doctor".into(),
            "mcp".into(),
        ],
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

pub fn status(root: &Path) -> Result<RepositoryStatus, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let ai = root.join(".ai");
    let config_path = ai.join("cockpit.toml");
    let config_text = fs::read_to_string(&config_path).map_err(|source| ObserverError::Read {
        path: config_path.clone(),
        source,
    })?;
    let config: RepositoryConfig =
        toml::from_str(&config_text).map_err(|error| ObserverError::State {
            path: config_path.clone(),
            message: error.to_string(),
        })?;
    validate_protocol_version(config.protocol_version).map_err(|error| ObserverError::State {
        path: config_path.clone(),
        message: error.to_string(),
    })?;
    let profile_bytes =
        fs::read(ai.join("project.json")).map_err(|source| ObserverError::Read {
            path: ai.join("project.json"),
            source,
        })?;
    let profile: AttachedProfile =
        serde_json::from_slice(&profile_bytes).map_err(|error| ObserverError::State {
            path: ai.join("project.json"),
            message: error.to_string(),
        })?;
    if profile.repository_id != config.repository_id
        || config.repository_id != repository_id(&root).to_string()
    {
        return Err(ObserverError::State {
            path: config_path,
            message: "repository identity does not match protocol state".into(),
        });
    }
    let readiness = repository_readiness(&root)?;
    Ok(RepositoryStatus {
        protocol_version: config.protocol_version,
        repository_schema_version: config.repository_schema_version,
        repository_id: profile.repository_id,
        state: profile.state,
        profile_version: profile.profile_version,
        active_work_items: count_suffix(&ai.join("work-items/active"), ".contract.json"),
        archived_work_items: count_suffix(&ai.join("work-items/archive"), ".archive.json"),
        readiness,
    })
}

/// Readiness is a deterministic, read-only projection used before entering a
/// new Work Item.  It deliberately does not become a process-global
/// scheduler: every invocation resolves one repository root and one fresh
/// snapshot.  Missing remote metadata is represented as `unknown`, never as
/// a green `ready_on_base` claim.
fn repository_readiness(root: &Path) -> Result<RepositoryReadiness, ObserverError> {
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
    let current_branch = git_text(&root, &["symbolic-ref", "--quiet", "--short", "HEAD"])
        .filter(|value| !value.is_empty());
    let current_revision = snapshot.head.clone();
    let default_base = discover_default_base(&root);
    let dirty_paths = non_governance_changed_paths(&snapshot);
    let unclosed_archived_work_items = unclosed_archived_work_items(&root)?;
    let active_work_items = count_suffix(&root.join(".ai/work-items/active"), ".contract.json");

    let mut blockers = Vec::new();
    if active_work_items > 0 {
        blockers.push("active_work_items_present".into());
    }
    if !unclosed_archived_work_items.is_empty() {
        blockers.push("archived_work_items_pending_close".into());
    }
    if !dirty_paths.is_empty() {
        blockers.push("working_tree_dirty_before_start".into());
    }
    if current_branch.is_none() {
        blockers.push("detached_head".into());
    }
    if let (Some(current), Some(default)) = (&current_revision, &default_base)
        && current != &default.revision
    {
        blockers.push("base_revision_not_synchronized".into());
    }

    let mut unknowns = Vec::new();
    if default_base.is_none() {
        unknowns.push("default_base_unknown".into());
    }
    if current_revision.is_none() {
        unknowns.push("current_revision_unknown".into());
    }
    if blockers.is_empty() && current_branch.is_some() && default_base.is_some() {
        unknowns.clear();
    }
    blockers.sort();
    blockers.dedup();
    unknowns.sort();
    unknowns.dedup();
    let ready_on_base = blockers.is_empty() && unknowns.is_empty();
    let state = if !blockers.is_empty() {
        "blocked"
    } else if !unknowns.is_empty() {
        "unknown"
    } else {
        "ready_on_base"
    };
    Ok(RepositoryReadiness {
        state: state.into(),
        ready_on_base,
        blockers,
        unknowns,
        current_branch,
        default_remote: default_base.as_ref().map(|base| base.remote.clone()),
        default_branch: default_base.as_ref().map(|base| base.branch.clone()),
        current_revision,
        default_revision: default_base.map(|base| base.revision),
        dirty_paths,
        unclosed_archived_work_items,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DefaultBaseRef {
    remote: String,
    branch: String,
    revision: String,
}

/// Resolve the locally known remote default branch without network access.
/// A missing or ambiguous symbolic ref is intentionally unknown; guessing
/// `main`/`master` would turn an unproven base into authorization.
fn discover_default_base(root: &Path) -> Option<DefaultBaseRef> {
    let remotes = git_text(root, &["remote"])?;
    let mut candidates = Vec::new();
    for remote in remotes
        .lines()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let symbolic_path = format!("refs/remotes/{remote}/HEAD");
        let Some(symbolic) = git_text(root, &["symbolic-ref", "--quiet", &symbolic_path]) else {
            continue;
        };
        let prefix = format!("refs/remotes/{remote}/");
        let Some(branch) = symbolic.strip_prefix(&prefix).map(str::trim) else {
            continue;
        };
        if branch.is_empty() {
            continue;
        }
        let Some(revision) = git_text(root, &["rev-parse", "--verify", &symbolic]) else {
            continue;
        };
        if revision.is_empty() {
            continue;
        }
        candidates.push(DefaultBaseRef {
            remote: remote.into(),
            branch: branch.into(),
            revision,
        });
    }
    if candidates.len() == 1 {
        candidates.pop()
    } else {
        None
    }
}

fn non_governance_changed_paths(snapshot: &RepositorySnapshot) -> Vec<String> {
    let mut paths = snapshot
        .changed_paths
        .iter()
        .filter(|path| !path.starts_with(".ai/") && path.as_str() != ".ai")
        .cloned()
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    paths
}

fn unclosed_archived_work_items(root: &Path) -> Result<Vec<String>, ObserverError> {
    let archive = root.join(".ai/work-items/archive");
    let expected_repository_id = repository_id(root).to_string();
    let entries = match fs::read_dir(&archive) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(ObserverError::Read {
                path: archive,
                source,
            });
        }
    };
    let mut archived_ids = std::collections::BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: archive.clone(),
            source,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some(work_item_id) = name.strip_suffix(".archive.json") {
            archived_ids.insert(work_item_id.to_owned());
        } else if let Some(work_item_id) = name.strip_suffix(".contract.json") {
            // A partially written archive is still an unresolved lifecycle
            // boundary; do not let a missing manifest hide it from entry.
            archived_ids.insert(work_item_id.to_owned());
        }
    }
    let mut pending = archived_ids
        .into_iter()
        .filter(|work_item_id| {
            archive_requires_close(root, work_item_id)
                && !close_decision_is_valid_for_status(root, work_item_id, &expected_repository_id)
        })
        .collect::<Vec<_>>();
    pending.sort();
    pending.dedup();
    Ok(pending)
}

/// New archive manifests explicitly opt into the close gate.  Older archive
/// bytes predate that gate and remain historical, so they do not deadlock
/// entry into a new Work Item. Superseded archives are resolved by their
/// recovery successor and never require a second close decision.
fn archive_requires_close(root: &Path, work_item_id: &str) -> bool {
    let path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.archive.json"));
    let Ok(manifest) = read_json(&path) else {
        return false;
    };
    manifest.get("state").and_then(serde_json::Value::as_str) == Some("archived")
        && manifest
            .get("closeRequired")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
}

fn validate_start_entry(root: &Path, reject_unclosed_archives: bool) -> Result<(), ObserverError> {
    let readiness = repository_readiness(root)?;
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
    if readiness
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
    if failures.is_empty() {
        return Ok(());
    }
    failures.sort();
    Err(ObserverError::State {
        path: PathBuf::from(root),
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

pub fn start_work_item(
    root: &Path,
    work_item_id: &str,
    intent: &str,
    goal: &str,
    scope: &[String],
) -> Result<LifecycleReceipt, ObserverError> {
    start_work_item_with_options(
        root,
        work_item_id,
        intent,
        goal,
        scope,
        &WorkItemStartOptions {
            risk: "normal".into(),
            authority: "missing".into(),
            ..WorkItemStartOptions::default()
        },
    )
}

pub fn start_work_item_with_options(
    root: &Path,
    work_item_id: &str,
    intent: &str,
    goal: &str,
    scope: &[String],
    options: &WorkItemStartOptions,
) -> Result<LifecycleReceipt, ObserverError> {
    // Recovery-generated `not_ready` scaffolds are an explicit continuation
    // of an existing lifecycle and may be activated while their predecessor
    // is still awaiting closure.  All ordinary starts must pass the same
    // repository entry gate as `work-item new`.
    let recovery_continuation = recovery_scaffold_exists(root, work_item_id);
    validate_start_entry(root, !recovery_continuation)?;
    if let Some(receipt) =
        activate_not_ready_scaffold(root, work_item_id, intent, goal, scope, options)?
    {
        return Ok(receipt);
    }
    if !recovery_continuation {
        ensure_no_unclosed_archived_work_items(root)?;
    }
    create_work_item_scaffold(
        root,
        &ContractScaffoldInput {
            work_item_id,
            mode: "implementation",
            intent,
            goal,
            scope,
            options,
            state: "implementation_active",
        },
    )?;
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "implementation_active".into(),
        timestamp: now(),
    })
}

/// Activate a recovery-generated `not_ready` scaffold without replacing its
/// predecessor binding or repository facts.  A scaffold created by
/// `work-item recover` is intentionally reserved first; `start` is the
/// explicit human-owned transition that supplies governance fields and makes
/// it eligible for preflight.  Ordinary duplicate starts still fail closed.
fn activate_not_ready_scaffold(
    root: &Path,
    work_item_id: &str,
    intent: &str,
    goal: &str,
    scope: &[String],
    options: &WorkItemStartOptions,
) -> Result<Option<LifecycleReceipt>, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let active = root.join(".ai/work-items/active");
    let contract_path = active.join(format!("{work_item_id}.contract.json"));
    let summary_path = active.join(format!("{work_item_id}.summary.json"));
    if !contract_path.is_file() || !summary_path.is_file() {
        return Ok(None);
    }
    let mut contract = read_json(&contract_path)?;
    let mut summary = read_json(&summary_path)?;
    if contract["state"] != serde_json::json!("not_ready")
        || summary["state"] != serde_json::json!("not_ready")
    {
        return Ok(None);
    }
    if contract["workItemId"] != serde_json::json!(work_item_id)
        || summary["workItemId"] != serde_json::json!(work_item_id)
        || contract["repositoryId"] != serde_json::json!(repository_id(&root).to_string())
    {
        return Err(ObserverError::State {
            path: contract_path,
            message: "recovery scaffold identity does not match this repository or Work Item"
                .into(),
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
    let profile_path = root.join(".ai/project.json");
    let profile: AttachedProfile = read_json(&profile_path).and_then(|value| {
        serde_json::from_value(value).map_err(|error| ObserverError::State {
            path: profile_path.clone(),
            message: error.to_string(),
        })
    })?;
    let profile_digest = attached_profile_digest(&profile, &profile_path)?;
    let current_snapshot_digest = snapshot_digest(&snapshot)?;
    contract["repositoryId"] = serde_json::json!(profile.repository_id);
    contract["baseRevision"] =
        serde_json::json!(snapshot.head.clone().unwrap_or_else(|| "unborn".into()));
    contract["projectProfileDigest"] = serde_json::json!(profile_digest);
    contract["repositorySnapshotDigest"] = serde_json::json!(current_snapshot_digest);
    contract["state"] = serde_json::json!("implementation_active");
    contract["intent"] = serde_json::json!(intent);
    contract["goal"] = serde_json::json!(goal);
    contract["scope"] = serde_json::to_value(scope).map_err(|error| ObserverError::State {
        path: contract_path.clone(),
        message: error.to_string(),
    })?;
    contract["outOfScope"] =
        serde_json::to_value(&options.out_of_scope).map_err(|error| ObserverError::State {
            path: contract_path.clone(),
            message: error.to_string(),
        })?;
    contract["risk"] = serde_json::json!(options.risk);
    contract["authority"] = serde_json::json!(options.authority);
    contract["acceptanceCriteria"] =
        serde_json::to_value(&options.acceptance_criteria).map_err(|error| {
            ObserverError::State {
                path: contract_path.clone(),
                message: error.to_string(),
            }
        })?;
    contract["requiredEvidenceClasses"] = serde_json::to_value(&options.required_evidence_classes)
        .map_err(|error| ObserverError::State {
            path: contract_path.clone(),
            message: error.to_string(),
        })?;
    contract["verification"] = serde_json::json!(["cargo test --locked --workspace"]);
    contract["resourceContext"] = serde_json::to_value(provisional_resource_context(&root))
        .map_err(|error| ObserverError::State {
            path: contract_path.clone(),
            message: error.to_string(),
        })?;
    summary["state"] = serde_json::json!("implementation_active");
    summary["repositoryId"] = serde_json::json!(profile.repository_id);
    summary["changedPaths"] = serde_json::json!(snapshot.changed_paths);
    summary["checkpointCount"] = serde_json::json!(0);
    summary["preflightState"] = serde_json::json!("not_run");
    summary["updatedAt"] = serde_json::json!(now());
    atomic_json(&contract_path, &contract)?;
    atomic_json(&summary_path, &summary)?;
    Ok(Some(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "implementation_active".into(),
        timestamp: now(),
    }))
}

/// Create a deterministic, validator-readable Work Item skeleton.
///
/// This is the single scaffold writer used by both the transitional `start`
/// lifecycle and the user-facing `work-item new` command. The caller supplies
/// only fields that are explicitly human-owned; the repository facts are read
/// from one fresh snapshot and the attached profile.
pub fn scaffold_work_item(
    root: &Path,
    work_item_id: &str,
    mode: &str,
) -> Result<WorkItemScaffoldReceipt, ObserverError> {
    validate_start_entry(root, true)?;
    ensure_no_unclosed_archived_work_items(root)?;
    scaffold_work_item_internal(root, work_item_id, mode)
}

/// Create a recovery successor scaffold.  Recovery is not an independent
/// next Work Item, so it may be created while its predecessor is archived and
/// awaiting the explicit recovery/close decision.  It still uses the same
/// atomic scaffold writer and repository-local identity facts.
fn scaffold_work_item_for_recovery(
    root: &Path,
    work_item_id: &str,
    mode: &str,
) -> Result<WorkItemScaffoldReceipt, ObserverError> {
    scaffold_work_item_internal(root, work_item_id, mode)
}

fn scaffold_work_item_internal(
    root: &Path,
    work_item_id: &str,
    mode: &str,
) -> Result<WorkItemScaffoldReceipt, ObserverError> {
    let mode = mode.trim();
    if mode.is_empty() {
        return Err(ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: "work item mode must not be empty".into(),
        });
    }
    let options = WorkItemStartOptions {
        out_of_scope: Vec::new(),
        risk: "unknown".into(),
        authority: "unknown".into(),
        acceptance_criteria: Vec::new(),
        required_evidence_classes: Vec::new(),
    };
    let scaffold = create_work_item_scaffold(
        root,
        &ContractScaffoldInput {
            work_item_id,
            mode,
            intent: "",
            goal: "",
            scope: &[],
            options: &options,
            state: "not_ready",
        },
    )?;
    Ok(WorkItemScaffoldReceipt {
        work_item_id: work_item_id.into(),
        mode: mode.into(),
        contract_path: scaffold.contract_path,
        state: "not_ready".into(),
        known_facts: scaffold.facts,
        human_input_required: vec![
            "intent".into(),
            "scope".into(),
            "acceptanceCriteria".into(),
            "authority".into(),
        ],
    })
}

struct CreatedWorkItemScaffold {
    contract_path: String,
    facts: WorkItemScaffoldFacts,
}

struct ContractScaffoldInput<'a> {
    work_item_id: &'a str,
    mode: &'a str,
    intent: &'a str,
    goal: &'a str,
    scope: &'a [String],
    options: &'a WorkItemStartOptions,
    state: &'a str,
}

struct WorkItemScaffoldReservation {
    reservation_path: PathBuf,
    contract_path: PathBuf,
    summary_path: PathBuf,
    contract_created: bool,
    summary_created: bool,
    committed: bool,
}

impl WorkItemScaffoldReservation {
    fn mark_contract_created(&mut self) {
        self.contract_created = true;
    }

    fn mark_summary_created(&mut self) {
        self.summary_created = true;
    }

    fn commit(&mut self) {
        self.committed = true;
    }
}

impl Drop for WorkItemScaffoldReservation {
    fn drop(&mut self) {
        if !self.committed {
            if self.summary_created {
                let _ = fs::remove_file(&self.summary_path);
            }
            if self.contract_created {
                let _ = fs::remove_file(&self.contract_path);
            }
        }
        let _ = fs::remove_file(&self.reservation_path);
    }
}

fn create_work_item_scaffold(
    root: &Path,
    input: &ContractScaffoldInput<'_>,
) -> Result<CreatedWorkItemScaffold, ObserverError> {
    validate_work_item_id(input.work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let ai = root.join(".ai");
    if !ai.join("cockpit.toml").exists() {
        attach(&root)?;
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
    let profile_path = ai.join("project.json");
    let profile: AttachedProfile = read_json(&profile_path).and_then(|value| {
        serde_json::from_value(value).map_err(|error| ObserverError::State {
            path: profile_path.clone(),
            message: error.to_string(),
        })
    })?;
    let profile_digest = attached_profile_digest(&profile, &profile_path)?;
    let facts = WorkItemScaffoldFacts {
        repository_id: profile.repository_id.clone(),
        base_revision: snapshot.head.clone().unwrap_or_else(|| "unborn".into()),
        project_profile_digest: profile_digest,
        repository_snapshot_digest: snapshot_digest(&snapshot)?,
    };
    let now = now();
    let contract = serde_json::json!({
        "protocolVersion": 1,
        "repositoryId": facts.repository_id,
        "workItemId": input.work_item_id,
        "mode": input.mode,
        "state": input.state,
        "intent": input.intent,
        "goal": input.goal,
        "scope": input.scope,
        "outOfScope": input.options.out_of_scope.clone(),
        "risk": input.options.risk.clone(),
        "authority": input.options.authority.clone(),
        "acceptanceCriteria": input.options.acceptance_criteria.clone(),
        "requiredEvidenceClasses": input.options.required_evidence_classes.clone(),
        "sources": [],
        "verification": [],
        "baseRevision": facts.base_revision,
        "projectProfileDigest": facts.project_profile_digest,
        "repositorySnapshotDigest": facts.repository_snapshot_digest,
        "resourceContext": provisional_resource_context(&root),
        "createdAt": now,
    });
    let summary = serde_json::json!({
        "protocolVersion": 1,
        "repositoryId": facts.repository_id,
        "workItemId": input.work_item_id,
        "mode": input.mode,
        "state": input.state,
        "changedPaths": snapshot.changed_paths,
        "checkpointCount": 0,
        "preflightState": "not_run",
        "createdAt": now,
        "updatedAt": now,
    });
    let active = ai.join("work-items/active");
    let archive = ai.join("work-items/archive");
    // A repository may legitimately have no active Work Items after all
    // previous work has been archived.  Scaffolding a new item must restore
    // the protocol directories instead of failing on the first atomic write.
    for directory in [&active, &archive] {
        fs::create_dir_all(directory).map_err(|source| ObserverError::Read {
            path: directory.clone(),
            source,
        })?;
    }
    let contract_path = active.join(format!("{}.contract.json", input.work_item_id));
    let summary_path = active.join(format!("{}.summary.json", input.work_item_id));
    let archive_path = archive.join(format!("{}.archive.json", input.work_item_id));
    let reservation_path = active.join(format!(".{}.scaffold.reserve", input.work_item_id));
    let mut reservation_file = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&reservation_path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(ObserverError::State {
                path: reservation_path,
                message: "work item already exists or scaffold reservation is active".into(),
            });
        }
        Err(source) => {
            return Err(ObserverError::Read {
                path: reservation_path,
                source,
            });
        }
    };
    if let Err(source) = reservation_file
        .write_all(b"ai-cockpit work-item scaffold reservation\n")
        .and_then(|()| reservation_file.sync_all())
    {
        drop(reservation_file);
        let _ = fs::remove_file(&reservation_path);
        return Err(ObserverError::Read {
            path: reservation_path,
            source,
        });
    }
    drop(reservation_file);
    let mut reservation = WorkItemScaffoldReservation {
        reservation_path,
        contract_path: contract_path.clone(),
        summary_path: summary_path.clone(),
        contract_created: false,
        summary_created: false,
        committed: false,
    };
    if [contract_path.clone(), summary_path.clone(), archive_path]
        .iter()
        .any(|path| path.exists())
    {
        return Err(ObserverError::State {
            path: contract_path,
            message: "work item already exists".into(),
        });
    }
    atomic_json(&contract_path, &contract)?;
    reservation.mark_contract_created();
    atomic_json(&summary_path, &summary)?;
    reservation.mark_summary_created();
    reservation.commit();
    Ok(CreatedWorkItemScaffold {
        contract_path: contract_path.to_string_lossy().into_owned(),
        facts,
    })
}

pub fn checkpoint_work_item(
    root: &Path,
    work_item_id: &str,
) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let mut summary: serde_json::Value = read_json(&path)?;
    let count = summary["checkpointCount"].as_u64().unwrap_or(0);
    if count != 0 {
        return Err(ObserverError::State {
            path: path.clone(),
            message: "work item already has a checkpoint; duplicate checkpoint is not allowed"
                .into(),
        });
    }
    let state = summary["state"].as_str().unwrap_or("");
    if state != "implementation_active" {
        return Err(ObserverError::State {
            path: path.clone(),
            message: format!(
                "checkpoint is invalid from state {state:?}; expected implementation_active"
            ),
        });
    }
    let preflight_state = summary["preflightState"].as_str().unwrap_or("");
    if !matches!(preflight_state, "green" | "yellow") {
        return Err(ObserverError::State {
            path: path.clone(),
            message: format!(
                "checkpoint requires a recorded non-red preflight result (state={preflight_state:?})"
            ),
        });
    }
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let current_snapshot_digest = snapshot_digest(&snapshot)?.to_string();
    if summary["preflightRepositorySnapshotDigest"]
        .as_str()
        .is_none_or(|value| value != current_snapshot_digest)
    {
        return Err(ObserverError::State {
            path: path.clone(),
            message: "checkpoint requires a preflight result for the current repository snapshot"
                .into(),
        });
    }
    let current_contract_digest = contract_digest(&contract_path)?.to_string();
    if summary["preflightContractDigest"]
        .as_str()
        .is_none_or(|value| value != current_contract_digest)
    {
        return Err(ObserverError::State {
            path: path.clone(),
            message: "checkpoint requires a preflight result for the current contract".into(),
        });
    }
    require_green_or_yellow_preflight_governance(
        &root,
        &contract_path,
        &contract,
        &snapshot,
        preflight_state,
    )?;
    // `before_edit` is the authorization-to-edit boundary. Once any
    // verification result exists, recording that checkpoint would rewrite
    // phase ordering and could make post-verification work appear authorized
    // before execution. Keep this reference-defined boundary fail-closed.
    if summary
        .get("verification")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|entries| !entries.is_empty())
    {
        return Err(ObserverError::State {
            path: path.clone(),
            message: "before_edit checkpoint must be recorded before required verification".into(),
        });
    }
    let timestamp = now();
    if contract.checkpoint_policy.is_some() {
        append_checkpoint_evidence(
            &mut summary,
            &root,
            &contract,
            "before_edit",
            &snapshot,
            &current_contract_digest,
            0,
            &timestamp,
        )?;
    }
    summary["checkpointCount"] = 1.into();
    summary["state"] = "checkpointed".into();
    summary["checkpointAt"] = timestamp.clone().into();
    summary["checkpointContractDigest"] = current_contract_digest.into();
    summary["checkpointRepositorySnapshotDigest"] = current_snapshot_digest.into();
    summary["updatedAt"] = timestamp.clone().into();
    atomic_json(&path, &summary)?;
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "checkpointed".into(),
        timestamp,
    })
}

fn checkpoint_required_check_names(contract: &Contract) -> Vec<String> {
    let mut checks = required_verification_checks(contract);
    if let Some(policy) = contract.checkpoint_policy.as_ref() {
        checks.extend(policy.required_checks.iter().cloned());
    }
    checks.sort();
    checks.dedup();
    checks
}

fn checkpoint_passed_check_count(contract: &Contract, summary: &serde_json::Value) -> u64 {
    let names = checkpoint_required_check_names(contract);
    names
        .iter()
        .filter(|name| {
            summary
                .get("verification")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| {
                    items.iter().any(|item| {
                        item.get("check").and_then(serde_json::Value::as_str) == Some(name.as_str())
                            && item.get("result").and_then(serde_json::Value::as_str)
                                == Some("passed")
                    })
                })
        })
        .count() as u64
}

#[allow(clippy::too_many_arguments)]
fn append_checkpoint_evidence(
    summary: &mut serde_json::Value,
    root: &Path,
    contract: &Contract,
    stage: &str,
    snapshot: &RepositorySnapshot,
    contract_hash: &str,
    required_checks_passed: u64,
    recorded_at: &str,
) -> Result<(), ObserverError> {
    let evidence = summary
        .as_object_mut()
        .expect("Work Item Summary is an object")
        .entry("checkpointEvidence")
        .or_insert_with(|| serde_json::json!([]));
    let entries = evidence
        .as_array_mut()
        .ok_or_else(|| ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: "checkpointEvidence must be an array".into(),
        })?;
    if entries
        .iter()
        .any(|item| item.get("stage").and_then(serde_json::Value::as_str) == Some(stage))
    {
        return Err(ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: format!("checkpointEvidence stage {stage} is already recorded"),
        });
    }
    let required_checks = checkpoint_required_check_names(contract);
    entries.push(serde_json::json!({
        "schemaVersion": 1,
        "repositoryId": repository_id(root),
        "workItemId": contract.work_item_id,
        "stage": stage,
        "recorded": true,
        "contractHash": contract_hash,
        "repositorySnapshotDigest": snapshot_digest(snapshot)?,
        "acceptanceCount": contract.acceptance_criteria.len(),
        "unknownCount": contract.unknowns.len(),
        "requiredChecks": required_checks.len(),
        "requiredChecksPassed": required_checks_passed,
        "recordedAt": recorded_at,
    }));
    Ok(())
}

/// Append a Contract-amendment revalidation record without rewriting the
/// immutable `before_edit` checkpoint.  A post-verification amendment marks
/// every prior required result stale; a fresh preflight and verification must
/// clear that marker before finish/archive/close can proceed.
pub fn revalidate_contract_amendment(
    root: &Path,
    work_item_id: &str,
    reason: &str,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    if reason.trim().is_empty() {
        return Err(ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: "contract amendment reason must not be empty".into(),
        });
    }
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let active = root.join(".ai/work-items/active");
    let contract_path = active.join(format!("{work_item_id}.contract.json"));
    let summary_path = active.join(format!("{work_item_id}.summary.json"));
    let contract = read_contract(&contract_path)?;
    let mut summary = read_json(&summary_path)?;
    // Contracts created before typed checkpoint evidence was introduced may
    // still have the original checkpoint identity fields on Summary while
    // `checkpointEvidence` is absent.  Upgrade that deterministic legacy
    // record in-memory so an explicit amendment can proceed; the original
    // Contract/Summary bytes remain bound by the generated before_edit hash.
    if !summary
        .get("checkpointEvidence")
        .is_some_and(serde_json::Value::is_array)
    {
        let legacy_contract_hash = summary
            .get("checkpointContractDigest")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| ObserverError::State {
                path: summary_path.clone(),
                message: "contract amendment requires a legacy checkpoint Contract digest".into(),
            })?;
        let legacy_snapshot_digest = summary
            .get("checkpointRepositorySnapshotDigest")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| ObserverError::State {
                path: summary_path.clone(),
                message: "contract amendment requires a legacy checkpoint snapshot digest".into(),
            })?;
        let recorded_at = summary
            .get("checkpointAt")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| ObserverError::State {
                path: summary_path.clone(),
                message: "contract amendment requires a legacy checkpoint timestamp".into(),
            })?;
        if summary["checkpointCount"] != serde_json::json!(1) {
            return Err(ObserverError::State {
                path: summary_path.clone(),
                message: "contract amendment requires exactly one legacy checkpoint".into(),
            });
        }
        summary["checkpointEvidence"] = serde_json::json!([{
            "schemaVersion": 1,
            "repositoryId": repository_id(&root),
            "workItemId": work_item_id,
            "stage": "before_edit",
            "recorded": true,
            "contractHash": legacy_contract_hash,
            "repositorySnapshotDigest": legacy_snapshot_digest,
            "acceptanceCount": contract.acceptance_criteria.len(),
            "unknownCount": contract.unknowns.len(),
            "requiredChecks": 0,
            "requiredChecksPassed": 0,
            "recordedAt": recorded_at,
        }]);
    }
    let evidence = summary
        .get("checkpointEvidence")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| ObserverError::State {
            path: summary_path.clone(),
            message: "contract amendment requires typed checkpointEvidence".into(),
        })?;
    let before_edit = evidence
        .iter()
        .find(|entry| entry.get("stage").and_then(serde_json::Value::as_str) == Some("before_edit"))
        .cloned()
        .ok_or_else(|| ObserverError::State {
            path: summary_path.clone(),
            message: "contract amendment requires a before_edit checkpoint".into(),
        })?;
    let before_edit: CheckpointEvidence =
        serde_json::from_value(before_edit).map_err(|error| ObserverError::State {
            path: summary_path.clone(),
            message: format!("before_edit checkpoint is malformed: {error}"),
        })?;
    if evidence.iter().any(|entry| {
        entry.get("stage").and_then(serde_json::Value::as_str) == Some("before_finish")
    }) {
        return Err(ObserverError::State {
            path: summary_path,
            message: "contract amendment after before_finish requires a recovery Work Item".into(),
        });
    }
    let amendment_hashes = evidence
        .iter()
        .filter(|entry| {
            entry.get("stage").and_then(serde_json::Value::as_str)
                == Some("contract_amendment_revalidation")
        })
        .map(|entry| {
            serde_json::from_value::<CheckpointEvidence>(entry.clone()).map_err(|error| {
                ObserverError::State {
                    path: summary_path.clone(),
                    message: format!("contract amendment checkpoint is malformed: {error}"),
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let previous_contract_hash = amendment_hashes
        .last()
        .map(|entry| entry.contract_hash.clone())
        .unwrap_or_else(|| before_edit.contract_hash.clone());
    let current_contract_hash = contract_digest(&contract_path)?.to_string();
    if current_contract_hash == previous_contract_hash {
        return Err(ObserverError::State {
            path: contract_path,
            message: "contract amendment must change Contract bytes".into(),
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
    let required_checks = checkpoint_required_check_names(&contract);
    let required_checks_passed = checkpoint_passed_check_count(&contract, &summary);
    let verification_started = summary
        .get("verification")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|entries| {
            entries.iter().any(|entry| {
                matches!(
                    entry.get("result").and_then(serde_json::Value::as_str),
                    Some("passed" | "failed" | "warning" | "blocked")
                )
            })
        })
        // A legacy command-only Contract has no typed required gates to
        // invalidate. Keep the amendment's historical fact, but do not mark
        // it as a gate invalidation that must contain a non-empty list.
        && !required_checks.is_empty();
    let record = serde_json::json!({
        "schemaVersion": 1,
        "repositoryId": repository_id(&root),
        "workItemId": work_item_id,
        "stage": "contract_amendment_revalidation",
        "recorded": true,
        "contractHash": current_contract_hash.clone(),
        "repositorySnapshotDigest": snapshot_digest(&snapshot)?,
        "acceptanceCount": contract.acceptance_criteria.len(),
        "unknownCount": contract.unknowns.len(),
        "requiredChecks": required_checks.len(),
        "requiredChecksPassed": 0,
        "originalBeforeEditContractHash": before_edit.contract_hash,
        "previousContractHash": previous_contract_hash,
        "reason": reason.trim(),
        "verificationStarted": verification_started,
        "invalidatedRequiredChecks": if verification_started { required_checks.clone() } else { Vec::new() },
        "requiredChecksPassedAtAmendment": if verification_started { Some(required_checks_passed) } else { None },
        "recordedAt": now(),
    });
    let entries = summary
        .get_mut("checkpointEvidence")
        .and_then(serde_json::Value::as_array_mut)
        .expect("checkpointEvidence was validated as an array");
    entries.push(record.clone());
    // A Contract amendment invalidates the finish-ready projection.  Reopen
    // the single checkpointed recovery state so the normal preflight → verify
    // → finish path can be replayed without hand-editing generated Summary
    // bytes.  The previous Outcome/evidence remain immutable predecessor
    // facts; a fresh verification will replace the active projection.
    if summary["state"] == serde_json::json!("finish_ready") {
        summary["state"] = "checkpointed".into();
        if let Some(object) = summary.as_object_mut() {
            object.remove("failedGate");
            object.remove("recoveryCondition");
            object.remove("outcomeState");
        }
    }
    summary["preflightState"] = "not_run".into();
    summary["verificationInvalidatedByContractAmendment"] = serde_json::json!({
        "contractHash": current_contract_hash,
        "invalidatedRequiredChecks": if verification_started { required_checks } else { Vec::new() },
        "recordedAt": now(),
    });
    atomic_json(&summary_path, &summary)?;
    Ok(record)
}

/// Evaluate and persist the preflight decision for an active Work Item.
///
/// Preflight is intentionally a repository-local receipt rather than process
/// state.  A yellow result may be recorded before verification (for example,
/// when the Contract requires a verification receipt that does not exist yet),
/// but `finish` requires a fresh green result.  This keeps the documented
/// start -> preflight -> checkpoint -> verify path usable without weakening the
/// finish gate.
pub fn preflight_work_item(
    root: &Path,
    contract_path: &Path,
) -> Result<GovernanceDecision, ObserverError> {
    preflight_work_item_internal(root, contract_path, None)
}

/// Evaluate and persist preflight while binding evidence checks to the
/// Runtime executing the request.  CLI and MCP use this entry point so a
/// foreign Runtime receipt cannot make the lifecycle appear green.
pub fn preflight_work_item_with_runtime(
    root: &Path,
    contract_path: &Path,
    runtime: &RuntimeContext,
) -> Result<GovernanceDecision, ObserverError> {
    preflight_work_item_internal(root, contract_path, Some(runtime))
}

fn preflight_work_item_internal(
    root: &Path,
    contract_path: &Path,
    current_runtime: Option<&RuntimeContext>,
) -> Result<GovernanceDecision, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let contract_path = if contract_path.is_absolute() {
        contract_path.to_path_buf()
    } else {
        root.join(contract_path)
    };
    let contract = read_contract(&contract_path)?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let raw_decision = governance_decision_for_contract_base_internal_with_archive(
        &root,
        &contract,
        &snapshot,
        current_runtime,
        false,
    )?;
    let decision =
        apply_preflight_review_evidence(&root, &contract, &snapshot, raw_decision.clone(), false)?;

    let active = root.join(".ai/work-items/active");
    let active_contract = active.join(format!("{}.contract.json", contract.work_item_id));
    let summary_path = active.join(format!("{}.summary.json", contract.work_item_id));
    if active_contract.is_file() && summary_path.is_file() {
        let active_contract =
            fs::canonicalize(&active_contract).map_err(|source| ObserverError::Read {
                path: active_contract.clone(),
                source,
            })?;
        let requested_contract =
            fs::canonicalize(&contract_path).map_err(|source| ObserverError::Read {
                path: contract_path.clone(),
                source,
            })?;
        if active_contract != requested_contract {
            return Err(ObserverError::State {
                path: contract_path,
                message: "preflight contract is not the active Work Item contract".into(),
            });
        }
        let mut summary: serde_json::Value = read_json(&summary_path)?;
        require_current_retry_recovery_binding(
            &root,
            &contract.work_item_id,
            &summary,
            current_runtime,
        )?;
        let current_state = summary["state"].as_str().unwrap_or("");
        // A scaffold is intentionally not an active lifecycle item yet.  Keep
        // the historical read-only preflight behavior for this state so
        // callers can inspect the candidate decision before `start` supplies
        // the human governance fields and activates the item.
        if current_state == "not_ready" {
            let state = decision_state_name(decision.state.clone());
            let decision_value =
                serde_json::to_value(&raw_decision).map_err(|error| ObserverError::State {
                    path: active_contract.clone(),
                    message: error.to_string(),
                })?;
            summary["preflightState"] = state.into();
            summary["preflightDecisionDigest"] = cockpit_protocol::digest_json(&decision_value)
                .map_err(|error| ObserverError::State {
                    path: active_contract.clone(),
                    message: error.to_string(),
                })?
                .to_string()
                .into();
            summary["preflightRepositorySnapshotDigest"] =
                snapshot_digest(&snapshot)?.to_string().into();
            summary["preflightContractDigest"] =
                contract_digest(&active_contract)?.to_string().into();
            summary["preflightAt"] = now().into();
            atomic_json(&summary_path, &summary)?;
            return Ok(decision);
        }
        if !matches!(current_state, "implementation_active" | "checkpointed") {
            return Err(ObserverError::State {
                path: summary_path,
                message: format!(
                    "preflight is invalid from state {current_state:?}; expected implementation_active or checkpointed"
                ),
            });
        }
        let state = decision_state_name(decision.state.clone());
        let decision_value =
            serde_json::to_value(&raw_decision).map_err(|error| ObserverError::State {
                path: active_contract.clone(),
                message: error.to_string(),
            })?;
        summary["preflightState"] = state.into();
        summary["preflightDecisionDigest"] = cockpit_protocol::digest_json(&decision_value)
            .map_err(|error| ObserverError::State {
                path: active_contract.clone(),
                message: error.to_string(),
            })?
            .to_string()
            .into();
        summary["preflightRepositorySnapshotDigest"] =
            snapshot_digest(&snapshot)?.to_string().into();
        summary["preflightContractDigest"] = contract_digest(&active_contract)?.to_string().into();
        summary["preflightAt"] = now().into();
        atomic_json(&summary_path, &summary)?;
    }
    Ok(decision)
}

fn decision_state_name(state: DecisionState) -> &'static str {
    match state {
        DecisionState::Green => "green",
        DecisionState::Yellow => "yellow",
        DecisionState::Red => "red",
    }
}

fn contract_digest(path: &Path) -> Result<Digest, ObserverError> {
    let contract: serde_json::Value = read_json(path)?;
    cockpit_protocol::digest_json(&contract).map_err(|error| ObserverError::State {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}

fn contract_digest_for_evidence(
    root: &Path,
    contract: &cockpit_protocol::Contract,
) -> Result<Digest, ObserverError> {
    let active = root
        .join(".ai/work-items/active")
        .join(format!("{}.contract.json", contract.work_item_id));
    let path = if active.is_file() {
        active
    } else {
        root.join(".ai/work-items/archive")
            .join(format!("{}.contract.json", contract.work_item_id))
    };
    contract_digest(&path)
}

fn require_green_or_yellow_preflight_governance(
    root: &Path,
    contract_path: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    preflight_state: &str,
) -> Result<(), ObserverError> {
    let decision = governance_decision_for_contract(root, contract, snapshot)?;
    let current_state = decision_state_name(decision.state.clone());
    if current_state == "red" || preflight_state == "red" {
        return Err(ObserverError::State {
            path: contract_path.to_path_buf(),
            message: format!(
                "checkpoint requires a non-red governance result (preflight={preflight_state}, current={})",
                current_state
            ),
        });
    }
    if preflight_state == "yellow"
        && matches!(
            decision.review_state.as_deref(),
            Some("needs_human_confirmation")
        )
    {
        return Err(ObserverError::State {
            path: contract_path.to_path_buf(),
            message:
                "checkpoint requires human confirmation for an incomplete or uncertain Contract"
                    .into(),
        });
    }
    Ok(())
}

pub fn finish_work_item(
    root: &Path,
    work_item_id: &str,
) -> Result<LifecycleReceipt, ObserverError> {
    let result = finish_work_item_internal(root, work_item_id, None);
    if let Err(error) = &result {
        let _ = persist_blocked_lifecycle_outcome(root, work_item_id, error);
    }
    result
}

/// Finish a Work Item while requiring evidence produced by the current
/// Runtime executable.  The unbound wrapper above is retained for embedders
/// that manage Runtime identity outside this crate; CLI/MCP use this bound
/// entry point.
pub fn finish_work_item_with_runtime(
    root: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> Result<LifecycleReceipt, ObserverError> {
    let result = finish_work_item_internal(root, work_item_id, Some(runtime));
    if let Err(error) = &result {
        let _ = persist_blocked_lifecycle_outcome(root, work_item_id, error);
    }
    result
}

fn finish_work_item_internal(
    root: &Path,
    work_item_id: &str,
    current_runtime: Option<&RuntimeContext>,
) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let active = root.join(".ai/work-items/active");
    let summary_path = active.join(format!("{work_item_id}.summary.json"));
    let mut summary: serde_json::Value = read_json(&summary_path)?;
    let summary_state = summary["state"].as_str().unwrap_or("");
    let retry_recovery_pending = summary["recoveryRetryPending"] == serde_json::json!(true);
    require_current_retry_recovery_binding(&root, work_item_id, &summary, current_runtime)?;
    if summary_state != "checkpointed" {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: format!(
                "finish is invalid from state {summary_state:?}; expected checkpointed"
            ),
        });
    }
    if summary["checkpointCount"] != serde_json::json!(1) {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "finish requires exactly one checkpoint".into(),
        });
    }
    if summary["preflightState"] != serde_json::json!("green") {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "finish requires a green preflight result after verification".into(),
        });
    }
    let contract_path = active.join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    if contract.checkpoint_policy.is_some() {
        let current_contract_hash = contract_digest(&contract_path)?.to_string();
        if summary["checkpointContractDigest"] != serde_json::json!(current_contract_hash) {
            return Err(ObserverError::State {
                path: summary_path.clone(),
                message: "finish requires a checkpoint for the current Contract".into(),
            });
        }
        let checkpoint_snapshot = summary["checkpointRepositorySnapshotDigest"]
            .as_str()
            .unwrap_or_default();
        let preflight_snapshot = summary["preflightRepositorySnapshotDigest"]
            .as_str()
            .unwrap_or_default();
        if checkpoint_snapshot.is_empty() || checkpoint_snapshot != preflight_snapshot {
            return Err(ObserverError::State {
                path: summary_path.clone(),
                message: "finish requires a checkpoint for the current repository snapshot".into(),
            });
        }
    }
    require_explicit_resource_finalization_plan(&contract, &contract_path, "finish")?;
    let original_summary = summary.clone();
    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    let evidence = read_json(&evidence_path).map_err(|_| ObserverError::State {
        path: evidence_path.clone(),
        message: "finish requires a recorded verification receipt".into(),
    })?;
    if evidence["workItemId"].as_str() != Some(work_item_id)
        || evidence["passed"] != serde_json::Value::Bool(true)
    {
        return Err(ObserverError::State {
            path: evidence_path,
            message: "verification receipt is not a passed receipt for this work item".into(),
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
    let current_digest = snapshot_digest(&snapshot)?;
    if summary["preflightRepositorySnapshotDigest"]
        .as_str()
        .is_none_or(|value| value != current_digest.as_str())
    {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "finish requires a green preflight result for the current repository snapshot"
                .into(),
        });
    }
    if evidence["repositorySnapshotDigest"] != serde_json::Value::String(current_digest.to_string())
    {
        return Err(ObserverError::State {
            path: evidence_path,
            message: "verification receipt is stale for the current repository snapshot".into(),
        });
    }
    let contract_value = read_json(&contract_path)?;
    let controls = if let Some(runtime) = current_runtime {
        validate_contract_summary_controls_with_runtime(
            &contract,
            &contract_value,
            &summary,
            runtime,
        )
    } else {
        validate_contract_summary_controls(&contract, &contract_value, &summary)
    };
    if controls.state == "blocked" {
        return Err(ObserverError::State {
            path: contract_path,
            message: format!(
                "Contract/Summary governance controls are blocked: {}",
                controls
                    .findings
                    .iter()
                    .map(|item| item.code.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        });
    }
    if verification_evidence_state(&root, &contract, &snapshot, false, current_runtime)?
        != EvidenceState::Complete
    {
        return Err(ObserverError::State {
            path: evidence_path,
            message: "verification evidence is not a valid current receipt".into(),
        });
    }
    if contract.checkpoint_policy.is_some() {
        let current_contract_hash = contract_digest(&contract_path)?.to_string();
        let has_before_finish = summary
            .get("checkpointEvidence")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|entries| {
                entries.iter().any(|entry| {
                    entry.get("stage").and_then(serde_json::Value::as_str) == Some("before_finish")
                })
            });
        if !has_before_finish {
            let passed = checkpoint_passed_check_count(&contract, &summary);
            append_checkpoint_evidence(
                &mut summary,
                &root,
                &contract,
                "before_finish",
                &snapshot,
                &current_contract_hash,
                passed,
                &now(),
            )?;
            atomic_json(&summary_path, &summary)?;
        }
        if let Err(errors) = validate_checkpoint_evidence_bindings(
            &contract,
            &summary,
            &repository_id(&root).to_string(),
            &current_digest.to_string(),
            &current_contract_hash,
        ) {
            return Err(ObserverError::State {
                path: summary_path.clone(),
                message: format!("checkpoint evidence is invalid: {}", errors.join(", ")),
            });
        }
    }
    if let Some(runtime) = current_runtime {
        require_green_governance_with_runtime(
            &root,
            &contract_path,
            &contract,
            &snapshot,
            "finish",
            runtime,
        )?;
    } else {
        require_green_governance(&root, &contract_path, &contract, &snapshot, "finish")?;
    }
    let timestamp = now();
    summary["state"] = "finish_ready".into();
    summary["updatedAt"] = timestamp.clone().into();
    atomic_json(&summary_path, &summary)?;
    let evidence_ref = format!(".ai/evidence/{work_item_id}.verification.json");
    let task_report = task_outcome_report(TaskOutcomeReportInput {
        root: &root,
        contract_path: &contract_path,
        contract: &contract,
        summary: Some(&summary),
        snapshot_digest: snapshot_digest(&snapshot).ok(),
        state: OutcomeState::Verified,
        decision_state: DecisionState::Green,
        summary_text: "Verification evidence passed; human-visible benefit remains explicitly unknown unless declared by the Work Item owner.",
        unknowns: &["user_visible_benefit_not_declared".into()],
        evidence_ref: &evidence_ref,
        failed_gate_override: None,
        recovery_condition_override: None,
        historical: false,
    });
    let (task_report_digest, task_report_markdown_digest) =
        write_task_outcome_artifacts(&root, work_item_id, &task_report, retry_recovery_pending)?;
    if retry_recovery_pending {
        summary
            .as_object_mut()
            .expect("Work Item Summary is an object")
            .remove("recoveryRetryPending");
        summary
            .as_object_mut()
            .expect("Work Item Summary is an object")
            .remove("recoveryRetryDecisionPath");
        summary
            .as_object_mut()
            .expect("Work Item Summary is an object")
            .remove("recoveryRetryDecisionDigest");
        if let Err(error) = atomic_json(&summary_path, &summary) {
            // marker の削除に失敗した場合は元の Summary と今回のレポートを戻し、
            // finish_ready と retry marker の矛盾した投影を残さない。
            let _ = atomic_json(&summary_path, &original_summary);
            let _ = fs::remove_file(active.join(format!("{work_item_id}.task-report.json")));
            let _ = fs::remove_file(active.join(format!("{work_item_id}.task-report.md")));
            return Err(error);
        }
    }
    let outcome_v2 = OutcomeV2 {
        schema_version: 2,
        repository_id: contract.repository_id.clone(),
        work_item_id: work_item_id.into(),
        state: OutcomeState::Verified,
        decision_state: Some(DecisionState::Green),
        summary: "Verification evidence passed; human-visible benefit remains explicitly unknown unless declared by the Work Item owner.".into(),
        acceptance_results: contract.acceptance_criteria.clone(),
        unknowns: vec!["user_visible_benefit_not_declared".into()],
        evidence_refs: vec![evidence_ref],
        human_benefit_report: HumanBenefitReport {
            state: OutcomeState::Unknown,
            user_visible_changes: Vec::new(),
            affected_users: Vec::new(),
            unknowns: vec!["user_visible_benefit_not_declared".into()],
            evidence_refs: vec![format!(".ai/evidence/{work_item_id}.verification.json")],
        },
        task_outcome_report: Some(task_report.clone()),
        failed_gate: None,
        recovery_condition: None,
        recovery_decision: None,
        historical_status: None,
    };
    let mut outcome = serde_json::to_value(&outcome_v2).map_err(|error| ObserverError::State {
        path: active.join(format!("{work_item_id}.outcome.json")),
        message: error.to_string(),
    })?;
    outcome["protocolVersion"] = serde_json::json!(1);
    outcome["workItemId"] = serde_json::json!(work_item_id);
    outcome["state"] = serde_json::json!("finish_ready");
    outcome["verification"] = serde_json::json!({
        "status": "verified",
        "required": true,
        "evidencePath": format!(".ai/evidence/{work_item_id}.verification.json"),
    });
    outcome["evidenceDigest"] = cockpit_protocol::digest_json(&evidence)
        .map_err(|error| ObserverError::State {
            path: root.join(".ai/evidence"),
            message: error.to_string(),
        })?
        .to_string()
        .into();
    outcome["taskReportDigest"] = task_report_digest.to_string().into();
    outcome["taskReportMarkdownDigest"] = task_report_markdown_digest.to_string().into();
    outcome["createdAt"] = timestamp.clone().into();
    if let Err(error) = atomic_json(
        &active.join(format!("{work_item_id}.outcome.json")),
        &outcome,
    ) {
        let _ = atomic_json(&summary_path, &original_summary);
        let _ = fs::remove_file(active.join(format!("{work_item_id}.task-report.json")));
        let _ = fs::remove_file(active.join(format!("{work_item_id}.task-report.md")));
        return Err(error);
    }
    if let Err(error) =
        append_task_outcome_events(&root, &contract, &task_report, retry_recovery_pending)
    {
        let _ = fs::remove_file(active.join(format!("{work_item_id}.outcome.json")));
        let _ = fs::remove_file(active.join(format!("{work_item_id}.task-report.json")));
        let _ = fs::remove_file(active.join(format!("{work_item_id}.task-report.md")));
        let _ = atomic_json(&summary_path, &original_summary);
        return Err(error);
    }
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "finish_ready".into(),
        timestamp,
    })
}

pub fn record_verification(
    root: &Path,
    work_item_id: &str,
    receipt: &serde_json::Value,
    runtime_version: &str,
    runtime_digest: &Digest,
) -> Result<serde_json::Value, ObserverError> {
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
    record_verification_with_snapshot(
        &root,
        work_item_id,
        receipt,
        runtime_version,
        runtime_digest,
        &snapshot,
    )
}

const RECOVERY_DECISION_INVALID: &str = "recovery_decision_invalid";

fn recovery_decision_error(
    path: impl Into<PathBuf>,
    code: &str,
    detail: impl std::fmt::Display,
) -> ObserverError {
    ObserverError::State {
        path: path.into(),
        message: format!("{RECOVERY_DECISION_INVALID}:{code}: {detail}"),
    }
}

fn validate_recovery_predecessor_bindings(
    root: &Path,
    work_item_id: &str,
    receipt: &RecoveryDecisionReceipt,
    current_runtime: Option<&RuntimeContext>,
    contract_path: &Path,
    summary_path: &Path,
    candidate_path: Option<&Path>,
) -> Result<(), ObserverError> {
    let decisions = root.join(".ai/decisions");
    if receipt.schema_version != 1
        || receipt.decision_id != "work-item-recovery"
        || !matches!(
            receipt.decision.as_str(),
            "retry" | "successor" | "supersede"
        )
        || receipt.work_item_id != work_item_id
        || receipt.predecessor_work_item_id != work_item_id
        || receipt.repository_id != repository_id(root).to_string()
        || receipt.actor.trim().is_empty()
        || receipt.authority_source.trim().is_empty()
        || receipt.reason.trim().is_empty()
        || receipt.resume_condition.trim().is_empty()
    {
        return Err(recovery_decision_error(
            decisions,
            "identity_mismatch",
            "repository, Work Item, decision, or authority identity is invalid",
        ));
    }
    match receipt.decision.as_str() {
        "retry" if receipt.successor_work_item_id.is_some() => {
            return Err(recovery_decision_error(
                decisions,
                "successor_identity_invalid",
                "retry recovery decision must not include successorWorkItemId",
            ));
        }
        "successor" | "supersede" => {
            let Some(successor_id) = receipt.successor_work_item_id.as_deref() else {
                return Err(recovery_decision_error(
                    decisions,
                    "successor_identity_invalid",
                    "successor recovery decision requires successorWorkItemId",
                ));
            };
            validate_work_item_id(successor_id).map_err(|_| {
                recovery_decision_error(
                    root.join(".ai/work-items"),
                    "successor_identity_invalid",
                    "successor Work Item identity is invalid",
                )
            })?;
            if successor_id == work_item_id {
                return Err(recovery_decision_error(
                    root.join(".ai/work-items"),
                    "successor_identity_invalid",
                    "successor Work Item equals predecessor",
                ));
            }
        }
        _ => {}
    }
    if chrono::DateTime::parse_from_rfc3339(&receipt.decided_at).is_err() {
        return Err(recovery_decision_error(
            decisions,
            "timestamp_invalid",
            "decidedAt must be RFC3339",
        ));
    }
    if let Some(runtime) = current_runtime
        && (receipt.runtime_version != runtime.runtime_version
            || receipt.runtime_digest != runtime.runtime_digest)
    {
        return Err(recovery_decision_error(
            decisions,
            "runtime_mismatch",
            "recovery decision Runtime identity does not match the current Runtime",
        ));
    }

    let expected_contract_digest = contract_digest(contract_path)?;
    if receipt.predecessor_contract_digest != expected_contract_digest {
        return Err(recovery_decision_error(
            contract_path,
            "predecessor_contract_mismatch",
            "predecessor Contract digest mismatch",
        ));
    }
    let summary = read_json(summary_path)?;
    let expected_summary_digest =
        cockpit_protocol::digest_json(&summary).map_err(|error| ObserverError::State {
            path: summary_path.into(),
            message: error.to_string(),
        })?;
    let retry_binding =
        retry_recovery_binding_matches(root, work_item_id, &summary, receipt, candidate_path)?;
    if receipt.predecessor_summary_digest != expected_summary_digest && !retry_binding {
        return Err(recovery_decision_error(
            summary_path,
            "predecessor_summary_mismatch",
            "predecessor Summary digest mismatch",
        ));
    }

    let outcome_path = work_item_artifact_path_optional(root, work_item_id, "outcome.json")?;
    match (&receipt.predecessor_outcome_digest, outcome_path.as_ref()) {
        (Some(expected), Some(path)) => {
            let actual = cockpit_protocol::digest_json(&read_json(path)?).map_err(|error| {
                ObserverError::State {
                    path: path.clone(),
                    message: error.to_string(),
                }
            })?;
            if expected != &actual && !retry_binding {
                return Err(recovery_decision_error(
                    path,
                    "predecessor_outcome_mismatch",
                    "predecessor Outcome digest mismatch",
                ));
            }
        }
        (Some(_), None) | (None, Some(_)) => {
            return Err(recovery_decision_error(
                root.join(".ai/work-items"),
                "predecessor_outcome_presence_mismatch",
                "predecessor Outcome presence does not match the recovery decision",
            ));
        }
        (None, None) => {}
    }

    let events_path = work_item_artifact_path_optional(root, work_item_id, "events.jsonl")?;
    match (&receipt.predecessor_events_digest, events_path.as_ref()) {
        (Some(expected), Some(path)) => {
            let actual =
                Digest::sha256_bytes(&fs::read(path).map_err(|source| ObserverError::Read {
                    path: path.clone(),
                    source,
                })?);
            if expected != &actual && !retry_binding {
                return Err(recovery_decision_error(
                    path,
                    "predecessor_events_mismatch",
                    "predecessor Events digest mismatch",
                ));
            }
        }
        (Some(_), None) | (None, Some(_)) => {
            return Err(recovery_decision_error(
                root.join(".ai/work-items"),
                "predecessor_events_presence_mismatch",
                "predecessor Events presence does not match the recovery decision",
            ));
        }
        (None, None) => {}
    }
    Ok(())
}

fn retry_recovery_binding_matches(
    root: &Path,
    work_item_id: &str,
    summary: &serde_json::Value,
    receipt: &RecoveryDecisionReceipt,
    candidate_path: Option<&Path>,
) -> Result<bool, ObserverError> {
    if receipt.decision != "retry"
        || summary["state"] != serde_json::json!("checkpointed")
        || summary["recoveryRetryPending"] != serde_json::json!(true)
    {
        return Ok(false);
    }
    let Some(candidate_path) = candidate_path else {
        return Ok(false);
    };
    let Some(file_name) = candidate_path.file_name().and_then(|value| value.to_str()) else {
        return Ok(false);
    };
    let canonical = format!("{work_item_id}.recovery.json");
    let versioned_prefix = format!("{work_item_id}.recovery.");
    if file_name != canonical
        && !(file_name.starts_with(&versioned_prefix) && file_name.ends_with(".json"))
    {
        return Ok(false);
    }
    let expected_path = summary["recoveryRetryDecisionPath"].as_str();
    if expected_path != Some(repository_relative_path(root, candidate_path).as_str()) {
        return Ok(false);
    }
    let value = serde_json::to_value(receipt).map_err(|error| ObserverError::State {
        path: root.join(".ai/decisions"),
        message: error.to_string(),
    })?;
    let digest = cockpit_protocol::digest_json(&value).map_err(|error| ObserverError::State {
        path: root.join(".ai/decisions"),
        message: error.to_string(),
    })?;
    Ok(summary["recoveryRetryDecisionDigest"] == serde_json::json!(digest.to_string()))
}

/// Verify that a pending retry marker is backed by the current Runtime-owned
/// recovery receipt before any lifecycle operation consumes the marker.
fn require_current_retry_recovery_binding(
    root: &Path,
    work_item_id: &str,
    summary: &serde_json::Value,
    current_runtime: Option<&RuntimeContext>,
) -> Result<(), ObserverError> {
    if summary["recoveryRetryPending"] != serde_json::json!(true) {
        return Ok(());
    }
    let decision = load_recovery_decision(root, work_item_id, current_runtime)?;
    if decision
        .as_ref()
        .is_some_and(|value| value.decision == "retry")
    {
        return Ok(());
    }
    Err(recovery_decision_error(
        root.join(".ai/decisions"),
        "retry_binding_missing",
        "pending retry marker has no current retry recovery receipt",
    ))
}

fn validate_recovery_successor_binding(
    root: &Path,
    work_item_id: &str,
    receipt: &RecoveryDecisionReceipt,
) -> Result<(), ObserverError> {
    if receipt.decision == "retry" {
        return Ok(());
    }
    let successor_id = receipt
        .successor_work_item_id
        .as_deref()
        .expect("recovery identity validator requires a successor");
    let successor_contract_path =
        work_item_artifact_path_optional(root, successor_id, "contract.json")?.ok_or_else(
            || {
                recovery_decision_error(
                    root.join(".ai/work-items"),
                    "successor_binding_missing",
                    "recovery decision requires an existing successor Contract",
                )
            },
        )?;
    let successor_contract = read_contract(&successor_contract_path).map_err(|error| {
        recovery_decision_error(&successor_contract_path, "successor_binding_invalid", error)
    })?;
    if successor_contract.work_item_id != successor_id
        || successor_contract.repository_id != repository_id(root).to_string()
        || successor_contract.predecessor_work_item_id.as_deref() != Some(work_item_id)
        || successor_contract.predecessor_contract_digest.as_ref()
            != Some(&receipt.predecessor_contract_digest)
    {
        return Err(recovery_decision_error(
            successor_contract_path,
            "successor_binding_mismatch",
            "successor Contract does not bind the predecessor repository, identity, and Contract digest",
        ));
    }
    Ok(())
}

/// Record an immutable, repository-bound retry, successor, or supersession decision. The
/// receipt binds the predecessor's exact Contract/Summary/Outcome/Event
/// digests and the Runtime identity. A second decision is appended under a
/// digest-suffixed filename; no predecessor bytes are replaced.
pub fn record_recovery_decision(
    root: &Path,
    work_item_id: &str,
    receipt: &serde_json::Value,
    runtime: &RuntimeContext,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let contract_path = work_item_artifact_path(&root, work_item_id, "contract.json")?;
    let summary_path = work_item_artifact_path(&root, work_item_id, "summary.json")?;
    let contract = read_contract(&contract_path)?;
    let typed: RecoveryDecisionReceipt =
        serde_json::from_value(receipt.clone()).map_err(|error| ObserverError::State {
            path: root.join(".ai/decisions"),
            message: format!("invalid recovery decision receipt: {error}"),
        })?;
    validate_recovery_predecessor_bindings(
        &root,
        work_item_id,
        &typed,
        Some(runtime),
        &contract_path,
        &summary_path,
        None,
    )?;
    if matches!(typed.decision.as_str(), "successor" | "supersede") {
        let Some(successor_id) = typed.successor_work_item_id.as_deref() else {
            return Err(ObserverError::State {
                path: root.join(".ai/decisions"),
                message: "successor recovery decision requires successorWorkItemId".into(),
            });
        };
        validate_work_item_id(successor_id)?;
        if successor_id == work_item_id {
            return Err(ObserverError::State {
                path: root.join(".ai/work-items"),
                message: "successor Work Item equals predecessor".into(),
            });
        }
        if typed.decision == "successor"
            && work_item_artifact_path_optional(&root, successor_id, "contract.json")?.is_some()
        {
            return Err(ObserverError::State {
                path: root.join(".ai/work-items"),
                message: "successor Work Item already exists".into(),
            });
        }
        if typed.decision == "supersede" {
            validate_recovery_successor_binding(&root, work_item_id, &typed)?;
        }
    }
    let value = serde_json::to_value(&typed).map_err(|error| ObserverError::State {
        path: root.join(".ai/decisions"),
        message: error.to_string(),
    })?;
    let decisions_dir = root.join(".ai/decisions");
    fs::create_dir_all(&decisions_dir).map_err(|source| ObserverError::Read {
        path: decisions_dir.clone(),
        source,
    })?;
    let canonical_path = decisions_dir.join(format!("{work_item_id}.recovery.json"));
    let path = if fs::symlink_metadata(&canonical_path).is_ok() {
        let digest =
            cockpit_protocol::digest_json(&value).map_err(|error| ObserverError::State {
                path: canonical_path.clone(),
                message: error.to_string(),
            })?;
        let digest = digest.to_string();
        decisions_dir.join(format!(
            "{work_item_id}.recovery.{}.json",
            digest.strip_prefix("sha256:").unwrap_or(&digest)
        ))
    } else {
        canonical_path
    };
    if fs::symlink_metadata(&path).is_ok() {
        let existing = read_json(&path)?;
        if existing == value {
            return Ok(existing);
        }
        return Err(ObserverError::State {
            path,
            message: "recovery decision receipt already exists with different content".into(),
        });
    }
    let retry_summary_backup = if typed.decision == "retry" {
        Some(prepare_retryable_lifecycle(&root, work_item_id)?)
    } else {
        None
    };
    if let Err(error) = atomic_json(&path, &value) {
        if let Some((summary_path, original_summary)) = retry_summary_backup {
            let _ = atomic_json(&summary_path, &original_summary);
        }
        return Err(error);
    }
    if typed.decision == "retry" {
        let retry_digest =
            cockpit_protocol::digest_json(&value).map_err(|error| ObserverError::State {
                path: path.clone(),
                message: error.to_string(),
            })?;
        let mut summary = read_json(&summary_path)?;
        summary["recoveryRetryDecisionPath"] =
            serde_json::json!(repository_relative_path(&root, &path));
        summary["recoveryRetryDecisionDigest"] = serde_json::json!(retry_digest.to_string());
        if let Err(error) = atomic_json(&summary_path, &summary) {
            if let Some((summary_path, original_summary)) = retry_summary_backup {
                let _ = atomic_json(&summary_path, &original_summary);
            }
            let _ = fs::remove_file(&path);
            return Err(error);
        }
    }
    if typed.decision == "successor" {
        let successor_id = typed
            .successor_work_item_id
            .as_deref()
            .expect("validated successor decision");
        let mode = contract.mode.as_deref().unwrap_or("implementation");
        scaffold_work_item_for_recovery(&root, successor_id, mode)?;
        let successor_contract_path = root
            .join(".ai/work-items/active")
            .join(format!("{successor_id}.contract.json"));
        let mut successor_contract = read_json(&successor_contract_path)?;
        successor_contract["predecessorWorkItemId"] = serde_json::json!(work_item_id);
        successor_contract["predecessorContractDigest"] =
            serde_json::json!(typed.predecessor_contract_digest.to_string());
        successor_contract["recoveryDecisionPath"] =
            serde_json::json!(repository_relative_path(&root, &path));
        atomic_json(&successor_contract_path, &successor_contract)?;
        let successor_summary_path = root
            .join(".ai/work-items/active")
            .join(format!("{successor_id}.summary.json"));
        let mut successor_summary = read_json(&successor_summary_path)?;
        successor_summary["predecessorWorkItemId"] = serde_json::json!(work_item_id);
        successor_summary["predecessorContractDigest"] =
            serde_json::json!(typed.predecessor_contract_digest.to_string());
        successor_summary["recoveryDecisionPath"] =
            serde_json::json!(repository_relative_path(&root, &path));
        atomic_json(&successor_summary_path, &successor_summary)?;
    }
    Ok(value)
}

/// Restore the only legal retry point after a lifecycle gate has projected a
/// blocked Outcome.  The failed Outcome remains bound by the recovery receipt;
/// a fresh verify/finish cycle will generate the next current projection.
fn prepare_retryable_lifecycle(
    root: &Path,
    work_item_id: &str,
) -> Result<(PathBuf, serde_json::Value), ObserverError> {
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let mut summary = read_json(&summary_path)?;
    let state = summary["state"].as_str().unwrap_or_default();
    if state == "checkpointed" {
        let original = summary.clone();
        // A failed finish may already have appended a before_finish record
        // against the snapshot that was current at that attempt. A retry is
        // the explicit recovery boundary for a fresh verification cycle, so
        // discard that stale terminal candidate and let finish append a new
        // current before_finish record. The predecessor Summary digest in the
        // recovery receipt preserves the old attempt immutably.
        if let Some(entries) = summary
            .get_mut("checkpointEvidence")
            .and_then(serde_json::Value::as_array_mut)
        {
            entries.retain(|entry| {
                entry.get("stage").and_then(serde_json::Value::as_str) != Some("before_finish")
            });
        }
        summary["recoveryRetryPending"] = serde_json::json!(true);
        atomic_json(&summary_path, &summary)?;
        return Ok((summary_path, original));
    }
    if state != "finish_ready" {
        return Err(ObserverError::State {
            path: summary_path,
            message: format!("retry recovery requires finish_ready state, got {state}"),
        });
    }
    let preflight_state = summary["preflightState"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let lifecycle_retry = summary["failedGate"].as_str() == Some("finish.lifecycle");
    if summary["checkpointCount"] != serde_json::json!(1)
        || (!matches!(preflight_state.as_str(), "green" | "yellow") && !lifecycle_retry)
    {
        return Err(ObserverError::State {
            path: summary_path,
            message: "retry recovery requires one checkpoint and either a non-red preflight result or a failed finish.lifecycle transition".into(),
        });
    }
    let original = summary.clone();
    if let Some(entries) = summary
        .get_mut("checkpointEvidence")
        .and_then(serde_json::Value::as_array_mut)
    {
        entries.retain(|entry| {
            entry.get("stage").and_then(serde_json::Value::as_str) != Some("before_finish")
        });
    }
    summary["state"] = serde_json::json!("checkpointed");
    summary["updatedAt"] = serde_json::json!(now());
    if let Some(object) = summary.as_object_mut() {
        object.remove("failedGate");
        object.remove("recoveryCondition");
        object.remove("outcomeState");
    }
    summary["recoveryRetryPending"] = serde_json::json!(true);
    atomic_json(&summary_path, &summary)?;
    Ok((summary_path, original))
}

fn work_item_artifact_path(
    root: &Path,
    work_item_id: &str,
    suffix: &str,
) -> Result<PathBuf, ObserverError> {
    work_item_artifact_path_optional(root, work_item_id, suffix)?.ok_or_else(|| {
        ObserverError::State {
            path: root.join(".ai/work-items"),
            message: format!("Work Item artifact not found: {work_item_id}.{suffix}"),
        }
    })
}

fn work_item_artifact_path_optional(
    root: &Path,
    work_item_id: &str,
    suffix: &str,
) -> Result<Option<PathBuf>, ObserverError> {
    for phase in ["active", "archive"] {
        let path = root
            .join(".ai/work-items")
            .join(phase)
            .join(format!("{work_item_id}.{suffix}"));
        if fs::symlink_metadata(&path).is_ok() {
            if !is_regular_non_symlink(&path)? {
                return Err(ObserverError::State {
                    path,
                    message: "Work Item artifact must be a regular non-symlink file".into(),
                });
            }
            return Ok(Some(path));
        }
    }
    Ok(None)
}

pub fn record_verification_with_snapshot(
    root: &Path,
    work_item_id: &str,
    receipt: &serde_json::Value,
    runtime_version: &str,
    runtime_digest: &Digest,
    snapshot: &RepositorySnapshot,
) -> Result<serde_json::Value, ObserverError> {
    record_verification_internal(
        root,
        work_item_id,
        receipt,
        runtime_version,
        runtime_digest,
        snapshot,
        None,
    )
}

/// Record verification evidence while binding it to the Runtime that is
/// executing the request.  The legacy `*_with_snapshot` API remains available
/// for embedders that intentionally own their Runtime identity; all CLI/MCP
/// paths use this function so a foreign but well-formed digest cannot pass a
/// current lifecycle operation.
pub fn record_verification_with_runtime(
    root: &Path,
    work_item_id: &str,
    receipt: &serde_json::Value,
    runtime: &RuntimeContext,
    snapshot: &RepositorySnapshot,
) -> Result<serde_json::Value, ObserverError> {
    record_verification_internal(
        root,
        work_item_id,
        receipt,
        &runtime.runtime_version,
        &runtime.runtime_digest,
        snapshot,
        Some(runtime),
    )
}

fn record_verification_internal(
    root: &Path,
    work_item_id: &str,
    receipt: &serde_json::Value,
    runtime_version: &str,
    runtime_digest: &Digest,
    snapshot: &RepositorySnapshot,
    current_runtime: Option<&RuntimeContext>,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    if fs::canonicalize(&snapshot.root).ok().as_ref() != Some(&root) {
        return Err(ObserverError::SnapshotRootMismatch);
    }
    let active_contract = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    if !active_contract.is_file() {
        return Err(ObserverError::State {
            path: active_contract,
            message: "verification evidence requires an active work item contract".into(),
        });
    }
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let summary: serde_json::Value = read_json(&summary_path)?;
    if !matches!(
        summary["state"].as_str(),
        Some("checkpointed" | "finish_ready")
    ) || summary["checkpointCount"] != serde_json::json!(1)
    {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "verification requires exactly one completed checkpoint and an active lifecycle state".into(),
        });
    }
    let recovery_retry_pending = summary["recoveryRetryPending"] == serde_json::json!(true);
    let contract_amendment_pending = summary
        .get("verificationInvalidatedByContractAmendment")
        .is_some();
    require_current_retry_recovery_binding(&root, work_item_id, &summary, current_runtime)?;
    if !matches!(summary["preflightState"].as_str(), Some("green" | "yellow"))
        && !recovery_retry_pending
        && !contract_amendment_pending
    {
        return Err(ObserverError::State {
            path: summary_path,
            message: "verification requires a recorded non-red preflight result unless an explicit recovery retry is pending".into(),
        });
    }
    if receipt["passed"] != serde_json::Value::Bool(true) {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence"),
            message: "failed verification cannot be recorded as completion evidence".into(),
        });
    }
    if receipt
        .get("workItemId")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|receipt_id| receipt_id != work_item_id)
    {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence"),
            message: "verification receipt belongs to another work item".into(),
        });
    }
    if let Some(runtime) = current_runtime
        && (runtime.runtime_version != runtime_version || runtime.runtime_digest != *runtime_digest)
    {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence"),
            message:
                "verification receipt Runtime identity arguments do not match the current Runtime"
                    .into(),
        });
    }
    let expected_repository_id = repository_id(&root).to_string();
    let typed_receipt = bind_typed_verification_receipt(
        receipt,
        work_item_id,
        &expected_repository_id,
        runtime_version,
        runtime_digest,
    )?;
    if current_runtime.is_some() && typed_receipt.is_none() {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence"),
            message: "current Runtime requires a strict typed verification receipt".into(),
        });
    }
    let current_contract_digest = contract_digest(&active_contract)?;
    let retention_policy = read_evidence_retention_policy(&root, work_item_id)?;
    let (stored_receipt, capture_mode) = match retention_policy
        .as_ref()
        .map(|policy| &policy.retention.persistence)
    {
        Some(EvidencePersistence::NoPersistence) => {
            return Err(ObserverError::State {
                path: root
                    .join(".ai/evidence")
                    .join(format!("{work_item_id}.retention.json")),
                message:
                    "no_persistence cannot produce completion evidence; use an external evidence owner or change the policy".into(),
            });
        }
        Some(EvidencePersistence::DigestOnly) => (None, VerificationCaptureMode::DigestOnly),
        Some(EvidencePersistence::RedactedCapture) => (
            Some(redact_verification_receipt(
                typed_receipt.as_ref().unwrap_or(receipt),
            )),
            VerificationCaptureMode::RedactedCapture,
        ),
        Some(EvidencePersistence::FullCapture) | None => (
            Some(typed_receipt.clone().unwrap_or_else(|| receipt.clone())),
            if typed_receipt.is_some() {
                VerificationCaptureMode::FullCapture
            } else {
                VerificationCaptureMode::LegacyUntyped
            },
        ),
    };
    let receipt_digest = cockpit_protocol::digest_json(stored_receipt.as_ref().unwrap_or(receipt))
        .map_err(|error| ObserverError::State {
            path: root.join(".ai/evidence"),
            message: error.to_string(),
        })?;
    let mut evidence = serde_json::json!({
        "protocolVersion": 1,
        "evidenceSchemaVersion": 2,
        "workItemId": work_item_id,
        "repositoryId": expected_repository_id,
        "runtimeVersion": runtime_version,
        "runtimeDigest": runtime_digest,
        "contractDigest": current_contract_digest,
        "repositorySnapshotDigest": snapshot_digest(snapshot)?,
        "passed": true,
        "receiptDigest": receipt_digest,
        "captureMode": serde_json::to_value(capture_mode).expect("capture mode serializes"),
        "createdAt": now(),
    });
    if let Some(receipt) = stored_receipt {
        evidence["receipt"] = receipt;
    }
    if let Some(policy) = retention_policy {
        evidence["retention"] =
            serde_json::to_value(policy).map_err(|error| ObserverError::State {
                path: root.join(".ai/evidence"),
                message: error.to_string(),
            })?;
    }
    let path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    atomic_json(&path, &evidence)?;

    // Verification can satisfy a Contract's required evidence.  Refresh the
    // recorded governance result against the same non-.ai snapshot so the
    // canonical lifecycle remains start -> preflight (possibly yellow) ->
    // checkpoint -> verify -> finish, without requiring an otherwise
    // redundant second CLI preflight invocation.
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let refreshed_snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let raw_decision = governance_decision_for_contract_base_internal_with_archive(
        &root,
        &contract,
        &refreshed_snapshot,
        None,
        false,
    )?;
    let decision = apply_preflight_review_evidence(
        &root,
        &contract,
        &refreshed_snapshot,
        raw_decision.clone(),
        false,
    )?;
    let decision_value =
        serde_json::to_value(&raw_decision).map_err(|error| ObserverError::State {
            path: contract_path.clone(),
            message: error.to_string(),
        })?;
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let mut summary: serde_json::Value = read_json(&summary_path)?;
    summary
        .as_object_mut()
        .expect("Work Item Summary is an object")
        .remove("verificationInvalidatedByContractAmendment");
    let verification_entries = summary
        .as_object_mut()
        .expect("Work Item Summary is an object")
        .entry("verification")
        .or_insert_with(|| serde_json::json!([]));
    let verification_entries =
        verification_entries
            .as_array_mut()
            .ok_or_else(|| ObserverError::State {
                path: summary_path.clone(),
                message: "Summary.verification must be an array".into(),
            })?;
    if let Some(results) = receipt.get("results").and_then(serde_json::Value::as_array) {
        for result in results {
            let Some(node_id) = result.get("nodeId").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let value = serde_json::json!({
                "check": node_id,
                "result": if result.get("passed") == Some(&serde_json::Value::Bool(true)) {
                    "passed"
                } else {
                    "failed"
                },
            });
            verification_entries.retain(|item| {
                item.get("check").and_then(serde_json::Value::as_str) != Some(node_id)
            });
            verification_entries.push(value);
        }
    }
    summary["preflightState"] = decision_state_name(decision.state.clone()).into();
    summary["preflightDecisionDigest"] = cockpit_protocol::digest_json(&decision_value)
        .map_err(|error| ObserverError::State {
            path: contract_path.clone(),
            message: error.to_string(),
        })?
        .to_string()
        .into();
    summary["preflightRepositorySnapshotDigest"] =
        snapshot_digest(&refreshed_snapshot)?.to_string().into();
    summary["preflightContractDigest"] = contract_digest(&contract_path)?.to_string().into();
    // A fresh verification after Contract amendment revalidates the existing
    // checkpoint against the new Contract and snapshot. The immutable
    // before_edit evidence remains in the append-only chain; only this
    // current lifecycle binding advances.
    summary["checkpointContractDigest"] = contract_digest(&contract_path)?.to_string().into();
    summary["checkpointRepositorySnapshotDigest"] =
        snapshot_digest(&refreshed_snapshot)?.to_string().into();
    summary["preflightAt"] = now().into();
    atomic_json(&summary_path, &summary)?;
    if summary["state"] == serde_json::json!("finish_ready") {
        refresh_active_outcome_verification_binding(
            &root,
            work_item_id,
            &evidence,
            &snapshot_digest(&refreshed_snapshot)?,
        )?;
    }
    Ok(evidence)
}

/// Refresh the active human Outcome after a verification retry that occurs
/// while the Work Item is already finish-ready.  A hosted PR gate may observe
/// the normal governance-only commits made after the first verification; the
/// retry must update every active Outcome/report binding instead of leaving an
/// old evidence digest that would make archive/close fail later.
fn refresh_active_outcome_verification_binding(
    root: &Path,
    work_item_id: &str,
    evidence: &serde_json::Value,
    current_snapshot_digest: &Digest,
) -> Result<(), ObserverError> {
    let active = root.join(".ai/work-items/active");
    let outcome_path = active.join(format!("{work_item_id}.outcome.json"));
    if !is_regular_non_symlink(&outcome_path)? {
        return Err(ObserverError::State {
            path: outcome_path,
            message: "finish-ready verification retry requires a regular Outcome".into(),
        });
    }
    let mut outcome = read_json(&outcome_path)?;
    if outcome["state"] != serde_json::json!("finish_ready")
        || outcome["verification"]["status"] != serde_json::json!("verified")
    {
        return Err(ObserverError::State {
            path: outcome_path,
            message: "finish-ready verification retry requires a verified Outcome".into(),
        });
    }
    outcome["evidenceDigest"] = cockpit_protocol::digest_json(evidence)
        .map_err(|error| ObserverError::State {
            path: root.join(".ai/evidence"),
            message: error.to_string(),
        })?
        .to_string()
        .into();

    let report_path = active.join(format!("{work_item_id}.task-report.json"));
    if is_regular_non_symlink(&report_path)? {
        let mut report: TaskOutcomeReport = serde_json::from_value(read_json(&report_path)?)
            .map_err(|error| ObserverError::State {
                path: report_path.clone(),
                message: format!("invalid active Task Outcome report: {error}"),
            })?;
        if report.work_item_id != work_item_id {
            return Err(ObserverError::State {
                path: report_path,
                message: "active Task Outcome report belongs to another Work Item".into(),
            });
        }
        report.bindings.repository_snapshot_digest = Some(current_snapshot_digest.clone());
        let report_value = serde_json::to_value(&report).map_err(|error| ObserverError::State {
            path: report_path.clone(),
            message: error.to_string(),
        })?;
        let report_bytes =
            serde_json::to_vec_pretty(&report_value).map_err(|error| ObserverError::State {
                path: report_path.clone(),
                message: error.to_string(),
            })?;
        atomic_write(&report_path, &report_bytes)?;
        let markdown_path = active.join(format!("{work_item_id}.task-report.md"));
        if is_regular_non_symlink(&markdown_path)? {
            atomic_write(&markdown_path, task_outcome_markdown(&report).as_bytes())?;
            outcome["taskReportMarkdownDigest"] =
                Digest::sha256_bytes(task_outcome_markdown(&report).as_bytes())
                    .to_string()
                    .into();
        }
        outcome["taskOutcomeReport"] = report_value;
        outcome["taskReportDigest"] = Digest::sha256_bytes(&report_bytes).to_string().into();
    }
    atomic_json(&outcome_path, &outcome)
}

/// Bind a raw execution result to its Work Item/repository/Runtime identity
/// and deserialize it through the strict wire type.  The CLI's raw result has
/// runtime fields at the envelope level; they are removed before adding the
/// required nested identity fields to the persisted receipt.
fn bind_typed_verification_receipt(
    receipt: &serde_json::Value,
    work_item_id: &str,
    repository_id: &str,
    runtime_version: &str,
    runtime_digest: &Digest,
) -> Result<Option<serde_json::Value>, ObserverError> {
    let Some(object) = receipt.as_object() else {
        return Ok(None);
    };
    if object.get("passed") != Some(&serde_json::Value::Bool(true)) {
        return Err(ObserverError::State {
            path: PathBuf::from(".ai/evidence"),
            message: "failed verification cannot be recorded as completion evidence".into(),
        });
    }
    let mut bound = receipt.clone();
    let Some(bound_object) = bound.as_object_mut() else {
        return Ok(None);
    };
    bound_object.remove("runtimeVersion");
    bound_object.remove("runtimeDigest");
    for (key, expected) in [
        ("workItemId", serde_json::Value::String(work_item_id.into())),
        (
            "repositoryId",
            serde_json::Value::String(repository_id.into()),
        ),
        (
            "runtimeVersion",
            serde_json::Value::String(runtime_version.into()),
        ),
        (
            "runtimeDigest",
            serde_json::Value::String(runtime_digest.to_string()),
        ),
    ] {
        if let Some(existing) = bound_object.get(key)
            && existing != &expected
        {
            return Err(ObserverError::State {
                path: PathBuf::from(".ai/evidence"),
                message: format!("verification receipt {key} does not match its binding"),
            });
        }
        bound_object.insert(key.into(), expected);
    }
    let typed: cockpit_verification::VerificationReceipt =
        match serde_json::from_value(bound.clone()) {
            Ok(value) => value,
            Err(_) => return Ok(None),
        };
    if !typed.passed
        || typed.work_item_id.as_deref() != Some(work_item_id)
        || typed.repository_id.as_deref() != Some(repository_id)
        || typed.runtime_version.as_deref() != Some(runtime_version)
        || typed.runtime_digest.as_deref() != Some(runtime_digest.to_string().as_str())
    {
        return Err(ObserverError::State {
            path: PathBuf::from(".ai/evidence"),
            message: "typed verification receipt has missing or mismatched identity".into(),
        });
    }
    Ok(Some(bound))
}

fn redact_verification_receipt(receipt: &serde_json::Value) -> serde_json::Value {
    match receipt {
        serde_json::Value::Object(map) => {
            let mut redacted = serde_json::Map::new();
            for (key, value) in map {
                let lowered = key.to_ascii_lowercase();
                if matches!(lowered.as_str(), "output" | "stdout" | "stderr" | "command")
                    || lowered.contains("log")
                {
                    redacted.insert(key.clone(), serde_json::Value::String("[redacted]".into()));
                } else {
                    redacted.insert(key.clone(), redact_verification_receipt(value));
                }
            }
            serde_json::Value::Object(redacted)
        }
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.iter().map(redact_verification_receipt).collect())
        }
        other => other.clone(),
    }
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
    let mut hasher = Sha256::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let Some((_, path)) = line.split_once('\t') else {
            continue;
        };
        if path == ".ai" || path.starts_with(".ai/") {
            continue;
        }
        hasher.update(path.as_bytes());
        hasher.update([0]);
        hasher.update(line.as_bytes());
        hasher.update([0]);
    }
    Ok(Digest::sha256_bytes(&hasher.finalize()))
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
    stable.tree_digest = source_tree_digest(snapshot)?.to_string();
    stable
        .changed_paths
        .retain(|path| !path.starts_with(".ai/"));
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
            if test_path {
                result
                    .unknowns
                    .push("test_weakening_inspection_unavailable".into());
            }
            if coverage_path {
                result
                    .unknowns
                    .push("coverage_weakening_inspection_unavailable".into());
            }
            if is_textual_material_path(&change.path) {
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
    if contract.repository_id != repository_id(&root).to_string() {
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
    )?;
    apply_preflight_review_evidence(root, contract, snapshot, decision, archived)
}

fn governance_decision_for_contract_base_internal_with_archive(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    current_runtime: Option<&RuntimeContext>,
    archived: bool,
) -> Result<GovernanceDecision, ObserverError> {
    let explicit_blockers = contract_freshness_findings(root, contract, snapshot)?;
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
    explicit_unknowns.extend(project_governance_unknowns(root, contract, snapshot)?);
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
    if let Some(contract_digest) = envelope.contract_digest.as_ref()
        && contract_digest != &contract_digest_for_evidence(root, contract)?
    {
        return Ok(EvidenceState::Contradictory);
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
    let ai = root.join(".ai");
    let active = ai.join("work-items/active");
    let archive = ai.join("work-items/archive");
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
    let mut artifacts = vec![
        ("contract", "contract.json"),
        ("summary", "summary.json"),
        ("outcome", "outcome.json"),
    ];
    let events_source = task_outcome_event_path(&root, work_item_id, false);
    if optional_regular_artifact(&events_source, "Task Outcome event stream")? {
        artifacts.push(("events", "events.jsonl"));
    }
    let report_source = active.join(format!("{work_item_id}.task-report.json"));
    if optional_regular_artifact(&report_source, "Task Outcome report")? {
        artifacts.push(("taskReport", "task-report.json"));
    }
    let markdown_source = active.join(format!("{work_item_id}.task-report.md"));
    if optional_regular_artifact(&markdown_source, "Task Outcome Markdown report")? {
        artifacts.push(("taskReportMarkdown", "task-report.md"));
    }
    let approach_source = active.join(format!("{work_item_id}.approach.json"));
    if optional_regular_artifact(&approach_source, "Implementation approach")? {
        artifacts.push(("approach", "approach.json"));
    }
    let intelligence_source = active.join(format!("{work_item_id}.intelligence.json"));
    if optional_regular_artifact(&intelligence_source, "Work Item intelligence sidecar")? {
        artifacts.push(("intelligence", "intelligence.json"));
    }
    let mut pending = Vec::new();
    for (name, suffix) in artifacts {
        let source_path = active.join(format!("{work_item_id}.{suffix}"));
        let target = archive.join(format!("{work_item_id}.{suffix}"));
        let source_bytes = fs::read(&source_path).map_err(|error| ObserverError::Read {
            path: source_path.clone(),
            source: error,
        })?;
        let archived_bytes =
            normalized_archive_artifact_bytes(suffix, &source_bytes, work_item_id)?;
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
    for (name, suffix, _, _, _, archived_bytes) in &pending {
        files.insert(
            format!("{name}Path"),
            serde_json::Value::String(format!(".ai/work-items/archive/{work_item_id}.{suffix}")),
        );
        files.insert(
            format!("{name}Digest"),
            serde_json::Value::String(Digest::sha256_bytes(archived_bytes).to_string()),
        );
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
    let candidates = [
        ("contract", "contract.json"),
        ("summary", "summary.json"),
        ("outcome", "outcome.json"),
        ("approach", "approach.json"),
        ("intelligence", "intelligence.json"),
        ("events", "events.jsonl"),
        ("taskReport", "task-report.json"),
        ("taskReportMarkdown", "task-report.md"),
    ];
    let mut files = serde_json::Map::new();
    let mut pending = Vec::new();
    for (name, suffix) in candidates {
        let source = active.join(format!("{work_item_id}.{suffix}"));
        if !optional_regular_artifact(&source, name)? {
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
        Some("implementation_active" | "checkpointed" | "finish_ready")
    ) {
        return Err(ObserverError::State {
            path: summary_path,
            message: "finalize-plan requires an active, checkpointed, or finish_ready Work Item"
                .into(),
        });
    }
    if let Some(existing) = &contract.resource_context
        && existing != context
        && existing.provider != "unknown"
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
    let canonical = resource_finalization_decision_path(root, work_item_id);
    let mut receipt = read_resource_finalization_receipt(&canonical)?;
    let mut path = canonical;
    let mut digest =
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
    let prefix = format!("{work_item_id}.finalize.");
    let canonical_name = format!("{work_item_id}.finalize.json");
    let mut candidates = fs::read_dir(root.join(".ai/decisions"))
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

fn ensure_resource_finalization_base_binding(
    receipt: &ResourceFinalizationReceipt,
    contract: &Contract,
    path: &Path,
) -> Result<(), ObserverError> {
    if receipt.pull_request.base_revision != contract.base_revision {
        return Err(ObserverError::State {
            path: path.into(),
            message: "resource finalization pull request base revision does not match the archived Contract base revision".into(),
        });
    }
    Ok(())
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
        ResourceFinalizationDisposition::Deleted
    ) && !local_resources_deleted(&root, &receipt)?
    {
        return Err(ObserverError::State {
            path: receipt_path.into(),
            message:
                "deleted finalization receipt does not match local branch/worktree postconditions"
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
        resolve_resource_finalization_head(&root, work_item_id)?;
    let (contract, contract_digest) = archived_contract_digest(&root, work_item_id)?;
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
        Some(&contract_digest),
        contract.resource_context.as_ref(),
    )
    .map_err(|error| ObserverError::State {
        path: path.clone(),
        message: error.to_string(),
    })?;
    ensure_resource_finalization_base_binding(&receipt, &contract, &path)?;
    if let Some(runtime) = runtime {
        ensure_resource_runtime_identity(&receipt, runtime, &path)?;
    }
    if matches!(
        receipt.result.disposition,
        ResourceFinalizationDisposition::Deleted
    ) && !local_resources_deleted(&root, &receipt)?
    {
        return Err(ObserverError::State {
            path,
            message: "resource finalization postconditions are not satisfied".into(),
        });
    }
    Ok(serde_json::json!({
        "workItemId": work_item_id,
        "state": "verified",
        "disposition": receipt.result.disposition,
        "sequence": sequence,
        "headPath": repository_relative_path(&root, &path),
        "headDigest": receipt_digest,
        "receipt": receipt
    }))
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
    let archive = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.archive.json"));
    let manifest = read_json(&archive)?;
    verify_archive_manifest(&root, work_item_id, &manifest)?;
    // An archived predecessor may have a valid, append-only supersede
    // recovery decision recorded after the original archive.  The recovery
    // decision is the explicit authority for this transition; the immutable
    // archive manifest remains `archived` and is never rewritten merely to
    // make close succeed.
    let manifest_superseded = manifest["state"] == serde_json::json!("superseded");
    let recovery_decision = load_recovery_decision(&root, work_item_id, current_runtime)?;
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
    if !superseded {
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
    if disposition != "deleted" {
        return Err(ObserverError::State {
            path: resource_finalization_decision_path(root, work_item_id),
            message: format!(
                "close requires resource finalization disposition deleted; retained resources require cleanup before close, got {disposition}"
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
    Ok(())
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
        let cached = read_json(&index_path)?;
        if let Ok(index) = serde_json::from_value::<cockpit_knowledge::KnowledgeIndex>(cached)
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

/// Build a request-scoped, provenance-aware implementation approach.  Facts
/// are copied from the current Observer snapshot and derivations name the
/// exact fact keys they consume.  Empty human-owned contract fields remain
/// unknown rather than being guessed from prose or filenames.
pub fn implementation_approach(
    root: &Path,
    work_item_id: &str,
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
    atomic_json(
        &root
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.approach.json")),
        &serde_json::to_value(&approach).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?,
    )?;
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
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let snapshot_digest = snapshot_digest(&snapshot)?;
    let outcome = outcome_v2_with_runtime(&root, work_item_id, runtime)?;
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
    source_digests.insert("repositorySnapshot".into(), snapshot_digest.clone());
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
        "snapshotDigest": snapshot_digest,
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
        snapshot_digest,
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
        let entry = match work_item_status_snapshot_with_runtime(&root, work_item_id, runtime) {
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
/// The helper is deliberately best-effort: the original gate error remains
/// authoritative, while any persisted projection is identity-bound and never
/// changes the lifecycle state to a terminal success.
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
    let text = error.to_string().to_ascii_lowercase();
    if text.contains("verification") {
        (
            "finish.verification".into(),
            "Record valid current verification evidence, rerun preflight, and retry finish.".into(),
        )
    } else if text.contains("preflight") {
        (
            "finish.preflight".into(),
            "Record a fresh non-red preflight result, then retry finish.".into(),
        )
    } else if text.contains("contract") || text.contains("governance") {
        (
            "finish.governance".into(),
            "Repair the Contract or governance projection, rerun preflight, and retry finish."
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
        if event.schema_version != 1
            || event.repository_id != expected_repository_id
            || event.work_item_id != expected_work_item_id
            || event.event_id.trim().is_empty()
            || event.event_type.trim().is_empty()
            || event.timestamp.trim().is_empty()
            || event.detail.trim().is_empty()
            || !event_detail_is_safe(&event.detail)
            || !ids.insert(event.event_id.clone())
            || event.related_event_ids.iter().any(|id| !ids.contains(id))
            || event
                .correction_of
                .as_ref()
                .is_some_and(|id| !ids.contains(id))
            || !matches!(
                event.event_type.as_str(),
                "blocked" | "completed" | "warning" | "stop" | "resolution" | "recovered"
            )
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
    for claim in &report.sections.warnings {
        append("warning", &claim.text, claim.evidence_refs.clone());
    }
    for claim in &report.sections.forced_stops {
        append("stop", &claim.text, claim.evidence_refs.clone());
    }
    for claim in &report.sections.resolutions {
        append("resolution", &claim.text, claim.evidence_refs.clone());
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
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
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
        && verification_evidence_state(&root, &contract, &snapshot, true, None)?
            == EvidenceState::Complete
        && verification_evidence_state(&root, &contract, &snapshot, true, current_runtime)?
            != EvidenceState::Complete;
    let historical = legacy || historical_runtime;
    let evidence_state = if historical {
        None
    } else {
        Some(verification_evidence_state(
            &root,
            &contract,
            &snapshot,
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
            } else {
                "Historical verification evidence was produced by an older Runtime and is not revalidated as a current result."
            },
            Some(if legacy {
                "legacy_evidence_historical"
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
    let persisted_failure = if !archived {
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
    let (recovery_decision, recovery_decision_invalid) =
        match load_recovery_decision(&root, work_item_id, current_runtime) {
            Ok(decision) => (decision, false),
            Err(_) => (None, true),
        };
    if recovery_decision_invalid {
        state = OutcomeState::Unknown;
        decision_state = DecisionState::Red;
        summary = "Recovery decision evidence is malformed, foreign, stale, or not bound to the current predecessor; outcome is stopped.";
        evidence_unknown = Some(RECOVERY_DECISION_INVALID);
    }
    let historical_status = if historical {
        Some(if legacy {
            "legacy".to_owned()
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
        snapshot_digest: snapshot_digest(&snapshot).ok(),
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
    validate_recovery_successor_binding(root, work_item_id, &receipt)?;
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
                    if !is_stale_recovery_binding_error(&error) {
                        return Err(error);
                    }
                    let retry = read_json(&path).ok().and_then(|value| {
                        serde_json::from_value::<RecoveryDecisionReceipt>(value).ok()
                    });
                    let Some(retry) = retry.filter(|receipt| receipt.decision == "retry") else {
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
    if let Some(work_item_id) = work_item_id {
        let path = root
            .join(".ai/evidence")
            .join(format!("{work_item_id}.verification.json"));
        match read_json(&path) {
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
            Err(_) => unknowns.push("verification_receipt_missing".into()),
        }
    } else {
        unknowns.push("work_item_not_selected".into());
    }
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
        repository_id: repository_id(&root).to_string(),
        work_item_id: work_item_id.map(str::to_owned),
        state,
        cost,
        bottlenecks,
        unknowns,
        evidence_refs,
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

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ObserverError> {
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
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
    output.sort();
    Ok(())
}
