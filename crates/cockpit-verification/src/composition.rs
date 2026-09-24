use crate::{VerificationCommand, VerificationReusePolicy, execute_bounded};
use cockpit_core::Digest;
use cockpit_protocol::{CompositionBinding, RuntimeCapabilityError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const COMPOSITION_SCHEMA_VERSION: u32 = 1;
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

    let unique = unique_composition_parent();
    let worktree = unique.join("composition");
    fs::create_dir_all(&unique).map_err(|source| CompositionError::Io {
        path: unique.clone(),
        source,
    })?;
    attempt.isolated_worktree = worktree.to_string_lossy().into_owned();
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
        attempt.cleanup = Some(cleanup_worktree(
            &input.repository_root,
            &worktree,
            &unique,
            false,
        ));
        persist_attempt(&input.state_dir, &attempt)?;
        return Ok(attempt);
    }

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
            attempt.cleanup = Some(cleanup_worktree(
                &input.repository_root,
                &worktree,
                &unique,
                true,
            ));
            persist_attempt(&input.state_dir, &attempt)?;
            return Ok(attempt);
        }
    }

    let observed_identity = observe_composition_identity(&worktree, &input);
    let identity_observed = observed_identity.is_some();
    if let Some(identity) = observed_identity {
        input.identity = identity;
        attempt.identity = input.identity.clone();
    }
    let predecessor_id = predecessor
        .as_ref()
        .map(|previous| previous.attempt_id.as_str());
    let mut node_identities_observed = true;
    let mut all_nodes_reused = predecessor.is_some() && identity_observed;
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
        let current_identity =
            observed_node_identity(&worktree, &input, command, &attempt.execution_records);
        if current_identity.is_none() {
            node_identities_observed = false;
        }
        let reusable = if identity_observed
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
        let record = execute_node(&worktree, command, &input, current_identity);
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
    attempt.cleanup = Some(cleanup_worktree(
        &input.repository_root,
        &worktree,
        &unique,
        true,
    ));
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
    identity_digest: Option<Digest>,
) -> CompositionExecutionRecord {
    let identity_digest = identity_digest
        .unwrap_or_else(|| Digest::sha256_bytes(b"unknown-composition-node-identity"));
    let verification_command = VerificationCommand::new(
        &command.node_id,
        &command.program,
        command.args.clone(),
        VerificationReusePolicy::NeverReuse,
    )
    .with_current_dir(worktree)
    .with_environment(
        command
            .environment
            .iter()
            .map(|(key, value)| (OsString::from(key), OsString::from(value)))
            .collect(),
    )
    .with_timeout_seconds(input.timeout_seconds);
    match execute_bounded(vec![verification_command], 1) {
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

fn observe_composition_identity(
    worktree: &Path,
    input: &CompositionInput,
) -> Option<CompositionIdentity> {
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
    let toolchain = input
        .commands
        .iter()
        .map(|command| executable_digest(worktree, &command.program, &command.environment))
        .collect::<Option<Vec<_>>>()?;
    let environment_digest = observed_environment_digest(&input.commands)?;
    let generated_paths = input
        .commands
        .iter()
        .flat_map(|command| command.input_paths.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    Some(CompositionIdentity {
        source_digest: Digest::sha256_bytes(source.as_bytes()),
        dependency_digest: lockfile_digest.clone(),
        interface_digest,
        configuration_digest,
        toolchain_digest: digest_serialized(&toolchain)?,
        lockfile_digest,
        generated_input_digest: digest_paths(worktree, &generated_paths)?,
        environment_digest,
        verifier_digest: digest_serialized(&toolchain)?,
        command_digest: composition_commands_digest(&input.commands),
    })
}

fn observed_node_identity(
    worktree: &Path,
    input: &CompositionInput,
    command: &CompositionCommand,
    execution_records: &[CompositionExecutionRecord],
) -> Option<Digest> {
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
    let executable_path = resolve_executable(worktree, &command.program, &command.environment)?;
    let executable = digest_executable(&executable_path)?;
    let observed_paths = deterministic_command_read_paths(command, &executable_path)?;
    let mut input_paths = command.input_paths.clone();
    input_paths.extend(observed_paths);
    input_paths.sort();
    input_paths.dedup();
    let paths_digest = digest_paths(worktree, &input_paths)?;
    let environment = observed_environment_digest(std::slice::from_ref(command))?;
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
        executable,
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
            || path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
            || path.as_os_str().is_empty()
        {
            return None;
        }
        let candidate = root.join(path);
        let metadata = fs::symlink_metadata(&candidate).ok()?;
        if !metadata.file_type().is_file() {
            return None;
        }
        let bytes = fs::read(&candidate).ok()?;
        observed.push((value, Digest::sha256_bytes(&bytes)));
    }
    digest_serialized(&observed)
}

fn observed_environment_digest(commands: &[CompositionCommand]) -> Option<Digest> {
    let mut environment = BTreeMap::<String, String>::new();
    for (key, value) in std::env::vars_os() {
        environment.insert(key.into_string().ok()?, value.into_string().ok()?);
    }
    let overlays = commands
        .iter()
        .map(|command| (&command.node_id, &command.environment))
        .collect::<Vec<_>>();
    let bytes = serde_json::to_vec(&(environment, overlays)).ok()?;
    Some(Digest::sha256_bytes(&bytes))
}

fn resolve_executable(
    worktree: &Path,
    program: &str,
    environment: &BTreeMap<String, String>,
) -> Option<PathBuf> {
    let program_path = Path::new(program);
    let candidate = if program_path.is_absolute() {
        program_path.to_path_buf()
    } else if program_path.components().count() > 1 {
        worktree.join(program_path)
    } else {
        let path = environment
            .get("PATH")
            .map(OsString::from)
            .or_else(|| std::env::var_os("PATH"))?;
        let directories = std::env::split_paths(&path).collect::<Vec<_>>();
        if directories.iter().any(|directory| !directory.is_absolute()) {
            // Relative PATH entries can be resolved against different working
            // directories by the parent and child launch paths. Do not claim
            // an executable identity when that ambiguity exists.
            return None;
        }
        directories
            .into_iter()
            .map(|directory| directory.join(program))
            .find(|path| path.is_file())?
    };
    let canonical = fs::canonicalize(candidate).ok()?;
    let metadata = fs::metadata(&canonical).ok()?;
    if !metadata.is_file() {
        return None;
    }
    Some(canonical)
}

fn digest_executable(executable: &Path) -> Option<Digest> {
    let bytes = fs::read(executable).ok()?;
    Some(Digest::sha256_bytes(&bytes))
}

fn executable_digest(
    worktree: &Path,
    program: &str,
    environment: &BTreeMap<String, String>,
) -> Option<Digest> {
    let executable = resolve_executable(worktree, program, environment)?;
    digest_executable(&executable)
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
    matches!(
        Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default(),
        "Cargo.toml" | "rust-toolchain" | "rust-toolchain.toml" | "Makefile" | "Dockerfile"
    )
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

fn cleanup_worktree(
    repository_root: &Path,
    worktree: &Path,
    parent: &Path,
    worktree_registered: bool,
) -> CompositionCleanup {
    let mut errors = Vec::new();
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
        if !removed.success {
            errors.push(format!(
                "git_worktree_remove_failed:{}",
                bounded(&removed.stderr)
            ));
        }
    }
    if let Err(error) = fs::remove_dir_all(parent)
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
            stderr: bounded_bytes(&output.stderr),
        },
        Err(error) => GitOutput {
            success: false,
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
