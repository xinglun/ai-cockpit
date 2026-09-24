use crate::{VerificationCommand, VerificationReusePolicy, execute_bounded};
use cockpit_core::Digest;
use cockpit_protocol::{CompositionBinding, RuntimeCapabilityError};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompositionInput {
    pub repository_root: PathBuf,
    pub state_dir: PathBuf,
    pub binding: CompositionBinding,
    pub identity: CompositionIdentity,
    pub commands: Vec<CompositionCommand>,
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
    input.binding.verifier.validate_candidate()?;
    validate_binding(&input.binding)?;

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

    let expected_command_digest = composition_commands_digest(&input.commands);
    if input.identity.command_digest != expected_command_digest {
        fail_without_worktree(
            &input.state_dir,
            &mut attempt,
            "composition_identity_mismatch:commands",
        )?;
        return Ok(attempt);
    }
    if input.commands.is_empty() {
        fail_without_worktree(&input.state_dir, &mut attempt, "required_checks_empty")?;
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

    let predecessor_id = predecessor
        .as_ref()
        .map(|previous| previous.attempt_id.as_str());
    let reusable = input
        .commands
        .iter()
        .map(|command| {
            predecessor.as_ref().and_then(|previous| {
                previous
                    .execution_records
                    .iter()
                    .find(|record| can_reuse_node(record, &input, command))
                    .map(|record| reused_record(record, command, previous.attempt_id.as_str()))
            })
        })
        .collect::<Vec<_>>();

    if reusable.iter().all(Option::is_some) {
        attempt.execution_records = reusable.into_iter().flatten().collect();
        attempt.reuse_decision = ReuseDecision {
            kind: ReuseDecisionKind::Reuse,
            reason: "all required checks match durable predecessor identities".into(),
            predecessor_attempt_id: predecessor_id.map(str::to_owned),
        };
        attempt.passed = true;
        attempt.failure = None;
        attempt.cleanup = Some(CompositionCleanup {
            attempted: false,
            removed: true,
            error: None,
        });
        persist_attempt(&input.state_dir, &attempt)?;
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

    for (index, command) in input.commands.iter().enumerate() {
        if let Some(record) = reusable[index].clone() {
            attempt.execution_records.push(record);
            persist_attempt(&input.state_dir, &attempt)?;
            continue;
        }
        let record = execute_node(&worktree, command, &input);
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
    if previous.passed
        && previous.failure.is_none()
        && previous.binding == current.binding
        && previous.identity == current.identity
    {
        return ReuseDecision {
            kind: ReuseDecisionKind::Reuse,
            reason: "exact composition binding and verification identity match".into(),
            predecessor_attempt_id: Some(previous.attempt_id.clone()),
        };
    }
    ReuseDecision {
        kind: ReuseDecisionKind::Execute,
        reason: "composition binding or verification identity changed".into(),
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
        if candidate.binding != *binding {
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

fn can_reuse_node(
    record: &CompositionExecutionRecord,
    input: &CompositionInput,
    command: &CompositionCommand,
) -> bool {
    record.node_id == command.node_id
        && record.passed
        && !record.timed_out
        && (record.spawned || record.reused)
        && record.identity_digest == node_identity_digest(&input.binding, &input.identity, command)
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
) -> CompositionExecutionRecord {
    let identity_digest = node_identity_digest(&input.binding, &input.identity, command);
    let verification_command = VerificationCommand::new(
        &command.node_id,
        &command.program,
        command.args.clone(),
        VerificationReusePolicy::NeverReuse,
    )
    .with_current_dir(worktree)
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

fn node_identity_digest(
    binding: &CompositionBinding,
    identity: &CompositionIdentity,
    command: &CompositionCommand,
) -> Digest {
    let mut identity_without_commands = identity.clone();
    identity_without_commands.command_digest = Digest::sha256_bytes(b"per-node-command-digest");
    let bytes = serde_json::to_vec(&(binding, identity_without_commands, command))
        .expect("composition node identity is serializable");
    Digest::sha256_bytes(&bytes)
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
