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
    pub spawned: bool,
    pub passed: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub output_digest: Digest,
    pub timed_out: bool,
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
}

fn composition_schema_version() -> u32 {
    COMPOSITION_SCHEMA_VERSION
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
    let attempt_id = new_attempt_id(&input);
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
        reuse_decision: ReuseDecision {
            kind: ReuseDecisionKind::Execute,
            reason: "fresh composition attempt".into(),
            predecessor_attempt_id: None,
        },
        passed: false,
        failure: None,
    };

    let expected_command_digest = composition_commands_digest(&input.commands);
    if input.identity.command_digest != expected_command_digest {
        attempt.failure = Some("composition_identity_mismatch:commands".into());
        persist_attempt(&input.state_dir, &attempt)?;
        return Ok(attempt);
    }

    if let Some(precondition) = input.preconditions.iter().find(|item| !item.satisfied) {
        attempt.failure = Some(format!(
            "precondition_failed:{}:{}",
            precondition.name, precondition.reason
        ));
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
        let _ = fs::remove_dir_all(&unique);
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
            cleanup_worktree(&input.repository_root, &worktree, &unique);
            persist_attempt(&input.state_dir, &attempt)?;
            return Ok(attempt);
        }
    }

    for command in &input.commands {
        let output = Command::new(&command.program)
            .args(&command.args)
            .current_dir(&worktree)
            .output();
        let record = match output {
            Ok(output) => {
                let stdout = bounded_bytes(&output.stdout);
                let stderr = bounded_bytes(&output.stderr);
                let output_digest = Digest::sha256_bytes(
                    [&output.stdout[..], &output.stderr[..]].concat().as_slice(),
                );
                CompositionExecutionRecord {
                    node_id: command.node_id.clone(),
                    program: command.program.clone(),
                    args: command.args.clone(),
                    spawned: true,
                    passed: output.status.success(),
                    exit_code: output.status.code(),
                    stdout,
                    stderr,
                    output_digest,
                    timed_out: false,
                }
            }
            Err(error) => CompositionExecutionRecord {
                node_id: command.node_id.clone(),
                program: command.program.clone(),
                args: command.args.clone(),
                spawned: false,
                passed: false,
                exit_code: None,
                stdout: String::new(),
                stderr: error.to_string(),
                output_digest: Digest::sha256_bytes(error.to_string().as_bytes()),
                timed_out: false,
            },
        };
        attempt.processes_spawned += usize::from(record.spawned);
        let passed = record.passed;
        let exit_code = record.exit_code;
        attempt.execution_records.push(record);
        if !passed {
            attempt.failure = Some(format!("command_failed:exit={exit_code:?}"));
            break;
        }
    }
    attempt.passed = attempt.failure.is_none() && attempt.processes_spawned == input.commands.len();
    cleanup_worktree(&input.repository_root, &worktree, &unique);
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
        .any(|record| !record.spawned || record.exit_code.is_none() || record.timed_out)
    {
        return ReuseDecision {
            kind: ReuseDecisionKind::Unknown,
            reason: "predecessor execution identity is incomplete".into(),
            predecessor_attempt_id: Some(previous.attempt_id.clone()),
        };
    }
    if previous.passed
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

fn persist_attempt(state_dir: &Path, attempt: &CompositionAttempt) -> Result<(), CompositionError> {
    fs::create_dir_all(state_dir).map_err(|source| CompositionError::Io {
        path: state_dir.to_path_buf(),
        source,
    })?;
    let path = state_dir.join(format!("{}.json", attempt.attempt_id));
    let temporary = state_dir.join(format!(".{}.tmp", attempt.attempt_id));
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

fn new_attempt_id(input: &CompositionInput) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let identity =
        serde_json::to_vec(&(&input.binding, &input.identity, timestamp)).unwrap_or_default();
    format!("composition-{}", Digest::sha256_bytes(&identity))
}

fn unique_composition_parent() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    std::env::temp_dir().join(format!(
        "ai-cockpit-composition-{}-{timestamp}",
        std::process::id()
    ))
}

fn cleanup_worktree(repository_root: &Path, worktree: &Path, parent: &Path) {
    let _ = git_command(
        repository_root,
        &[
            "worktree",
            "remove",
            "--force",
            worktree.to_str().unwrap_or_default(),
        ],
    );
    let _ = fs::remove_dir_all(parent);
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
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
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
