use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use chrono::{DateTime, Utc};
use cockpit_core::{
    ActionKind, AuthorityState, DecisionState, Digest, EvidenceState, GovernanceDecision,
    GovernanceInput, evaluate,
};
use cockpit_git::{ChangeContentState, ChangeKind, RepositorySnapshot};
use cockpit_protocol::{
    AgentAdapterCompatibility, AgentInterfaceAvailability, AgentInterfaceManifest, AgentInterfaces,
    AgentRootBinding, ApprovalMode, AuditEvent, AuditExportManifest, CapabilityConfidence,
    CapabilityTruth, CapabilityTruthRegistry, Contract, DataClassification, DelegatedEvidence,
    DelegatedEvidenceReceipt, DiagnosisState, EvidenceDisposition, EvidenceDispositionItem,
    EvidencePersistence, EvidenceRetention, EvidenceRetentionPolicy, EvidenceValidity, FactOrigin,
    GovernanceCost, GovernancePolicy, GovernancePolicyDocument, HumanBenefitReport, HumanDecision,
    ImplementationApproach, OutcomeState, OutcomeV2, PerformanceDiagnosis, PolicyLayer,
    QualityCommand, RepositoryConfig, RuntimeContext, SchemaMigrationStep, TruthState,
    WorkItemCompatibility, WorkItemIntelligence, default_repository_schema_version,
    merge_policy_layers, repository_schema_migration_chain, validate_evidence_retention,
    validate_protocol_version,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::BTreeMap;
#[cfg(unix)]
use std::collections::BTreeSet;
#[cfg(unix)]
use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;

mod outcome_render;

pub use outcome_render::render_human_outcome;

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
pub struct RepositoryStatus {
    pub protocol_version: u32,
    pub repository_schema_version: u32,
    pub repository_id: String,
    pub state: String,
    pub profile_version: u64,
    pub active_work_items: usize,
    pub archived_work_items: usize,
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
    if input.program.is_empty()
        || input.scope.is_empty()
        || !matches!(input.stage.as_str(), "task" | "pr" | "release")
        || !matches!(input.runner.as_str(), "local" | "hosted")
    {
        return Ok(denied_reuse("verification_context_invalid"));
    }
    let base_commit = if input.stage == "task" {
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
    let base_commit = if input.stage == "task" {
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
    Ok(RepositoryStatus {
        protocol_version: config.protocol_version,
        repository_schema_version: config.repository_schema_version,
        repository_id: profile.repository_id,
        state: profile.state,
        profile_version: profile.profile_version,
        active_work_items: count_suffix(&ai.join("work-items/active"), ".contract.json"),
        archived_work_items: count_suffix(&ai.join("work-items/archive"), ".archive.json"),
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
        "baseRevision": facts.base_revision,
        "projectProfileDigest": facts.project_profile_digest,
        "repositorySnapshotDigest": facts.repository_snapshot_digest,
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
    let timestamp = now();
    summary["checkpointCount"] = 1.into();
    summary["state"] = "checkpointed".into();
    summary["checkpointAt"] = timestamp.clone().into();
    summary["updatedAt"] = timestamp.clone().into();
    atomic_json(&path, &summary)?;
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "checkpointed".into(),
        timestamp,
    })
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
    let decision =
        governance_decision_for_contract_internal(&root, &contract, &snapshot, current_runtime)?;

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
        let current_state = summary["state"].as_str().unwrap_or("");
        // A scaffold is intentionally not an active lifecycle item yet.  Keep
        // the historical read-only preflight behavior for this state so
        // callers can inspect the candidate decision before `start` supplies
        // the human governance fields and activates the item.
        if current_state == "not_ready" {
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
            serde_json::to_value(&decision).map_err(|error| ObserverError::State {
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
    Ok(())
}

pub fn finish_work_item(
    root: &Path,
    work_item_id: &str,
) -> Result<LifecycleReceipt, ObserverError> {
    finish_work_item_internal(root, work_item_id, None)
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
    finish_work_item_internal(root, work_item_id, Some(runtime))
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
    let contract_path = active.join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    if verification_evidence_state(&root, &contract, &snapshot, false, current_runtime)?
        != EvidenceState::Complete
    {
        return Err(ObserverError::State {
            path: evidence_path,
            message: "verification evidence is not a valid current receipt".into(),
        });
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
    let outcome_v2 = OutcomeV2 {
        schema_version: 2,
        repository_id: contract.repository_id.clone(),
        work_item_id: work_item_id.into(),
        state: OutcomeState::Verified,
        decision_state: Some(DecisionState::Green),
        summary: "Verification evidence passed; human-visible benefit remains explicitly unknown unless declared by the Work Item owner.".into(),
        acceptance_results: contract.acceptance_criteria.clone(),
        unknowns: vec!["user_visible_benefit_not_declared".into()],
        evidence_refs: vec![format!(".ai/evidence/{work_item_id}.verification.json")],
        human_benefit_report: HumanBenefitReport {
            state: OutcomeState::Unknown,
            user_visible_changes: Vec::new(),
            affected_users: Vec::new(),
            unknowns: vec!["user_visible_benefit_not_declared".into()],
            evidence_refs: vec![format!(".ai/evidence/{work_item_id}.verification.json")],
        },
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
    outcome["createdAt"] = timestamp.clone().into();
    if let Err(error) = atomic_json(
        &active.join(format!("{work_item_id}.outcome.json")),
        &outcome,
    ) {
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
    if summary["state"] != serde_json::json!("checkpointed")
        || summary["checkpointCount"] != serde_json::json!(1)
    {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "verification requires exactly one completed checkpoint".into(),
        });
    }
    if !matches!(summary["preflightState"].as_str(), Some("green" | "yellow")) {
        return Err(ObserverError::State {
            path: summary_path,
            message: "verification requires a recorded non-red preflight result".into(),
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
    let decision = governance_decision_for_contract(&root, &contract, &refreshed_snapshot)?;
    let decision_value = serde_json::to_value(&decision).map_err(|error| ObserverError::State {
        path: contract_path.clone(),
        message: error.to_string(),
    })?;
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let mut summary: serde_json::Value = read_json(&summary_path)?;
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
    summary["preflightAt"] = now().into();
    atomic_json(&summary_path, &summary)?;
    Ok(evidence)
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
        let existing = read_json(&path)?;
        let existing: EvidenceRetentionPolicy =
            serde_json::from_value(existing).map_err(|error| ObserverError::State {
                path: path.clone(),
                message: error.to_string(),
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
    if policy.repository_id != repository_id(&root).to_string()
        || policy.work_item_id != work_item_id
    {
        return Err(ObserverError::State {
            path,
            message: "retention policy repository/work item binding mismatch".into(),
        });
    }
    validate_evidence_retention(&policy.retention).map_err(|error| ObserverError::State {
        path,
        message: error.to_string(),
    })?;
    Ok(Some(policy))
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

pub fn snapshot_digest(snapshot: &RepositorySnapshot) -> Result<Digest, ObserverError> {
    let mut stable = snapshot.clone();
    stable
        .changed_paths
        .retain(|path| !path.starts_with(".ai/"));
    cockpit_protocol::digest_json(&stable).map_err(|error| ObserverError::State {
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

fn contract_policy_rule<'a>(
    contract: &Contract,
    policy: &'a GovernancePolicy,
) -> Option<&'a cockpit_protocol::PolicyRule> {
    let operation = contract.operation.as_deref().unwrap_or_else(|| {
        if contract.risk.to_ascii_lowercase().contains("destructive") {
            "production_destructive"
        } else {
            "modify_source"
        }
    });
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
    governance_decision_for_contract_internal_with_archive(
        root,
        contract,
        snapshot,
        current_runtime,
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
        explicit_unknowns: signals.unknowns,
        outcome_state_override: None,
        authority_override: None,
    };
    let policy = effective_policy_for_contract(root, contract)?;
    apply_policy_to_governance_input(contract, policy.as_ref(), &mut input);
    Ok(evaluate(input))
}

fn read_contract(path: &Path) -> Result<cockpit_protocol::Contract, ObserverError> {
    let value: serde_json::Value = read_json(path)?;
    serde_json::from_value(value).map_err(|error| ObserverError::State {
        path: path.to_path_buf(),
        message: format!("invalid work item contract: {error}"),
    })
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
    let names = ["contract", "summary", "outcome"];
    let mut files = serde_json::Map::new();
    let mut pending = Vec::new();
    for name in names {
        let source_path = active.join(format!("{work_item_id}.{name}.json"));
        let target = archive.join(format!("{work_item_id}.{name}.json"));
        let bytes = fs::read(&source_path).map_err(|error| ObserverError::Read {
            path: source_path.clone(),
            source: error,
        })?;
        files.insert(
            format!("{name}Path"),
            serde_json::Value::String(format!(".ai/work-items/archive/{work_item_id}.{name}.json")),
        );
        files.insert(
            format!("{name}Digest"),
            serde_json::Value::String(Digest::sha256_bytes(&bytes).to_string()),
        );
        if target.exists() {
            return Err(ObserverError::State {
                path: target,
                message: "archive target already exists".into(),
            });
        }
        pending.push((source_path, target));
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
    let manifest = serde_json::json!({
        "protocolVersion": 1,
        "workItemId": work_item_id,
        "state": "archived",
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
        state: "archived".into(),
        timestamp,
    })
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
    let contract_path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    let summary_path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.summary.json"));
    let summary: serde_json::Value = read_json(&summary_path)?;
    if summary["state"] != serde_json::json!("finish_ready")
        || summary["checkpointCount"] != serde_json::json!(1)
        || summary["preflightState"] != serde_json::json!("green")
    {
        return Err(ObserverError::State {
            path: summary_path,
            message:
                "close requires archived finish_ready state, one checkpoint, and green preflight"
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
    if verification_evidence_state(&root, &contract, &snapshot, true, current_runtime)?
        != EvidenceState::Complete
    {
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
    validate_policy_decision(&root, &contract, human_decision)?;
    let outcome = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.outcome.json"));
    let outcome_value = read_json(&outcome)?;
    if outcome_value["verification"]["status"] != "verified" {
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
    decision["humanDecision"] = serde_json::Value::String(human_decision.decision.trim().into());
    decision["decisionState"] = serde_json::Value::String("confirmed".into());
    decision["structuredDecision"] =
        serde_json::to_value(human_decision).map_err(|error| ObserverError::State {
            path: root.join(".ai/decisions"),
            message: error.to_string(),
        })?;
    atomic_json(&decision_path, &decision)?;
    Ok(receipt)
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
        || manifest["state"] != serde_json::Value::String("archived".into())
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
    if contract.intent.trim().is_empty() {
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
    let evidence_state = if legacy {
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
    let (state, decision_state, summary, evidence_unknown) = if legacy {
        (
            OutcomeState::NotReady,
            DecisionState::Yellow,
            "Historical verification evidence uses a legacy schema and is not revalidated as a current result.",
            Some("legacy_evidence_historical"),
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
    })
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
    let profile: AttachedProfile = read_json(&profile_path).and_then(|value| {
        serde_json::from_value(value).map_err(|error| ObserverError::State {
            path: profile_path.clone(),
            message: error.to_string(),
        })
    })?;
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
        let confirmed = profile.tests.iter().any(|test| test == command);
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
            } else {
                vec!["command_not_profile_confirmed".into()]
            },
        });
    }
    capabilities.sort_by(|left, right| left.capability.cmp(&right.capability));
    capabilities.dedup_by(|left, right| left.capability == right.capability);
    Ok(CapabilityTruthRegistry {
        schema_version: 1,
        repository_id: repository_id(&root).to_string(),
        snapshot_digest: snapshot_digest(&snapshot)?,
        capabilities,
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
    if !path.is_file() {
        return Ok(None);
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
    if !contract_path.is_file() {
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
    let intelligence = read_work_item_intelligence(&root, work_item_id)?;
    let mut reasons = Vec::new();
    let mut conflicts = Vec::new();
    let mut dependencies_satisfied = true;
    let mut unknowns = Vec::new();
    if target_scope.is_empty() {
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
        let declared_conflict = intelligence.conflicts_with.iter().any(|id| id == other_id);
        if declared_conflict {
            conflicts.push(other_id.to_string());
            reasons.push(format!("explicit_conflict:{other_id}"));
            continue;
        }
        match scope_list_relation(&target_scope, &other_scope) {
            ScopeRelation::Overlap => {
                conflicts.push(other_id.to_string());
                reasons.push(format!("scope_overlap:{other_id}"));
            }
            ScopeRelation::Unknown => {
                reasons.push(format!("scope_overlap_unknown:{other_id}"));
            }
            ScopeRelation::Disjoint => {}
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
