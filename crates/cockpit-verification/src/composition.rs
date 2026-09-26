use crate::{VerificationCommand, VerificationReusePolicy, execute_bounded_with_process_observer};
use cockpit_core::Digest;
use cockpit_protocol::{CompositionBinding, RuntimeCapabilityError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[cfg(unix)]
use std::ffi::CString;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
#[cfg(unix)]
use std::io::Read;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const COMPOSITION_SCHEMA_VERSION: u32 = 1;
const PROCESS_OBSERVATION_SCHEMA_VERSION: u32 = 1;
const MAX_OUTPUT_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompositionIdentity {
    pub source_digest: Digest,
    pub dependency_digest: Digest,
    pub interface_digest: Digest,
    pub configuration_digest: Digest,
    pub toolchain_digest: Digest,
    pub lockfile_digest: Digest,
    pub generated_input_digest: Digest,
    pub environment_digest: Digest,
    pub verifier_digest: Digest,
    pub command_digest: Digest,
}

impl Default for CompositionIdentity {
    fn default() -> Self {
        let unobserved = || Digest::sha256_bytes(b"runtime-has-not-observed-composition-identity");
        Self {
            source_digest: unobserved(),
            dependency_digest: unobserved(),
            interface_digest: unobserved(),
            configuration_digest: unobserved(),
            toolchain_digest: unobserved(),
            lockfile_digest: unobserved(),
            generated_input_digest: unobserved(),
            environment_digest: unobserved(),
            verifier_digest: unobserved(),
            command_digest: unobserved(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompositionPrecondition {
    pub name: String,
    pub satisfied: bool,
    pub reason: String,
}

impl CompositionPrecondition {
    pub fn satisfied(name: &str) -> Self {
        Self {
            name: name.into(),
            satisfied: true,
            reason: "observed and bound".into(),
        }
    }

    pub fn unsatisfied(name: &str, reason: &str) -> Self {
        Self {
            name: name.into(),
            satisfied: false,
            reason: reason.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompositionCommand {
    pub node_id: String,
    pub program: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub environment: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub input_paths: Vec<String>,
    #[serde(default)]
    pub covered_scenarios: Vec<String>,
    #[serde(default)]
    pub covered_constraints: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompositionInput {
    pub repository_root: PathBuf,
    pub state_dir: PathBuf,
    pub binding: CompositionBinding,
    #[serde(default)]
    pub identity: CompositionIdentity,
    pub commands: Vec<CompositionCommand>,
    #[serde(default)]
    pub reusable_node_ids: Vec<String>,
    pub preconditions: Vec<CompositionPrecondition>,
    #[serde(default = "default_composition_timeout")]
    pub timeout_seconds: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReuseDecisionKind {
    Reuse,
    Execute,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReuseDecision {
    pub kind: ReuseDecisionKind,
    pub reason: String,
    pub predecessor_attempt_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompositionExecutionRecord {
    pub node_id: String,
    pub program: String,
    pub args: Vec<String>,
    #[serde(default = "legacy_node_identity_digest")]
    pub identity_digest: Digest,
    pub spawned: bool,
    #[serde(default)]
    pub reused: bool,
    pub passed: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub output_digest: Digest,
    pub timed_out: bool,
    #[serde(default)]
    pub predecessor_attempt_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompositionCleanup {
    pub attempted: bool,
    pub removed: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompositionAttempt {
    #[serde(default = "composition_schema_version")]
    pub schema_version: u32,
    pub attempt_id: String,
    pub binding: CompositionBinding,
    pub identity: CompositionIdentity,
    pub preconditions: Vec<CompositionPrecondition>,
    pub isolated_worktree: String,
    pub text_conflicts: Vec<String>,
    pub execution_records: Vec<CompositionExecutionRecord>,
    pub processes_spawned: usize,
    pub reuse_decision: ReuseDecision,
    pub passed: bool,
    pub failure: Option<String>,
    #[serde(default)]
    pub cleanup: Option<CompositionCleanup>,
    #[serde(default)]
    pub recorded_at_unix_nanos: u128,
    #[serde(default)]
    pub owner_pid: Option<u32>,
    #[serde(default)]
    pub process_observation_schema_version: u32,
    #[serde(default)]
    pub active_execution_node: Option<String>,
    #[serde(default)]
    pub active_process_group_id: Option<u32>,
}

impl CompositionAttempt {
    /// Whether this attempt is internally coherent enough to be projected or
    /// reused as a successful terminal composition.
    pub fn is_coherent_successful_terminal(&self) -> bool {
        is_reusable_terminal_attempt(self)
    }
}

fn composition_schema_version() -> u32 {
    COMPOSITION_SCHEMA_VERSION
}
fn default_composition_timeout() -> u64 {
    crate::DEFAULT_EXECUTION_SECONDS
}
fn legacy_node_identity_digest() -> Digest {
    Digest::sha256_bytes(b"legacy-composition-node-identity")
}

#[derive(Debug, Error)]
pub enum CompositionError {
    #[error(transparent)]
    Runtime(#[from] RuntimeCapabilityError),
    #[error("composition binding is invalid: {0}")]
    InvalidBinding(String),
    #[error("composition state I/O failed at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("composition state serialization failed: {0}")]
    Serialization(String),
    #[error("composition attempt {attempt_id} is still owned by live process {owner_pid}")]
    ActiveAttempt { attempt_id: String, owner_pid: u32 },
    #[error("composition attempt {attempt_id} has no verifiable owner; preserving its worktree")]
    UnknownAttemptOwner { attempt_id: String },
    #[error(
        "composition attempt {attempt_id} still has active verifier process group {process_group_id}; preserving its worktree"
    )]
    ActiveVerifierProcessGroup {
        attempt_id: String,
        process_group_id: u32,
    },
    #[error(
        "composition attempt {attempt_id} still has verifier process {process_id} using its worktree; preserving it"
    )]
    ActiveVerifierDescendant { attempt_id: String, process_id: u32 },
}

pub fn run_composition(input: CompositionInput) -> Result<CompositionAttempt, CompositionError> {
    let mut input = input;
    input.binding.verifier.validate_candidate()?;
    validate_binding(&input.binding)?;
    // Caller identity fields are descriptive only. Start from explicit
    // sentinels so a failed observation cannot persist self-reported claims.
    input.identity = CompositionIdentity::default();
    input.identity.command_digest = composition_commands_digest(&input.commands);

    let predecessor = load_latest_attempt(&input.state_dir, &input.binding)?;
    let predecessor =
        reconcile_abandoned_attempt(&input.repository_root, &input.state_dir, predecessor)?;
    let recorded_at_unix_nanos = now_unix_nanos();
    let attempt_id = new_attempt_id(&input, recorded_at_unix_nanos);
    let reuse_decision = predecessor.as_ref().map_or(
        ReuseDecision {
            kind: ReuseDecisionKind::Execute,
            reason: "fresh composition attempt".into(),
            predecessor_attempt_id: None,
        },
        |previous| classify_reuse(previous, &input),
    );
    let mut attempt = CompositionAttempt {
        schema_version: COMPOSITION_SCHEMA_VERSION,
        attempt_id,
        binding: input.binding.clone(),
        identity: input.identity.clone(),
        preconditions: input.preconditions.clone(),
        isolated_worktree: String::new(),
        text_conflicts: Vec::new(),
        execution_records: Vec::new(),
        processes_spawned: 0,
        reuse_decision,
        passed: false,
        failure: Some("in_progress".into()),
        cleanup: None,
        recorded_at_unix_nanos,
        owner_pid: Some(std::process::id()),
        process_observation_schema_version: PROCESS_OBSERVATION_SCHEMA_VERSION,
        active_execution_node: None,
        active_process_group_id: None,
    };

    // This snapshot is the recovery boundary: an interrupted parent leaves a
    // durable in_progress attempt rather than an invisible execution.
    persist_attempt(&input.state_dir, &attempt)?;

    if input.commands.is_empty() {
        fail_without_worktree(&input.state_dir, &mut attempt, "required_checks_empty")?;
        return Ok(attempt);
    }
    if !valid_command_graph(&input.commands) {
        fail_without_worktree(
            &input.state_dir,
            &mut attempt,
            "invalid_composition_command_graph",
        )?;
        return Ok(attempt);
    }
    if let Some(precondition) = input.preconditions.iter().find(|item| !item.satisfied) {
        fail_without_worktree(
            &input.state_dir,
            &mut attempt,
            &format!(
                "precondition_failed:{}:{}",
                precondition.name, precondition.reason
            ),
        )?;
        return Ok(attempt);
    }
    if input
        .commands
        .iter()
        .any(|command| controlled_command_environment(command).is_none())
    {
        fail_without_worktree(
            &input.state_dir,
            &mut attempt,
            "unbound_runtime_environment_override",
        )?;
        return Ok(attempt);
    }

    let unique = unique_composition_parent();
    let worktree = unique.join("composition");
    create_private_composition_parent(&unique).map_err(|source| CompositionError::Io {
        path: unique.clone(),
        source,
    })?;
    let mut worktree_guard = CompositionWorktreeGuard::new(
        input.repository_root.clone(),
        worktree.clone(),
        unique.clone(),
    );
    attempt.isolated_worktree = worktree.to_string_lossy().into_owned();
    persist_attempt(&input.state_dir, &attempt)?;
    let added = git_command(
        &input.repository_root,
        &[
            "worktree",
            "add",
            "--detach",
            worktree.to_str().unwrap_or_default(),
            &input.binding.target_sha,
        ],
    );
    if !added.success {
        attempt.failure = Some(format!("worktree_add_failed:{}", bounded(&added.stderr)));
        attempt.cleanup = Some(worktree_guard.cleanup());
        persist_attempt(&input.state_dir, &attempt)?;
        return Ok(attempt);
    }
    worktree_guard.mark_registered();

    for head in &input.binding.participant_heads {
        let result = git_command(
            &worktree,
            &[
                "-c",
                "user.email=ai-cockpit@example.invalid",
                "-c",
                "user.name=AI Cockpit composition",
                "merge",
                "--no-edit",
                "--no-ff",
                head,
            ],
        );
        if !result.success {
            attempt.text_conflicts.push(bounded(&result.stderr));
            attempt.failure = Some("text_conflict_or_merge_failure".into());
            let _ = git_command(&worktree, &["merge", "--abort"]);
            attempt.cleanup = Some(worktree_guard.cleanup());
            persist_attempt(&input.state_dir, &attempt)?;
            return Ok(attempt);
        }
    }

    let commands_bound = input.commands.iter().all(|command| {
        controlled_command_environment(command).is_some_and(|environment| {
            resolve_executable(&worktree, &command.program, &environment).is_some()
        })
    });
    if !commands_bound {
        attempt.failure = Some("composition_executable_unbound".into());
        attempt.cleanup = Some(worktree_guard.cleanup());
        persist_attempt(&input.state_dir, &attempt)?;
        return Ok(attempt);
    }

    let (identity_observed, runtime_toolchain_digest) =
        if let Some(observed) = observe_composition_identity(&worktree, &input) {
            input.identity = observed.identity;
            attempt.identity = input.identity.clone();
            (true, Some(observed.runtime_toolchain_digest))
        } else {
            (false, None)
        };
    let predecessor_id = predecessor
        .as_ref()
        .map(|previous| previous.attempt_id.as_str());
    let mut node_identities_observed = true;
    let mut all_nodes_reused = predecessor
        .as_ref()
        .is_some_and(is_reusable_terminal_attempt)
        && identity_observed;
    attempt.reuse_decision = ReuseDecision {
        kind: if identity_observed {
            ReuseDecisionKind::Execute
        } else {
            ReuseDecisionKind::Unknown
        },
        reason: "node inputs and upstream receipts are evaluated in execution order".into(),
        predecessor_attempt_id: predecessor_id.map(str::to_owned),
    };
    persist_attempt(&input.state_dir, &attempt)?;

    for command in &input.commands {
        let environment = controlled_command_environment(command)
            .expect("command environment was checked before any process started");
        let Some(executable) = resolve_executable(&worktree, &command.program, &environment) else {
            node_identities_observed = false;
            all_nodes_reused = false;
            attempt.failure = Some(format!(
                "composition_executable_became_unbound:{}",
                command.node_id
            ));
            break;
        };
        let current_identity = observed_node_identity(
            &worktree,
            &input,
            command,
            &attempt.execution_records,
            &executable,
            &environment,
            runtime_toolchain_digest.as_ref(),
        );
        if current_identity.is_none() {
            node_identities_observed = false;
        }
        let reusable = if identity_observed
            && predecessor
                .as_ref()
                .is_some_and(is_reusable_terminal_attempt)
            && input
                .reusable_node_ids
                .iter()
                .any(|node_id| node_id == &command.node_id)
        {
            predecessor.as_ref().and_then(|previous| {
                let identity = current_identity.as_ref()?;
                previous
                    .execution_records
                    .iter()
                    .find(|record| {
                        record.node_id == command.node_id
                            && record.passed
                            && !record.timed_out
                            && (record.spawned || record.reused)
                            && &record.identity_digest == identity
                    })
                    .map(|record| reused_record(record, command, previous.attempt_id.as_str()))
            })
        } else {
            None
        };
        if let Some(record) = reusable {
            attempt.execution_records.push(record);
            persist_attempt(&input.state_dir, &attempt)?;
            continue;
        }
        all_nodes_reused = false;
        attempt.active_execution_node = Some(command.node_id.clone());
        attempt.active_process_group_id = None;
        persist_attempt(&input.state_dir, &attempt)?;
        let record = execute_node(
            &worktree,
            command,
            &input,
            &executable,
            &environment,
            current_identity,
            &attempt,
        );
        attempt.active_execution_node = None;
        attempt.active_process_group_id = None;
        attempt.processes_spawned += usize::from(record.spawned);
        let passed = record.passed;
        attempt.execution_records.push(record);
        persist_attempt(&input.state_dir, &attempt)?;
        if !passed {
            let exit_code = attempt
                .execution_records
                .last()
                .and_then(|item| item.exit_code);
            attempt.failure = Some(format!("command_failed:exit={exit_code:?}"));
            persist_attempt(&input.state_dir, &attempt)?;
            break;
        }
    }

    attempt.passed = (attempt.failure.is_none()
        || attempt.failure.as_deref() == Some("in_progress"))
        && attempt.execution_records.len() == input.commands.len()
        && attempt.execution_records.iter().all(|record| record.passed);
    if attempt.passed {
        attempt.failure = None;
    }
    attempt.reuse_decision = ReuseDecision {
        kind: if !identity_observed || !node_identities_observed {
            ReuseDecisionKind::Unknown
        } else if all_nodes_reused && attempt.passed {
            ReuseDecisionKind::Reuse
        } else {
            ReuseDecisionKind::Execute
        },
        reason: if !identity_observed {
            "runtime could not observe a complete composition identity; reuse disabled".into()
        } else if !node_identities_observed {
            "one or more node inputs or upstream receipts are not fully observable; reuse disabled for those nodes".into()
        } else if all_nodes_reused && attempt.passed {
            "runtime-observed node inputs and upstream receipts match durable predecessor identities".into()
        } else {
            "one or more nodes were executed because their observed inputs or upstream receipts changed".into()
        },
        predecessor_attempt_id: predecessor_id.map(str::to_owned),
    };
    match verifier_process_using_worktree(&worktree) {
        Ok(Some(process_id)) => {
            attempt.passed = false;
            attempt.failure = Some(format!("verifier_descendant_active:{process_id}"));
            worktree_guard.preserve();
            persist_attempt(&input.state_dir, &attempt)?;
            return Ok(attempt);
        }
        Err(error) => {
            attempt.passed = false;
            attempt.failure = Some(format!("verifier_process_state_unknown:{error}"));
            worktree_guard.preserve();
            persist_attempt(&input.state_dir, &attempt)?;
            return Ok(attempt);
        }
        Ok(None) => {}
    }
    attempt.cleanup = Some(worktree_guard.cleanup());
    if !attempt
        .cleanup
        .as_ref()
        .is_some_and(|cleanup| cleanup.removed)
    {
        attempt.passed = false;
        attempt.failure = Some("composition_cleanup_failed".into());
    }
    persist_attempt(&input.state_dir, &attempt)?;
    Ok(attempt)
}

pub fn classify_reuse(previous: &CompositionAttempt, current: &CompositionInput) -> ReuseDecision {
    if previous.schema_version != COMPOSITION_SCHEMA_VERSION {
        return ReuseDecision {
            kind: ReuseDecisionKind::Unknown,
            reason: "unsupported predecessor schema".into(),
            predecessor_attempt_id: Some(previous.attempt_id.clone()),
        };
    }
    if previous
        .execution_records
        .iter()
        .any(|record| !record.spawned && !record.reused)
    {
        return ReuseDecision {
            kind: ReuseDecisionKind::Unknown,
            reason: "predecessor execution identity is incomplete".into(),
            predecessor_attempt_id: Some(previous.attempt_id.clone()),
        };
    }
    let _ = current;
    ReuseDecision {
        kind: ReuseDecisionKind::Unknown,
        reason: "reuse classification requires runtime-observed node inputs".into(),
        predecessor_attempt_id: Some(previous.attempt_id.clone()),
    }
}

pub fn composition_commands_digest(commands: &[CompositionCommand]) -> Digest {
    let bytes = serde_json::to_vec(commands).unwrap_or_default();
    Digest::sha256_bytes(&bytes)
}

fn validate_binding(binding: &CompositionBinding) -> Result<(), CompositionError> {
    if binding.schema_version != COMPOSITION_SCHEMA_VERSION
        || binding.binding_id.trim().is_empty()
        || binding.target_branch.trim().is_empty()
        || binding.target_sha.len() != 40
        || binding.participant_work_items.len() != binding.participant_heads.len()
        || binding.participant_work_items.len() != binding.contract_digests.len()
        || binding.participant_work_items.is_empty()
    {
        return Err(CompositionError::InvalidBinding(
            "composition identity cardinality is invalid".into(),
        ));
    }
    if binding
        .participant_work_items
        .iter()
        .any(|item| item.trim().is_empty())
    {
        return Err(CompositionError::InvalidBinding(
            "participant Work Item identity is empty".into(),
        ));
    }
    Ok(())
}

fn valid_command_graph(commands: &[CompositionCommand]) -> bool {
    let mut prior_nodes = BTreeSet::new();
    for command in commands {
        if command.node_id.trim().is_empty() || !prior_nodes.insert(command.node_id.as_str()) {
            return false;
        }
        let mut dependencies = BTreeSet::new();
        if command.depends_on.iter().any(|dependency| {
            dependency.trim().is_empty()
                || !dependencies.insert(dependency.as_str())
                || !prior_nodes.contains(dependency.as_str())
                || dependency == &command.node_id
        }) {
            return false;
        }
    }
    true
}

fn fail_without_worktree(
    state_dir: &Path,
    attempt: &mut CompositionAttempt,
    failure: &str,
) -> Result<(), CompositionError> {
    attempt.failure = Some(failure.into());
    attempt.cleanup = Some(CompositionCleanup {
        attempted: false,
        removed: true,
        error: None,
    });
    persist_attempt(state_dir, attempt)
}

fn load_latest_attempt(
    state_dir: &Path,
    binding: &CompositionBinding,
) -> Result<Option<CompositionAttempt>, CompositionError> {
    let entries = match fs::read_dir(state_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(CompositionError::Io {
                path: state_dir.to_path_buf(),
                source,
            });
        }
    };
    let mut latest: Option<CompositionAttempt> = None;
    for entry in entries {
        let entry = entry.map_err(|source| CompositionError::Io {
            path: state_dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|source| CompositionError::Io {
            path: path.clone(),
            source,
        })?;
        if !metadata.file_type().is_file() {
            return Err(CompositionError::Serialization(format!(
                "composition state is not a regular file: {}",
                path.display()
            )));
        }
        let bytes = fs::read(&path).map_err(|source| CompositionError::Io {
            path: path.clone(),
            source,
        })?;
        let candidate: CompositionAttempt = serde_json::from_slice(&bytes)
            .map_err(|error| CompositionError::Serialization(error.to_string()))?;
        if path.file_stem().and_then(|stem| stem.to_str()) != Some(candidate.attempt_id.as_str()) {
            return Err(CompositionError::Serialization(format!(
                "composition attempt filename does not match embedded identity: {}",
                path.display()
            )));
        }
        if !same_composition_lineage(&candidate.binding, binding) {
            continue;
        }
        if latest.as_ref().is_none_or(|current| {
            (candidate.recorded_at_unix_nanos, &candidate.attempt_id)
                > (current.recorded_at_unix_nanos, &current.attempt_id)
        }) {
            latest = Some(candidate);
        }
    }
    Ok(latest)
}

fn same_composition_lineage(previous: &CompositionBinding, current: &CompositionBinding) -> bool {
    previous.repository_id == current.repository_id
        && previous.target_branch == current.target_branch
        && previous.participant_work_items == current.participant_work_items
        && previous.verifier == current.verifier
}

fn is_reusable_terminal_attempt(attempt: &CompositionAttempt) -> bool {
    attempt.schema_version == COMPOSITION_SCHEMA_VERSION
        && attempt.passed
        && attempt.failure.is_none()
        && attempt.preconditions.iter().all(|item| item.satisfied)
        && !attempt.isolated_worktree.is_empty()
        && attempt.text_conflicts.is_empty()
        && attempt
            .cleanup
            .as_ref()
            .is_some_and(|cleanup| cleanup.attempted && cleanup.removed && cleanup.error.is_none())
        && attempt.active_execution_node.is_none()
        && attempt.active_process_group_id.is_none()
        && !attempt.execution_records.is_empty()
        && attempt.processes_spawned
            == attempt
                .execution_records
                .iter()
                .filter(|record| record.spawned)
                .count()
        && attempt.execution_records.iter().all(|record| {
            record.passed
                && !record.timed_out
                && record.exit_code == Some(0)
                && (record.spawned ^ record.reused)
                && if record.reused {
                    record.predecessor_attempt_id.is_some()
                } else {
                    record.predecessor_attempt_id.is_none()
                }
        })
}

fn reconcile_abandoned_attempt(
    repository_root: &Path,
    state_dir: &Path,
    previous: Option<CompositionAttempt>,
) -> Result<Option<CompositionAttempt>, CompositionError> {
    let Some(mut attempt) = previous else {
        return Ok(None);
    };
    let cleanup_pending = attempt.failure.as_deref() == Some("in_progress")
        || attempt
            .cleanup
            .as_ref()
            .is_some_and(|cleanup| !cleanup.removed)
        || (!attempt.isolated_worktree.is_empty() && attempt.cleanup.is_none());
    if !cleanup_pending {
        return Ok(Some(attempt));
    }
    let Some(owner_pid) = attempt.owner_pid else {
        return Err(CompositionError::UnknownAttemptOwner {
            attempt_id: attempt.attempt_id,
        });
    };
    if process_is_alive(owner_pid) {
        return Err(CompositionError::ActiveAttempt {
            attempt_id: attempt.attempt_id,
            owner_pid,
        });
    }

    if attempt.failure.as_deref() == Some("in_progress")
        && attempt.process_observation_schema_version < PROCESS_OBSERVATION_SCHEMA_VERSION
    {
        return Err(CompositionError::UnknownAttemptOwner {
            attempt_id: attempt.attempt_id,
        });
    }

    match (
        attempt.active_execution_node.as_deref(),
        attempt.active_process_group_id,
    ) {
        (Some(_), Some(process_group_id)) if process_group_is_alive(process_group_id) => {
            return Err(CompositionError::ActiveVerifierProcessGroup {
                attempt_id: attempt.attempt_id,
                process_group_id,
            });
        }
        (Some(_), Some(_)) => {
            attempt.active_execution_node = None;
            attempt.active_process_group_id = None;
        }
        (Some(_), None) | (None, Some(_)) => {
            return Err(CompositionError::UnknownAttemptOwner {
                attempt_id: attempt.attempt_id,
            });
        }
        (None, None) => {}
    }

    let validated_paths = if attempt.isolated_worktree.is_empty() {
        None
    } else {
        Some(
            validated_composition_paths(&attempt, owner_pid).ok_or_else(|| {
                CompositionError::Serialization(format!(
                    "interrupted composition path is outside its owned temporary namespace: {}",
                    attempt.isolated_worktree
                ))
            })?,
        )
    };
    if let Some((worktree, _)) = &validated_paths {
        match verifier_process_using_worktree(worktree) {
            Ok(Some(process_id)) => {
                return Err(CompositionError::ActiveVerifierDescendant {
                    attempt_id: attempt.attempt_id,
                    process_id,
                });
            }
            Err(_) => {
                return Err(CompositionError::UnknownAttemptOwner {
                    attempt_id: attempt.attempt_id,
                });
            }
            Ok(None) => {}
        }
    }
    let cleanup = if let Some((worktree, parent)) = validated_paths {
        let registered = worktree_is_registered(repository_root, &worktree)?;
        cleanup_worktree(repository_root, &worktree, &parent, registered)
    } else {
        CompositionCleanup {
            attempted: false,
            removed: true,
            error: None,
        }
    };
    if !cleanup.removed {
        return Err(CompositionError::Serialization(format!(
            "interrupted composition cleanup could not be proven complete: {}",
            cleanup.error.as_deref().unwrap_or("unknown cleanup error")
        )));
    }
    attempt.cleanup = Some(cleanup);
    if attempt.failure.as_deref() == Some("in_progress") {
        attempt.failure = Some("interrupted_owner_terminated".into());
    }
    persist_attempt(state_dir, &attempt)?;
    Ok(Some(attempt))
}

fn validated_composition_paths(
    attempt: &CompositionAttempt,
    owner_pid: u32,
) -> Option<(PathBuf, PathBuf)> {
    let worktree = PathBuf::from(&attempt.isolated_worktree);
    if !worktree.is_absolute()
        || worktree.file_name().and_then(|name| name.to_str()) != Some("composition")
    {
        return None;
    }
    let parent = worktree.parent()?.to_path_buf();
    let parent_name = parent.file_name()?.to_str()?;
    let suffix = parent_name.strip_prefix("ai-cockpit-composition-")?;
    let (pid, timestamp) = suffix.split_once('-')?;
    if pid.parse::<u32>().ok()? != owner_pid || timestamp.parse::<u128>().ok()? == 0 {
        return None;
    }
    let canonical_temp = fs::canonicalize(std::env::temp_dir()).ok()?;
    let canonical_parent_parent = fs::canonicalize(parent.parent()?).ok()?;
    if canonical_parent_parent != canonical_temp {
        return None;
    }
    if let Ok(metadata) = fs::symlink_metadata(&parent)
        && (!metadata.is_dir() || metadata.file_type().is_symlink())
    {
        return None;
    }
    if let Ok(metadata) = fs::symlink_metadata(&worktree)
        && (!metadata.is_dir() || metadata.file_type().is_symlink())
    {
        return None;
    }
    Some((worktree, parent))
}

fn worktree_is_registered(
    repository_root: &Path,
    worktree: &Path,
) -> Result<bool, CompositionError> {
    let listed = git_command(repository_root, &["worktree", "list", "--porcelain"]);
    if !listed.success {
        return Err(CompositionError::Serialization(format!(
            "cannot verify interrupted worktree registration: {}",
            bounded(&listed.stderr)
        )));
    }
    let expected = fs::canonicalize(worktree).unwrap_or_else(|_| worktree.to_path_buf());
    Ok(listed.stdout.lines().any(|line| {
        line.strip_prefix("worktree ").is_some_and(|path| {
            let listed = PathBuf::from(path);
            fs::canonicalize(&listed).unwrap_or(listed) == expected
        })
    }))
}

#[cfg(unix)]
fn process_is_alive(pid: u32) -> bool {
    // SAFETY: `kill(pid, 0)` performs no signal delivery and only probes the
    // process table. A permission error is treated as live, failing closed.
    let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
    if result == 0 {
        return true;
    }
    !matches!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(code) if code == libc::ESRCH
    )
}

#[cfg(windows)]
fn process_is_alive(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{
        CloseHandle, ERROR_INVALID_PARAMETER, GetLastError, STILL_ACTIVE,
    };
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    // SAFETY: The returned process handle is checked and always closed. Any
    // inability to inspect a process is treated as live to avoid unsafe cleanup.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return GetLastError() != ERROR_INVALID_PARAMETER;
        }
        let mut exit_code = 0;
        let inspected = GetExitCodeProcess(handle, &mut exit_code) != 0;
        CloseHandle(handle);
        !inspected || exit_code == STILL_ACTIVE as u32
    }
}

#[cfg(unix)]
fn process_group_is_alive(process_group_id: u32) -> bool {
    // SAFETY: a negative pid probes the process group without delivering a signal.
    let result = unsafe { libc::kill(-(process_group_id as libc::pid_t), 0) };
    if result == 0 {
        return true;
    }
    !matches!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(code) if code == libc::ESRCH
    )
}

#[cfg(target_os = "linux")]
fn verifier_process_using_worktree(worktree: &Path) -> Result<Option<u32>, String> {
    use std::os::unix::fs::MetadataExt;

    let worktree = fs::canonicalize(worktree).map_err(|error| error.to_string())?;
    let proc_entries = fs::read_dir("/proc").map_err(|error| error.to_string())?;
    for entry in proc_entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let Some(process_id) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        if process_id == std::process::id() {
            continue;
        }
        let process_dir = entry.path();
        let metadata = match fs::metadata(&process_dir) {
            Ok(metadata) if metadata.uid() == unsafe { libc::getuid() } => metadata,
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.to_string()),
        };
        let _ = metadata;
        let cwd = process_dir.join("cwd");
        match fs::read_link(&cwd) {
            Ok(path) if path_is_within(&worktree, &path) => return Ok(Some(process_id)),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                return Err(format!(
                    "cannot inspect process {process_id} working directory"
                ));
            }
            Err(error) => return Err(error.to_string()),
        }
        let descriptors = process_dir.join("fd");
        let descriptors = match fs::read_dir(&descriptors) {
            Ok(descriptors) => descriptors,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                return Err(format!(
                    "cannot inspect process {process_id} file descriptors"
                ));
            }
            Err(error) => return Err(error.to_string()),
        };
        for descriptor in descriptors {
            let descriptor = descriptor.map_err(|error| error.to_string())?;
            match fs::read_link(descriptor.path()) {
                Ok(path) if path_is_within(&worktree, &path) => return Ok(Some(process_id)),
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                    return Err(format!("cannot inspect process {process_id} open files"));
                }
                Err(error) => return Err(error.to_string()),
            }
        }
    }
    Ok(None)
}

#[cfg(target_os = "macos")]
fn verifier_process_using_worktree(worktree: &Path) -> Result<Option<u32>, String> {
    let worktree = fs::canonicalize(worktree).map_err(|error| error.to_string())?;
    let user_id = unsafe { libc::getuid() }.to_string();
    let output = Command::new("lsof")
        .args(["-nP", "-Fpn", "-a", "-u", &user_id, "+D"])
        .arg(&worktree)
        .output()
        .map_err(|error| format!("cannot inspect open worktree files: {error}"))?;
    let mut process_id = None;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(value) = line.strip_prefix('p') {
            process_id = value.parse::<u32>().ok();
        } else if let Some(value) = line.strip_prefix('n')
            && path_is_within(&worktree, Path::new(value))
        {
            return Ok(process_id);
        }
    }
    if !output.status.success() && !(output.status.code() == Some(1) && output.stdout.is_empty()) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "lsof exited with status {:?}: {}",
            output.status.code(),
            stderr.trim()
        ));
    }
    Ok(None)
}

#[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
fn verifier_process_using_worktree(_worktree: &Path) -> Result<Option<u32>, String> {
    Err("process working-directory inspection is unsupported on this Unix platform".into())
}

#[cfg(windows)]
fn verifier_process_using_worktree(_worktree: &Path) -> Result<Option<u32>, String> {
    // The executor's Windows Job Object owns descendants and kills them when
    // the parent handle closes; no separate worktree-path scan is necessary.
    Ok(None)
}

#[cfg(not(any(unix, windows)))]
fn verifier_process_using_worktree(_worktree: &Path) -> Result<Option<u32>, String> {
    Err("process working-directory inspection is unsupported on this platform".into())
}

fn path_is_within(root: &Path, candidate: &Path) -> bool {
    let candidate = candidate.to_string_lossy();
    let candidate = candidate.strip_suffix(" (deleted)").unwrap_or(&candidate);
    Path::new(candidate) == root || Path::new(candidate).starts_with(root)
}

#[cfg(windows)]
fn process_group_is_alive(process_group_id: u32) -> bool {
    // The bounded executor owns a kill-on-close Job Object on Windows; the
    // observed process identifier is therefore the strongest portable probe.
    process_is_alive(process_group_id)
}

#[cfg(not(any(unix, windows)))]
fn process_group_is_alive(_process_group_id: u32) -> bool {
    true
}

fn reused_record(
    previous: &CompositionExecutionRecord,
    command: &CompositionCommand,
    predecessor_attempt_id: &str,
) -> CompositionExecutionRecord {
    CompositionExecutionRecord {
        node_id: command.node_id.clone(),
        program: command.program.clone(),
        args: command.args.clone(),
        identity_digest: previous.identity_digest.clone(),
        spawned: false,
        reused: true,
        passed: true,
        exit_code: previous.exit_code,
        stdout: previous.stdout.clone(),
        stderr: previous.stderr.clone(),
        output_digest: previous.output_digest.clone(),
        timed_out: false,
        predecessor_attempt_id: Some(predecessor_attempt_id.into()),
    }
}

fn execute_node(
    worktree: &Path,
    command: &CompositionCommand,
    input: &CompositionInput,
    executable: &ResolvedExecutable,
    environment: &BTreeMap<String, String>,
    identity_digest: Option<Digest>,
    attempt: &CompositionAttempt,
) -> CompositionExecutionRecord {
    let identity_digest = identity_digest
        .unwrap_or_else(|| Digest::sha256_bytes(b"unknown-composition-node-identity"));
    let verification_command = VerificationCommand::new(
        &command.node_id,
        &executable.launch_path.to_string_lossy(),
        command.args.clone(),
        VerificationReusePolicy::NeverReuse,
    )
    .with_current_dir(worktree)
    .with_cleared_environment()
    .with_environment(
        environment
            .iter()
            .map(|(key, value)| (OsString::from(key), OsString::from(value)))
            .collect(),
    )
    .with_timeout_seconds(input.timeout_seconds);
    let state_dir = input.state_dir.clone();
    let attempt_id = attempt.attempt_id.clone();
    let owner_pid = attempt.owner_pid.unwrap_or(std::process::id());
    let expected_node_id = command.node_id.clone();
    match execute_bounded_with_process_observer(
        vec![verification_command],
        1,
        move |node_id, process_group_id, active| {
            if node_id != expected_node_id {
                return Err(format!("unexpected active verification node: {node_id}"));
            }
            update_active_process_record(
                &state_dir,
                &attempt_id,
                owner_pid,
                node_id,
                process_group_id,
                active,
            )
        },
    ) {
        Ok(receipt) => {
            let execution = receipt
                .execution_records
                .iter()
                .find(|record| record.node_id == command.node_id);
            let result = receipt
                .results
                .iter()
                .find(|result| result.node_id == command.node_id);
            let stdout = execution
                .map(|record| decode_hex_bounded(&record.stdout_hex))
                .unwrap_or_default();
            let stderr = execution
                .map(|record| decode_hex_bounded(&record.stderr_hex))
                .unwrap_or_else(|| "bounded_executor_missing_record".into());
            let output_digest = result
                .and_then(|result| result.output_digest.as_deref())
                .and_then(|digest| digest.parse().ok())
                .unwrap_or_else(|| Digest::sha256_bytes(format!("{stdout}{stderr}").as_bytes()));
            CompositionExecutionRecord {
                node_id: command.node_id.clone(),
                program: command.program.clone(),
                args: command.args.clone(),
                identity_digest,
                spawned: execution.is_some_and(|record| record.spawned),
                reused: false,
                passed: result.is_some_and(|result| result.passed),
                exit_code: execution.and_then(|record| record.exit_code),
                stdout,
                stderr,
                output_digest,
                timed_out: execution.is_some_and(|record| record.timed_out),
                predecessor_attempt_id: None,
            }
        }
        Err(error) => CompositionExecutionRecord {
            node_id: command.node_id.clone(),
            program: command.program.clone(),
            args: command.args.clone(),
            identity_digest,
            spawned: false,
            reused: false,
            passed: false,
            exit_code: None,
            stdout: String::new(),
            stderr: bounded(&error.to_string()),
            output_digest: Digest::sha256_bytes(error.to_string().as_bytes()),
            timed_out: false,
            predecessor_attempt_id: None,
        },
    }
}

fn update_active_process_record(
    state_dir: &Path,
    attempt_id: &str,
    owner_pid: u32,
    node_id: &str,
    process_group_id: u32,
    active: bool,
) -> Result<(), String> {
    let path = state_dir.join(format!("{attempt_id}.json"));
    let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    if !metadata.file_type().is_file() {
        return Err("composition attempt is not a regular file".into());
    }
    let bytes = fs::read(&path).map_err(|error| error.to_string())?;
    let mut attempt: CompositionAttempt =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if attempt.attempt_id != attempt_id
        || attempt.owner_pid != Some(owner_pid)
        || attempt.active_execution_node.as_deref() != Some(node_id)
    {
        return Err("composition active-process identity changed".into());
    }
    if active {
        if attempt.active_process_group_id.is_some() {
            return Err("composition already records an active process group".into());
        }
        attempt.active_process_group_id = Some(process_group_id);
    } else {
        if attempt.active_process_group_id != Some(process_group_id) {
            return Err("composition active process group does not match".into());
        }
        attempt.active_process_group_id = None;
        attempt.active_execution_node = None;
    }
    persist_attempt(state_dir, &attempt).map_err(|error| error.to_string())
}

struct ObservedCompositionIdentity {
    identity: CompositionIdentity,
    runtime_toolchain_digest: Digest,
}

fn observe_composition_identity(
    worktree: &Path,
    input: &CompositionInput,
) -> Option<ObservedCompositionIdentity> {
    let source = git_text(worktree, &["rev-parse", "HEAD^{tree}"])?;
    let lockfiles = git_text(worktree, &["ls-tree", "-r", "--name-only", "HEAD"])?
        .lines()
        .filter(|path| is_lockfile(path))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let lockfile_digest = digest_paths(worktree, &lockfiles)?;
    let configuration_paths = git_text(worktree, &["ls-tree", "-r", "--name-only", "HEAD"])?
        .lines()
        .filter(|path| is_configuration_file(path))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let configuration_digest = digest_paths(worktree, &configuration_paths)?;
    let interface_digest = digest_serialized(&input.binding.contract_digests)?;
    let runtime_environment = controlled_command_environment(input.commands.first()?)?;
    let runtime_toolchain_digest =
        observe_runtime_toolchain_digest(worktree, &runtime_environment)?;
    let toolchain = input
        .commands
        .iter()
        .map(|command| {
            let environment = controlled_command_environment(command)?;
            let executable = resolve_executable(worktree, &command.program, &environment)?;
            Some(executable.digest)
        })
        .collect::<Option<Vec<_>>>()?;
    let observed_toolchain_digest = digest_serialized(&(&toolchain, &runtime_toolchain_digest))?;
    let environment_digest = observed_environment_digest(&input.commands)?;
    let generated_paths = input
        .commands
        .iter()
        .flat_map(|command| command.input_paths.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    Some(ObservedCompositionIdentity {
        identity: CompositionIdentity {
            source_digest: Digest::sha256_bytes(source.as_bytes()),
            dependency_digest: lockfile_digest.clone(),
            interface_digest,
            configuration_digest,
            toolchain_digest: observed_toolchain_digest.clone(),
            lockfile_digest,
            generated_input_digest: digest_paths(worktree, &generated_paths)?,
            environment_digest,
            verifier_digest: observed_toolchain_digest,
            command_digest: composition_commands_digest(&input.commands),
        },
        runtime_toolchain_digest,
    })
}

fn observe_runtime_toolchain_digest(
    worktree: &Path,
    environment: &BTreeMap<String, String>,
) -> Option<Digest> {
    let home = environment
        .get("HOME")
        .or_else(|| environment.get("USERPROFILE"))?;
    let home = fs::canonicalize(home).ok()?;
    if !home.is_dir() {
        return None;
    }

    let mut cargo_configuration = Vec::new();
    for path in [".cargo/config", ".cargo/config.toml"] {
        let bytes = read_optional_regular_file_beneath(&home, Path::new(path))?;
        cargo_configuration.push((path, bytes.as_deref().map(Digest::sha256_bytes)));
    }

    let workspace_channel = read_workspace_toolchain_channel(worktree)?;
    let rustup_home = home.join(".rustup");
    let rustup_metadata = match fs::symlink_metadata(&rustup_home) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if workspace_channel.is_some() {
                return None;
            }
            return digest_serialized(&("no-rustup-install", cargo_configuration));
        }
        Err(_) => return None,
    };
    if rustup_metadata.file_type().is_symlink() || !rustup_metadata.is_dir() {
        return None;
    }

    let settings = read_regular_file_beneath(&home, Path::new(".rustup/settings.toml"))?;
    let default_channel = parse_rustup_default_toolchain(&settings)?;
    let requested_channel = workspace_channel.unwrap_or(default_channel);
    if !is_safe_toolchain_channel(&requested_channel) {
        return None;
    }

    let rustup_home = fs::canonicalize(rustup_home).ok()?;
    if !rustup_home.starts_with(&home) {
        return None;
    }
    let installed_toolchain = resolve_rustup_toolchain_directory(&rustup_home, &requested_channel)?;
    let toolchain_root = PathBuf::from("toolchains").join(&installed_toolchain);
    let bin_root = toolchain_root.join("bin");
    let rustlib_root = toolchain_root.join("lib/rustlib");
    if !require_real_directory_beneath(&rustup_home, &bin_root)?
        || !require_real_directory_beneath(&rustup_home, &rustlib_root)?
    {
        return None;
    }

    let mut installed_files = Vec::new();
    let bin_path = rustup_home.join(&bin_root);
    let mut bin_entries = fs::read_dir(&bin_path)
        .ok()?
        .map(|entry| entry.ok())
        .collect::<Option<Vec<_>>>()?;
    bin_entries.sort_by_key(|entry| entry.file_name());
    if bin_entries.is_empty() {
        return None;
    }
    for entry in bin_entries {
        let file_type = entry.file_type().ok()?;
        if file_type.is_symlink() || !file_type.is_file() {
            return None;
        }
        let name = entry.file_name().into_string().ok()?;
        let relative = bin_root.join(&name);
        let bytes = read_regular_file_beneath(&rustup_home, &relative)?;
        installed_files.push((
            relative.to_string_lossy().into_owned(),
            Digest::sha256_bytes(&bytes),
        ));
    }

    let rustlib_path = rustup_home.join(&rustlib_root);
    let mut rustlib_entries = fs::read_dir(&rustlib_path)
        .ok()?
        .map(|entry| entry.ok())
        .collect::<Option<Vec<_>>>()?;
    rustlib_entries.sort_by_key(|entry| entry.file_name());
    let mut has_components = false;
    let mut has_manifest = false;
    for entry in rustlib_entries {
        let file_type = entry.file_type().ok()?;
        if file_type.is_dir() {
            continue;
        }
        if file_type.is_symlink() || !file_type.is_file() {
            return None;
        }
        let name = entry.file_name().into_string().ok()?;
        if name == "components" {
            has_components = true;
        }
        if name.starts_with("manifest-") {
            has_manifest = true;
        }
        let relative = rustlib_root.join(&name);
        let bytes = read_regular_file_beneath(&rustup_home, &relative)?;
        installed_files.push((
            relative.to_string_lossy().into_owned(),
            Digest::sha256_bytes(&bytes),
        ));
    }
    if !has_components || !has_manifest {
        return None;
    }

    let settings_digest = Digest::sha256_bytes(&settings);
    digest_serialized(&(
        "rustup-toolchain-v1",
        cargo_configuration,
        settings_digest,
        requested_channel,
        installed_toolchain,
        installed_files,
    ))
}

fn read_optional_regular_file_beneath(root: &Path, relative: &Path) -> Option<Option<Vec<u8>>> {
    match fs::symlink_metadata(root.join(relative)) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return None;
            }
            read_regular_file_beneath(root, relative).map(Some)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Some(None),
        Err(_) => None,
    }
}

fn read_workspace_toolchain_channel(worktree: &Path) -> Option<Option<String>> {
    let toml = read_optional_regular_file_beneath(worktree, Path::new("rust-toolchain.toml"))?;
    let plain = read_optional_regular_file_beneath(worktree, Path::new("rust-toolchain"))?;
    match (toml, plain) {
        (Some(toml), Some(plain)) => {
            let toml = parse_toolchain_toml_channel(&toml)?;
            let plain = parse_toolchain_file_channel(&plain)?;
            if toml == plain {
                Some(Some(toml))
            } else {
                None
            }
        }
        (Some(toml), None) => Some(Some(parse_toolchain_toml_channel(&toml)?)),
        (None, Some(plain)) => Some(Some(parse_toolchain_file_channel(&plain)?)),
        (None, None) => Some(None),
    }
}

fn parse_toolchain_toml_channel(bytes: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(bytes).ok()?;
    let mut section = "";
    let mut channel = None;
    for raw_line in text.lines() {
        let line = strip_toml_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line.strip_prefix('[')?.strip_suffix(']')?.trim();
            continue;
        }
        if section != "toolchain" {
            continue;
        }
        let (key, value) = line.split_once('=')?;
        if key.trim() == "channel" {
            if channel.is_some() {
                return None;
            }
            channel = Some(parse_toml_channel_string(value.trim())?);
        }
    }
    channel
}

fn parse_toolchain_file_channel(bytes: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(bytes).ok()?;
    let mut channel = None;
    for raw_line in text.lines() {
        let line = strip_toml_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if channel.is_some() {
            return None;
        }
        channel = Some(line.to_owned());
    }
    let channel = channel?;
    is_safe_toolchain_channel(&channel).then_some(channel)
}

fn parse_rustup_default_toolchain(bytes: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(bytes).ok()?;
    let mut section = "";
    let mut default_channel = None;
    for raw_line in text.lines() {
        let line = strip_toml_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line.strip_prefix('[')?.strip_suffix(']')?.trim();
            if section != "overrides" {
                return None;
            }
            continue;
        }
        if section == "overrides" {
            return None;
        }
        let (key, value) = line.split_once('=')?;
        match key.trim() {
            "default_toolchain" => {
                if default_channel.is_some() {
                    return None;
                }
                default_channel = Some(parse_toml_channel_string(value.trim())?);
            }
            "profile" => {
                parse_toml_channel_string(value.trim())?;
            }
            "version" => {
                let value = value.trim();
                if value.parse::<u64>().is_err() {
                    parse_toml_channel_string(value)?;
                }
            }
            _ => return None,
        }
    }
    let channel = default_channel?;
    is_safe_toolchain_channel(&channel).then_some(channel)
}

fn parse_toml_channel_string(value: &str) -> Option<String> {
    let value = value.strip_prefix('"')?;
    let end = value.find('"')?;
    if !value[end + 1..].trim().is_empty() {
        return None;
    }
    let channel = value[..end].to_owned();
    is_safe_toolchain_channel(&channel).then_some(channel)
}

fn strip_toml_comment(line: &str) -> &str {
    let mut quoted = false;
    let mut escaped = false;
    for (index, character) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if quoted && character == '\\' {
            escaped = true;
            continue;
        }
        if character == '"' {
            quoted = !quoted;
        } else if character == '#' && !quoted {
            return &line[..index];
        }
    }
    line
}

fn is_safe_toolchain_channel(channel: &str) -> bool {
    !channel.is_empty()
        && channel != "."
        && channel != ".."
        && channel
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'+' | b'-' | b'_'))
}

fn resolve_rustup_toolchain_directory(rustup_home: &Path, channel: &str) -> Option<String> {
    let toolchains = Path::new("toolchains");
    if !require_real_directory_beneath(rustup_home, toolchains)? {
        return None;
    }
    let mut exact = None;
    let mut aliases = Vec::new();
    for entry in fs::read_dir(rustup_home.join(toolchains)).ok()? {
        let entry = entry.ok()?;
        let name = entry.file_name().into_string().ok()?;
        let is_exact = name == channel;
        if !is_exact && !name.starts_with(&format!("{channel}-")) {
            continue;
        }
        let file_type = entry.file_type().ok()?;
        if file_type.is_symlink() || !file_type.is_dir() {
            return None;
        }
        if is_exact {
            if exact.replace(name).is_some() {
                return None;
            }
        } else {
            aliases.push(name);
        }
    }
    if let Some(exact) = exact {
        Some(exact)
    } else if aliases.len() == 1 {
        aliases.pop()
    } else {
        None
    }
}

fn require_real_directory_beneath(root: &Path, relative: &Path) -> Option<bool> {
    let components = relative
        .components()
        .map(|component| match component {
            Component::Normal(value) => Some(value),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    if components.is_empty() {
        return None;
    }
    let mut path = root.to_path_buf();
    for component in components {
        path.push(component);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if !metadata.file_type().is_symlink() && metadata.is_dir() => {}
            Ok(_) => return None,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Some(false),
            Err(_) => return None,
        }
    }
    Some(true)
}

fn observed_node_identity(
    worktree: &Path,
    input: &CompositionInput,
    command: &CompositionCommand,
    execution_records: &[CompositionExecutionRecord],
    executable: &ResolvedExecutable,
    environment: &BTreeMap<String, String>,
    runtime_toolchain_digest: Option<&Digest>,
) -> Option<Digest> {
    let runtime_toolchain_digest = runtime_toolchain_digest?;
    let dependencies = command
        .depends_on
        .iter()
        .map(|dependency| {
            let record = execution_records.iter().find(|record| {
                record.node_id == *dependency
                    && record.passed
                    && !record.timed_out
                    && (record.spawned || record.reused)
            })?;
            Some((dependency, &record.identity_digest, &record.output_digest))
        })
        .collect::<Option<Vec<_>>>()?;
    let observed_paths = deterministic_command_read_paths(command, &executable.identity_path)?;
    let mut input_paths = command.input_paths.clone();
    input_paths.extend(observed_paths);
    input_paths.sort();
    input_paths.dedup();
    let paths_digest = digest_paths(worktree, &input_paths)?;
    let environment = digest_serialized(environment)?;
    let bytes = serde_json::to_vec(&(
        &command.node_id,
        &command.program,
        &command.args,
        &command.depends_on,
        dependencies,
        &command.environment,
        &command.covered_scenarios,
        &command.covered_constraints,
        input.timeout_seconds,
        &input.binding.target_branch,
        &input.binding.participant_work_items,
        &input.binding.contract_digests,
        &executable.digest,
        runtime_toolchain_digest,
        paths_digest,
        environment,
    ))
    .ok()?;
    Some(Digest::sha256_bytes(&bytes))
}

/// Return file reads only for commands whose read set follows directly from
/// their executable and argv. Shells, scripts, absolute paths, and command
/// wrappers remain executable but are never reused without a bounded read set.
fn deterministic_command_read_paths(
    command: &CompositionCommand,
    executable: &Path,
) -> Option<Vec<String>> {
    if !is_system_utility(executable) {
        return None;
    }
    match command.program.as_str() {
        "true" | "false" | "env" if command.args.is_empty() => Some(Vec::new()),
        "cat"
            if !command.args.is_empty()
                && command.args.iter().all(|argument| {
                    let path = Path::new(argument);
                    !argument.starts_with('-')
                        && !path.is_absolute()
                        && !path.as_os_str().is_empty()
                        && path
                            .components()
                            .all(|component| matches!(component, Component::Normal(_)))
                }) =>
        {
            Some(command.args.clone())
        }
        _ => None,
    }
}

fn is_system_utility(executable: &Path) -> bool {
    #[cfg(unix)]
    {
        executable.starts_with("/bin") || executable.starts_with("/usr/bin")
    }
    #[cfg(not(unix))]
    {
        let _ = executable;
        false
    }
}

fn digest_paths(root: &Path, paths: &[String]) -> Option<Digest> {
    let mut observed = Vec::new();
    for value in paths {
        let path = Path::new(value);
        if path.is_absolute()
            || path
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
            || path.as_os_str().is_empty()
        {
            return None;
        }
        let bytes = read_regular_file_beneath(root, path)?;
        observed.push((value, Digest::sha256_bytes(&bytes)));
    }
    digest_serialized(&observed)
}

fn read_regular_file_beneath(root: &Path, relative: &Path) -> Option<Vec<u8>> {
    read_regular_file_beneath_with_interleave(root, relative, |_| {})
}

fn read_regular_file_beneath_with_interleave<F>(
    root: &Path,
    relative: &Path,
    after_open: F,
) -> Option<Vec<u8>>
where
    F: FnOnce(&mut File),
{
    #[cfg(unix)]
    {
        use std::os::fd::{AsRawFd, FromRawFd};
        use std::os::unix::ffi::OsStrExt;

        let root = fs::canonicalize(root).ok()?;
        let root_handle = File::open(&root).ok()?;
        if !root_handle.metadata().ok()?.is_dir() {
            return None;
        }
        let components = relative
            .components()
            .map(|component| match component {
                Component::Normal(value) => Some(value),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        if components.is_empty() {
            return None;
        }
        let mut directories = vec![root_handle];
        for component in &components[..components.len() - 1] {
            let name = CString::new(component.as_bytes()).ok()?;
            // SAFETY: `name` is NUL-terminated and the parent descriptor is
            // held open in `directories`; O_NOFOLLOW rejects symlinked
            // ancestors rather than resolving them outside the worktree.
            let descriptor = unsafe {
                libc::openat(
                    directories.last()?.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_RDONLY | libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW,
                )
            };
            if descriptor < 0 {
                return None;
            }
            // SAFETY: `openat` returned a new owned descriptor.
            let directory = unsafe { File::from_raw_fd(descriptor) };
            if !directory.metadata().ok()?.is_dir() {
                return None;
            }
            directories.push(directory);
        }
        let name = CString::new(components.last()?.as_bytes()).ok()?;
        // SAFETY: the final component is opened relative to a held directory
        // descriptor, and O_NOFOLLOW prevents a final-component symlink.
        let descriptor = unsafe {
            libc::openat(
                directories.last()?.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK,
            )
        };
        if descriptor < 0 {
            return None;
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let mut file = unsafe { File::from_raw_fd(descriptor) };
        if !file.metadata().ok()?.is_file() {
            return None;
        }
        if !read_set_handles_still_match_paths(&root, &components, &directories, &file) {
            return None;
        }
        let mutation_observer = ReadSetMutationObserver::start(&directories)?;
        if !mutation_observer.verify_unchanged() {
            return None;
        }
        after_open(&mut file);
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).ok()?;
        (mutation_observer.verify_unchanged()
            && read_set_handles_still_match_paths(&root, &components, &directories, &file))
        .then_some(bytes)
    }
    #[cfg(not(unix))]
    {
        // This path currently lacks a platform-native handle-bound proof that
        // parent directories remain contained while the bytes are read. A
        // path-only check followed by File::open leaves a reparse/rename race,
        // so inputs on these platforms are deliberately non-reusable.
        let _ = (root, relative, after_open);
        None
    }
}

#[cfg(target_os = "linux")]
struct ReadSetMutationObserver {
    instance: std::os::fd::OwnedFd,
    _directories: Vec<File>,
}

#[cfg(target_os = "linux")]
impl ReadSetMutationObserver {
    fn start(directories: &[File]) -> Option<Self> {
        use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

        // SAFETY: inotify_init1 takes no pointers and returns a new descriptor
        // on success, which is immediately wrapped in OwnedFd.
        let raw_instance = unsafe { libc::inotify_init1(libc::IN_CLOEXEC | libc::IN_NONBLOCK) };
        if raw_instance < 0 {
            return None;
        }
        // SAFETY: `raw_instance` is a newly owned descriptor from inotify_init1.
        let instance = unsafe { OwnedFd::from_raw_fd(raw_instance) };
        let mut retained = Vec::with_capacity(directories.len());
        for directory in directories {
            let handle = directory.try_clone().ok()?;
            let descriptor_path =
                std::ffi::CString::new(format!("/proc/self/fd/{}", handle.as_raw_fd())).ok()?;
            let mask = libc::IN_MOVE_SELF | libc::IN_DELETE_SELF | libc::IN_UNMOUNT;
            // SAFETY: descriptor_path is NUL-terminated and instance is a live
            // inotify descriptor; the watch follows the held directory handle.
            if unsafe {
                libc::inotify_add_watch(instance.as_raw_fd(), descriptor_path.as_ptr(), mask)
            } < 0
            {
                return None;
            }
            retained.push(handle);
        }
        Some(Self {
            instance,
            _directories: retained,
        })
    }

    fn verify_unchanged(&self) -> bool {
        use std::os::fd::AsRawFd;

        let mut buffer = [0_u8; 4096];
        loop {
            // SAFETY: the buffer is writable for its full length and the
            // inotify descriptor remains owned by self for this call.
            let count = unsafe {
                libc::read(
                    self.instance.as_raw_fd(),
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                )
            };
            if count < 0 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return error.kind() == std::io::ErrorKind::WouldBlock;
            }
            return false;
        }
    }
}

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
struct ReadSetMutationObserver {
    queue: std::os::fd::OwnedFd,
    _directories: Vec<File>,
}

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
impl ReadSetMutationObserver {
    fn start(directories: &[File]) -> Option<Self> {
        use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

        // SAFETY: kqueue takes no pointers and returns a new descriptor on
        // success, which is immediately wrapped in OwnedFd.
        let raw_queue = unsafe { libc::kqueue() };
        if raw_queue < 0 {
            return None;
        }
        // SAFETY: `raw_queue` is a newly owned descriptor from kqueue.
        let queue = unsafe { OwnedFd::from_raw_fd(raw_queue) };
        let mut retained = Vec::with_capacity(directories.len());
        for directory in directories {
            let handle = directory.try_clone().ok()?;
            // SAFETY: zero is a valid initial representation for kevent before
            // its fields are populated below.
            let mut change: libc::kevent = unsafe { std::mem::zeroed() };
            change.ident = handle.as_raw_fd() as libc::uintptr_t;
            change.filter = libc::EVFILT_VNODE;
            change.flags = libc::EV_ADD | libc::EV_CLEAR;
            change.fflags = libc::NOTE_RENAME | libc::NOTE_DELETE;
            // SAFETY: change points to one initialized kevent and queue is live.
            if unsafe {
                libc::kevent(
                    queue.as_raw_fd(),
                    &change,
                    1,
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null(),
                )
            } < 0
            {
                return None;
            }
            retained.push(handle);
        }
        Some(Self {
            queue,
            _directories: retained,
        })
    }

    fn verify_unchanged(&self) -> bool {
        use std::os::fd::AsRawFd;

        // SAFETY: zero is a valid output buffer for the kernel to fill.
        let mut event: libc::kevent = unsafe { std::mem::zeroed() };
        let timeout = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        loop {
            // SAFETY: queue is live, event is writable, and timeout is valid.
            let result = unsafe {
                libc::kevent(
                    self.queue.as_raw_fd(),
                    std::ptr::null(),
                    0,
                    &mut event,
                    1,
                    &timeout,
                )
            };
            if result < 0 {
                if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return false;
            }
            return result == 0;
        }
    }
}

#[cfg(all(
    unix,
    not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))
))]
struct ReadSetMutationObserver;

#[cfg(all(
    unix,
    not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))
))]
impl ReadSetMutationObserver {
    fn start(_directories: &[File]) -> Option<Self> {
        None
    }

    fn verify_unchanged(&self) -> bool {
        false
    }
}

#[cfg(unix)]
fn read_set_handles_still_match_paths(
    canonical_root: &Path,
    components: &[&std::ffi::OsStr],
    directories: &[File],
    opened_file: &File,
) -> bool {
    use std::os::unix::fs::MetadataExt;

    let Some(root_handle) = directories.first() else {
        return false;
    };
    let Ok(root_path) = fs::canonicalize(canonical_root) else {
        return false;
    };
    let Ok(root_display) = File::open(&root_path) else {
        return false;
    };
    let Ok(root_metadata) = root_handle.metadata() else {
        return false;
    };
    let Ok(root_display_metadata) = root_display.metadata() else {
        return false;
    };
    if !root_metadata.is_dir()
        || !root_display_metadata.is_dir()
        || root_metadata.dev() != root_display_metadata.dev()
        || root_metadata.ino() != root_display_metadata.ino()
        || directories.len() != components.len()
    {
        return false;
    }

    let mut display = root_path.clone();
    for (index, component) in components[..components.len() - 1].iter().enumerate() {
        display.push(component);
        let Ok(canonical) = fs::canonicalize(&display) else {
            return false;
        };
        if !canonical.starts_with(&root_path) {
            return false;
        }
        let Ok(displayed_directory) = File::open(&display) else {
            return false;
        };
        let Ok(opened_metadata) = directories[index + 1].metadata() else {
            return false;
        };
        let Ok(displayed_metadata) = displayed_directory.metadata() else {
            return false;
        };
        if !opened_metadata.is_dir()
            || !displayed_metadata.is_dir()
            || opened_metadata.dev() != displayed_metadata.dev()
            || opened_metadata.ino() != displayed_metadata.ino()
        {
            return false;
        }
    }

    let Some(leaf) = components.last() else {
        return false;
    };
    display.push(leaf);
    let Ok(canonical_file) = fs::canonicalize(&display) else {
        return false;
    };
    if !canonical_file.starts_with(&root_path) {
        return false;
    }
    let Ok(path_metadata) = fs::symlink_metadata(&display) else {
        return false;
    };
    if path_metadata.file_type().is_symlink() || !path_metadata.is_file() {
        return false;
    }
    let Ok(displayed_file) = File::open(&display) else {
        return false;
    };
    let Ok(opened_metadata) = opened_file.metadata() else {
        return false;
    };
    let Ok(displayed_metadata) = displayed_file.metadata() else {
        return false;
    };
    opened_metadata.is_file()
        && displayed_metadata.is_file()
        && opened_metadata.dev() == displayed_metadata.dev()
        && opened_metadata.ino() == displayed_metadata.ino()
}

fn observed_environment_digest(commands: &[CompositionCommand]) -> Option<Digest> {
    let environments = commands
        .iter()
        .map(|command| Some((&command.node_id, controlled_command_environment(command)?)))
        .collect::<Option<Vec<_>>>()?;
    digest_serialized(&environments)
}

#[derive(Clone, Debug)]
struct ResolvedExecutable {
    launch_path: PathBuf,
    identity_path: PathBuf,
    digest: Digest,
}

fn resolve_executable(
    worktree: &Path,
    program: &str,
    environment: &BTreeMap<String, String>,
) -> Option<ResolvedExecutable> {
    let program_path = Path::new(program);
    let candidate = if program_path.is_absolute() {
        program_path.to_path_buf()
    } else if program_path.components().count() > 1 {
        worktree.join(program_path)
    } else {
        let path = OsString::from(environment.get("PATH")?);
        let directories = std::env::split_paths(&path).collect::<Vec<_>>();
        if directories.iter().any(|directory| !directory.is_absolute()) {
            return None;
        }
        directories
            .into_iter()
            .flat_map(|directory| executable_candidates(&directory, program))
            .find(|path| is_executable_file(path))?
    };
    if !is_executable_file(&candidate) {
        return None;
    }
    let identity_path = fs::canonicalize(&candidate).ok()?;
    let canonical_worktree = fs::canonicalize(worktree).ok()?;
    let trusted_roots = trusted_executable_directories();
    let inside_worktree = identity_path.starts_with(&canonical_worktree);
    let inside_trusted_root = trusted_roots
        .iter()
        .any(|directory| identity_path.starts_with(directory));
    if !inside_worktree && !inside_trusted_root {
        return None;
    }
    let digest = digest_executable(&identity_path)?;
    Some(ResolvedExecutable {
        launch_path: candidate,
        identity_path,
        digest,
    })
}

fn executable_candidates(directory: &Path, program: &str) -> Vec<PathBuf> {
    let candidate = directory.join(program);
    #[cfg(windows)]
    if candidate.extension().is_none() {
        return vec![
            candidate.with_extension("exe"),
            candidate.with_extension("com"),
        ];
    }
    vec![candidate]
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

fn trusted_executable_directories() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    #[cfg(unix)]
    {
        if let Some(home) = std::env::var_os("HOME") {
            candidates.push(PathBuf::from(home).join(".cargo/bin"));
        }
        candidates.extend(
            [
                "/usr/bin",
                "/bin",
                "/usr/local/bin",
                "/opt/homebrew/bin",
                "/opt/local/bin",
            ]
            .into_iter()
            .map(PathBuf::from),
        );
    }
    #[cfg(windows)]
    {
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            candidates.push(PathBuf::from(profile).join(".cargo/bin"));
        }
        if let Some(system_root) = std::env::var_os("SystemRoot") {
            candidates.push(PathBuf::from(&system_root).join("System32"));
            candidates.push(PathBuf::from(system_root));
        }
    }
    let mut result = Vec::new();
    for candidate in candidates {
        if let Ok(directory) = fs::canonicalize(candidate)
            && directory.is_dir()
            && !result.contains(&directory)
        {
            result.push(directory);
        }
    }
    result
}

fn controlled_command_environment(
    command: &CompositionCommand,
) -> Option<BTreeMap<String, String>> {
    if command
        .environment
        .keys()
        .any(|key| is_runtime_controlled_environment_key(key))
    {
        return None;
    }
    let directories = trusted_executable_directories();
    if directories.is_empty() {
        return None;
    }
    let mut environment = BTreeMap::new();
    environment.insert(
        "PATH".into(),
        std::env::join_paths(&directories)
            .ok()?
            .into_string()
            .ok()?,
    );
    #[cfg(unix)]
    if let Some(home) = std::env::var_os("HOME") {
        let home = fs::canonicalize(home).ok()?;
        environment.insert("HOME".into(), home.to_string_lossy().into_owned());
    }
    #[cfg(windows)]
    {
        let system_root = fs::canonicalize(std::env::var_os("SystemRoot")?).ok()?;
        environment.insert(
            "SystemRoot".into(),
            system_root.to_string_lossy().into_owned(),
        );
        environment.insert("WINDIR".into(), system_root.to_string_lossy().into_owned());
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            let profile = fs::canonicalize(profile).ok()?;
            environment.insert("USERPROFILE".into(), profile.to_string_lossy().into_owned());
        }
    }
    let temporary = fs::canonicalize(std::env::temp_dir()).ok()?;
    #[cfg(unix)]
    environment.insert("TMPDIR".into(), temporary.to_string_lossy().into_owned());
    #[cfg(windows)]
    {
        environment.insert("TEMP".into(), temporary.to_string_lossy().into_owned());
        environment.insert("TMP".into(), temporary.to_string_lossy().into_owned());
    }
    environment.insert("LANG".into(), "C".into());
    environment.insert("LC_ALL".into(), "C".into());
    environment.extend(command.environment.clone());
    Some(environment)
}

fn is_runtime_controlled_environment_key(key: &str) -> bool {
    let key = key.to_ascii_uppercase();
    matches!(
        key.as_str(),
        "PATH"
            | "HOME"
            | "USERPROFILE"
            | "SYSTEMROOT"
            | "WINDIR"
            | "CARGO_HOME"
            | "RUSTUP_HOME"
            | "RUSTUP_TOOLCHAIN"
            | "RUSTC"
            | "RUSTDOC"
            | "RUSTFLAGS"
            | "CARGO_ENCODED_RUSTFLAGS"
            | "CARGO_BUILD_TARGET"
            | "CARGO_TARGET_DIR"
            | "RUSTC_WRAPPER"
            | "RUSTC_WORKSPACE_WRAPPER"
            | "RUSTC_BOOTSTRAP"
            | "DYLD_INSERT_LIBRARIES"
            | "DYLD_LIBRARY_PATH"
            | "DYLD_FRAMEWORK_PATH"
            | "LD_PRELOAD"
            | "LD_LIBRARY_PATH"
            | "LIBPATH"
            | "SHLIB_PATH"
    ) || key.starts_with("DYLD_")
        || key.starts_with("LD_")
}

fn digest_executable(executable: &Path) -> Option<Digest> {
    let bytes = fs::read(executable).ok()?;
    Some(Digest::sha256_bytes(&bytes))
}

fn is_lockfile(path: &str) -> bool {
    let name = Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    matches!(
        name,
        "Cargo.lock"
            | "package-lock.json"
            | "pnpm-lock.yaml"
            | "yarn.lock"
            | "poetry.lock"
            | "uv.lock"
            | "Gemfile.lock"
            | "go.sum"
    )
}

fn is_configuration_file(path: &str) -> bool {
    let path = Path::new(path);
    matches!(
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default(),
        "Cargo.toml" | "rust-toolchain" | "rust-toolchain.toml" | "Makefile" | "Dockerfile"
    ) || matches!(path.to_str(), Some(".cargo/config" | ".cargo/config.toml"))
}

fn git_text(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
}

fn digest_serialized(value: &impl Serialize) -> Option<Digest> {
    Some(Digest::sha256_bytes(&serde_json::to_vec(value).ok()?))
}

fn persist_attempt(state_dir: &Path, attempt: &CompositionAttempt) -> Result<(), CompositionError> {
    fs::create_dir_all(state_dir).map_err(|source| CompositionError::Io {
        path: state_dir.to_path_buf(),
        source,
    })?;
    let path = state_dir.join(format!("{}.json", attempt.attempt_id));
    let temporary = state_dir.join(format!(".{}.{}.tmp", attempt.attempt_id, now_unix_nanos()));
    let bytes = serde_json::to_vec_pretty(attempt)
        .map_err(|error| CompositionError::Serialization(error.to_string()))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|source| CompositionError::Io {
            path: temporary.clone(),
            source,
        })?;
    file.write_all(&bytes)
        .map_err(|source| CompositionError::Io {
            path: temporary.clone(),
            source,
        })?;
    file.sync_all().map_err(|source| CompositionError::Io {
        path: temporary.clone(),
        source,
    })?;
    fs::rename(&temporary, &path).map_err(|source| CompositionError::Io { path, source })
}

fn new_attempt_id(input: &CompositionInput, timestamp: u128) -> String {
    let identity =
        serde_json::to_vec(&(&input.binding, &input.identity, timestamp)).unwrap_or_default();
    format!("composition-{}", Digest::sha256_bytes(&identity))
}

fn now_unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}

fn unique_composition_parent() -> PathBuf {
    std::env::temp_dir().join(format!(
        "ai-cockpit-composition-{}-{}",
        std::process::id(),
        now_unix_nanos()
    ))
}

fn create_private_composition_parent(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        let mut builder = fs::DirBuilder::new();
        builder.mode(0o700).create(path)
    }
    #[cfg(not(unix))]
    {
        fs::create_dir(path)
    }
}

struct CompositionWorktreeGuard {
    repository_root: PathBuf,
    worktree: PathBuf,
    parent: PathBuf,
    registered: bool,
    armed: bool,
}

impl CompositionWorktreeGuard {
    fn new(repository_root: PathBuf, worktree: PathBuf, parent: PathBuf) -> Self {
        Self {
            repository_root,
            worktree,
            parent,
            registered: false,
            armed: true,
        }
    }

    fn mark_registered(&mut self) {
        self.registered = true;
    }

    fn cleanup(&mut self) -> CompositionCleanup {
        self.armed = false;
        cleanup_worktree(
            &self.repository_root,
            &self.worktree,
            &self.parent,
            self.registered,
        )
    }

    fn preserve(&mut self) {
        self.armed = false;
    }
}

impl Drop for CompositionWorktreeGuard {
    fn drop(&mut self) {
        if self.armed {
            let _ = cleanup_worktree(
                &self.repository_root,
                &self.worktree,
                &self.parent,
                self.registered,
            );
        }
    }
}

fn cleanup_worktree(
    repository_root: &Path,
    worktree: &Path,
    parent: &Path,
    worktree_registered: bool,
) -> CompositionCleanup {
    let mut errors = Vec::new();
    let mut remove_parent = !worktree_registered;
    if worktree_registered {
        let removed = git_command(
            repository_root,
            &[
                "worktree",
                "remove",
                "--force",
                worktree.to_str().unwrap_or_default(),
            ],
        );
        if removed.success {
            remove_parent = true;
        } else {
            errors.push(format!(
                "git_worktree_remove_failed:{}",
                bounded(&removed.stderr)
            ));
        }
    }
    if remove_parent
        && let Err(error) = fs::remove_dir_all(parent)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        errors.push(format!("parent_remove_failed:{error}"));
    }
    CompositionCleanup {
        attempted: true,
        removed: errors.is_empty(),
        error: (!errors.is_empty()).then(|| errors.join(";")),
    }
}

struct GitOutput {
    success: bool,
    stdout: String,
    stderr: String,
}

fn git_command(root: &Path, args: &[&str]) -> GitOutput {
    match Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(args)
        .output()
    {
        Ok(output) => GitOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: bounded_bytes(&output.stderr),
        },
        Err(error) => GitOutput {
            success: false,
            stdout: String::new(),
            stderr: error.to_string(),
        },
    }
}

fn bounded(value: &str) -> String {
    value.chars().take(MAX_OUTPUT_BYTES).collect()
}

fn bounded_bytes(value: &[u8]) -> String {
    String::from_utf8_lossy(&value[..value.len().min(MAX_OUTPUT_BYTES)]).into_owned()
}

fn decode_hex_bounded(value: &str) -> String {
    let mut decoded = Vec::with_capacity(value.len() / 2);
    for pair in value.as_bytes().chunks(2) {
        if pair.len() != 2 {
            break;
        }
        let Some(high) = hex_value(pair[0]) else {
            return String::new();
        };
        let Some(low) = hex_value(pair[1]) else {
            return String::new();
        };
        decoded.push((high << 4) | low);
        if decoded.len() >= MAX_OUTPUT_BYTES {
            break;
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(all(
    test,
    any(
        target_os = "linux",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )
))]
mod read_set_containment_tests {
    use super::read_regular_file_beneath_with_interleave;
    use std::fs;
    use std::io::Read;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TemporaryDirectories(Vec<PathBuf>);

    impl Drop for TemporaryDirectories {
        fn drop(&mut self) {
            for path in &self.0 {
                let _ = fs::remove_dir_all(path);
            }
        }
    }

    #[test]
    fn moving_an_open_read_set_parent_outside_during_read_is_not_trusted() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let base = std::env::temp_dir().join(format!(
            "cockpit-read-set-move-{}-{nonce}",
            std::process::id()
        ));
        let root = base.join("repository");
        let outside = base.join("outside");
        fs::create_dir_all(root.join("inputs")).expect("create in-repository input directory");
        fs::create_dir_all(&outside).expect("create outside directory");
        let _cleanup = TemporaryDirectories(vec![base.clone()]);
        fs::write(root.join("inputs/input.txt"), b"opened-before-move\n")
            .expect("write original input");
        let moved = outside.join("moved-inputs");
        let replacement = outside.join("replacement-inputs");
        let mut bytes_read_while_outside = Vec::new();

        let observed = read_regular_file_beneath_with_interleave(
            &root,
            Path::new("inputs/input.txt"),
            |opened_file| {
                fs::rename(root.join("inputs"), &moved)
                    .expect("move opened parent outside the repository");
                fs::create_dir(root.join("inputs")).expect("replace original parent path");
                fs::write(root.join("inputs/input.txt"), b"replacement-inside\n")
                    .expect("write replacement input");
                opened_file
                    .read_to_end(&mut bytes_read_while_outside)
                    .expect("read from the opened handle while its directory is outside");
                fs::rename(root.join("inputs"), replacement)
                    .expect("move replacement away from the registered path");
                fs::rename(&moved, root.join("inputs"))
                    .expect("restore the original directory after the outside read");
            },
        );

        assert_eq!(
            bytes_read_while_outside, b"opened-before-move\n",
            "the interleaving must prove that bytes were actually read while the held parent was outside"
        );
        assert_eq!(
            observed, None,
            "bytes read through a directory moved outside the repository must not enter the reuse identity"
        );
    }
}
