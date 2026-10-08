use crate::{
    ProcessAdmissionCheck, ProcessStartGate, VerificationCommand, VerificationReusePolicy,
    execute_bounded_with_process_observer, execute_bounded_with_process_observer_and_start_gate,
};
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
#[cfg(all(feature = "test-support", target_os = "linux"))]
use std::sync::Arc;
#[cfg(unix)]
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
#[cfg(target_os = "linux")]
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const COMPOSITION_SCHEMA_VERSION: u32 = 3;
const COMPOSITION_BINDING_SCHEMA_VERSION: u32 = 1;
const PROCESS_OBSERVATION_SCHEMA_VERSION: u32 = 1;
const MAX_OUTPUT_BYTES: usize = 64 * 1024;
static NEXT_COMPOSITION_PARENT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

static OWNER_INTERRUPTION_SIGNAL: AtomicI32 = AtomicI32::new(0);

#[cfg(unix)]
struct OwnerInterruptionHandlerState {
    active_guards: usize,
    previous_action: Option<libc::sigaction>,
}

#[cfg(unix)]
static OWNER_INTERRUPTION_HANDLER_STATE: Mutex<OwnerInterruptionHandlerState> =
    Mutex::new(OwnerInterruptionHandlerState {
        active_guards: 0,
        previous_action: None,
    });

#[cfg(unix)]
extern "C" fn record_owner_interruption_signal(signal: libc::c_int) {
    OWNER_INTERRUPTION_SIGNAL.store(signal, Ordering::SeqCst);
}

/// Captures SIGINT while an admitted composition is active so it can stop the
/// verifier, persist the signal, and clean up before returning to the caller.
pub struct OwnerInterruptionGuard;

impl OwnerInterruptionGuard {
    pub fn install() -> Result<Self, CompositionError> {
        #[cfg(unix)]
        {
            let mut state = OWNER_INTERRUPTION_HANDLER_STATE
                .lock()
                .map_err(|error| CompositionError::InterruptionHandler(error.to_string()))?;
            if state.active_guards == 0 {
                OWNER_INTERRUPTION_SIGNAL.store(0, Ordering::SeqCst);
                let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
                action.sa_sigaction = record_owner_interruption_signal as *const () as usize;
                action.sa_flags = 0;
                unsafe {
                    libc::sigemptyset(&mut action.sa_mask);
                }
                let mut previous_action: libc::sigaction = unsafe { std::mem::zeroed() };
                let result =
                    unsafe { libc::sigaction(libc::SIGINT, &action, &mut previous_action) };
                if result != 0 {
                    return Err(CompositionError::InterruptionHandler(
                        std::io::Error::last_os_error().to_string(),
                    ));
                }
                state.previous_action = Some(previous_action);
            }
            state.active_guards += 1;
        }
        Ok(Self)
    }
}

impl Drop for OwnerInterruptionGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        if let Ok(mut state) = OWNER_INTERRUPTION_HANDLER_STATE.lock()
            && state.active_guards > 0
        {
            state.active_guards -= 1;
            if state.active_guards == 0 {
                if let Some(previous_action) = state.previous_action.take() {
                    unsafe {
                        libc::sigaction(libc::SIGINT, &previous_action, std::ptr::null_mut());
                    }
                }
                OWNER_INTERRUPTION_SIGNAL.store(0, Ordering::SeqCst);
            }
        }
    }
}

pub(crate) fn owner_interruption_signal() -> Option<i32> {
    let signal = OWNER_INTERRUPTION_SIGNAL.load(Ordering::SeqCst);
    (signal != 0).then_some(signal)
}

pub fn observe_composition_process_identity(
    process_id: u32,
) -> Result<CompositionProcessIdentity, String> {
    #[cfg(target_os = "linux")]
    {
        let stat = fs::read_to_string(format!("/proc/{process_id}/stat"))
            .map_err(|error| format!("read Linux process identity for {process_id}: {error}"))?;
        let member = parse_linux_process_stat(process_id, &stat)?;
        Ok(CompositionProcessIdentity {
            process_id,
            start_time_ticks: Some(member.start_time_ticks),
            process_group_id: Some(member.process_group_id),
            session_id: Some(member.session_id),
        })
    }
    #[cfg(all(unix, not(target_os = "linux")))]
    {
        let pid = process_id as libc::pid_t;
        let process_group_id = unsafe { libc::getpgid(pid) };
        let session_id = unsafe { libc::getsid(pid) };
        if process_group_id < 0 || session_id < 0 {
            return Err(format!(
                "observe Unix process group/session for {process_id}: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(CompositionProcessIdentity {
            process_id,
            start_time_ticks: None,
            process_group_id: Some(process_group_id as u32),
            session_id: Some(session_id as u32),
        })
    }
    #[cfg(windows)]
    {
        Ok(CompositionProcessIdentity {
            process_id,
            start_time_ticks: None,
            process_group_id: None,
            session_id: None,
        })
    }
}

pub fn current_composition_process_identity() -> Result<CompositionProcessIdentity, String> {
    observe_composition_process_identity(std::process::id())
}

pub fn initialize_composition_supervisor_backend() -> Result<CompositionSupervisorBackend, String> {
    #[cfg(target_os = "linux")]
    {
        // PR_SET_CHILD_SUBREAPER is scoped to this dedicated helper process.
        let set_result = unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) };
        if set_result != 0 {
            return Err(format!(
                "enable process-local Linux child subreaper: {}",
                std::io::Error::last_os_error()
            ));
        }
        let mut enabled: libc::c_int = 0;
        let get_result = unsafe {
            libc::prctl(
                libc::PR_GET_CHILD_SUBREAPER,
                &mut enabled as *mut libc::c_int,
                0,
                0,
                0,
            )
        };
        if get_result != 0 || enabled != 1 {
            return Err("Linux child subreaper state could not be verified".into());
        }
        Ok(CompositionSupervisorBackend::LinuxSubreaper)
    }
    #[cfg(all(unix, not(target_os = "linux")))]
    {
        Ok(CompositionSupervisorBackend::UnixProcessGroup)
    }
    #[cfg(windows)]
    {
        Ok(CompositionSupervisorBackend::WindowsProcessGroup)
    }
}

pub fn composition_linux_boot_id() -> Result<Option<String>, String> {
    #[cfg(target_os = "linux")]
    {
        let value = fs::read_to_string("/proc/sys/kernel/random/boot_id")
            .map_err(|error| format!("read Linux boot identity: {error}"))?;
        let value = value.trim();
        if value.is_empty() || value.len() > 64 {
            return Err("Linux boot identity is empty or malformed".into());
        }
        Ok(Some(value.to_owned()))
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(None)
    }
}

pub fn reap_composition_supervisor_descendants() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        reap_linux_composition_descendants()
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn reap_linux_composition_descendants() -> Result<(), String> {
    let own_pid = std::process::id();
    loop {
        let mut status = 0;
        let result = unsafe { libc::waitpid(-1, &mut status, libc::WNOHANG) };
        if result > 0 {
            continue;
        }
        if result < 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ECHILD) {
                return Ok(());
            }
            return Err(format!("reap Linux composition child: {error}"));
        }

        let children = linux_direct_children(own_pid)?;
        if children.is_empty() {
            return Err("waitpid reported a child but /proc lists no owned child".into());
        }
        for child in children {
            // An unreaped child cannot have its PID recycled, so this identity
            // observation binds the signal to the process in our child list.
            let current = observe_composition_process_identity(child.process_id)?;
            if current != child {
                return Err(format!(
                    "owned Linux child {} changed identity before termination",
                    child.process_id
                ));
            }
            let killed = unsafe { libc::kill(child.process_id as libc::pid_t, libc::SIGKILL) };
            if killed != 0 {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() != Some(libc::ESRCH) {
                    return Err(format!(
                        "terminate owned Linux child {}: {error}",
                        child.process_id
                    ));
                }
            }
        }
        loop {
            let mut status = 0;
            let result = unsafe { libc::waitpid(-1, &mut status, libc::WNOHANG) };
            if result > 0 {
                continue;
            }
            if result < 0 {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() == Some(libc::ECHILD) {
                    return Ok(());
                }
                return Err(format!("reap terminated Linux composition child: {error}"));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

#[cfg(target_os = "linux")]
fn linux_direct_children(
    parent_process_id: u32,
) -> Result<Vec<CompositionProcessIdentity>, String> {
    let entries = fs::read_dir("/proc")
        .map_err(|error| format!("enumerate Linux processes for child ownership: {error}"))?;
    let mut children = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("enumerate Linux processes: {error}"))?;
        let Some(process_id) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        let stat_path = entry.path().join("stat");
        let stat = match fs::read_to_string(&stat_path) {
            Ok(stat) => stat,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "read Linux process stat {} while proving child ownership: {error}",
                    stat_path.display()
                ));
            }
        };
        if parse_linux_process_parent_id(process_id, &stat)? == parent_process_id {
            let member = parse_linux_process_stat(process_id, &stat)?;
            children.push(CompositionProcessIdentity {
                process_id,
                start_time_ticks: Some(member.start_time_ticks),
                process_group_id: Some(member.process_group_id),
                session_id: Some(member.session_id),
            });
        }
    }
    Ok(children)
}

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub termination_signal: Option<i32>,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionExecutionOutcome {
    Passed,
    Failed,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionCleanupDisposition {
    Cleaned,
    Deferred,
    Retained,
    Failed,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionSupervisorBackend {
    LinuxSubreaper,
    UnixProcessGroup,
    WindowsProcessGroup,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompositionProcessIdentity {
    pub process_id: u32,
    pub start_time_ticks: Option<u64>,
    pub process_group_id: Option<u32>,
    pub session_id: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompositionSupervisorReceipt {
    pub schema_version: u32,
    pub backend: CompositionSupervisorBackend,
    pub attempt_id: String,
    pub run_nonce: String,
    pub generation: u64,
    pub owner: CompositionProcessIdentity,
    pub supervisor: CompositionProcessIdentity,
    pub linux_boot_id: Option<String>,
    pub environment_digest: Digest,
    pub runtime_version: String,
    pub runtime_digest: Digest,
    pub repository_id: Digest,
    pub target_sha: String,
    pub snapshot_digest: Digest,
    pub command_plan_digest: Digest,
    pub input_environment_digest: Digest,
    pub execution_records_digest: Digest,
    pub descendants_reaped_to_echild: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompositionSupervisorLaunch {
    pub repository_root: PathBuf,
    pub work_item_id: String,
    pub generation: u64,
    pub input: CompositionInput,
    pub attempt_id: String,
    pub run_nonce: String,
    pub owner: CompositionProcessIdentity,
    pub runtime_version: String,
    pub runtime_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompositionSupervisorReady {
    pub schema_version: u32,
    pub supervisor: CompositionProcessIdentity,
    pub runtime_version: String,
    pub runtime_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CompositionSupervisorControl {
    Release {
        receipt: CompositionSupervisorReceipt,
    },
    Abort {
        reason: String,
        receipt: CompositionSupervisorReceipt,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompositionSupervisorReply {
    pub attempt: Option<CompositionAttempt>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProcessGroupLeaderIdentity {
    pub leader_pid: u32,
    pub leader_start_time_ticks: u64,
    pub process_group_id: u32,
    pub session_id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompositionAttempt {
    #[serde(default = "legacy_composition_schema_version")]
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
    pub execution_outcome: CompositionExecutionOutcome,
    #[serde(default)]
    pub execution_evidence_complete: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supervisor_receipt: Option<CompositionSupervisorReceipt>,
    #[serde(default)]
    pub cleanup_disposition: CompositionCleanupDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_termination_signal: Option<i32>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_process_group_identity: Option<ProcessGroupLeaderIdentity>,
    #[serde(default, skip_serializing_if = "bool_is_false")]
    pub owned_tree_termination_unknown: bool,
}

fn bool_is_false(value: &bool) -> bool {
    !*value
}

impl CompositionAttempt {
    /// Whether this attempt is internally coherent enough to be projected or
    /// reused as a successful terminal composition.
    pub fn is_coherent_successful_terminal(&self) -> bool {
        is_reusable_terminal_attempt(self)
    }
}

fn legacy_composition_schema_version() -> u32 {
    1
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
    #[error("composition interruption handler could not be installed: {0}")]
    InterruptionHandler(String),
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
    run_composition_inner(input, None, None, None, None)
}

pub fn run_composition_with_process_gates(
    input: CompositionInput,
    process_admission_check: ProcessAdmissionCheck,
    process_start_gate: ProcessStartGate,
) -> Result<CompositionAttempt, CompositionError> {
    run_composition_inner(
        input,
        Some(process_admission_check),
        Some(process_start_gate),
        None,
        None,
    )
}

pub fn run_composition_with_supervisor_receipt(
    input: CompositionInput,
    process_admission_check: ProcessAdmissionCheck,
    process_start_gate: ProcessStartGate,
    receipt: CompositionSupervisorReceipt,
) -> Result<CompositionAttempt, CompositionError> {
    let attempt_id = receipt.attempt_id.clone();
    run_composition_inner(
        input,
        Some(process_admission_check),
        Some(process_start_gate),
        Some(receipt),
        Some(attempt_id),
    )
}

/// Test-only final worktree observation for selected pure cache fixtures.
/// The production observer remains authoritative for recovery and retry.
#[cfg(all(feature = "test-support", target_os = "linux"))]
#[doc(hidden)]
pub enum TestCompletedWorktreeObservation {
    KnownEmpty,
}

#[cfg(all(feature = "test-support", target_os = "linux"))]
#[doc(hidden)]
pub fn run_composition_with_test_completed_worktree_observation(
    input: CompositionInput,
    receipt: CompositionSupervisorReceipt,
    observation: TestCompletedWorktreeObservation,
) -> Result<CompositionAttempt, CompositionError> {
    let attempt_id = receipt.attempt_id.clone();
    let final_observation = match observation {
        TestCompletedWorktreeObservation::KnownEmpty => FinalWorktreeObservation::KnownEmpty,
    };
    // This fixed test fixture keeps every verifier on the normal admission
    // and ProcessStartGate call path. The observation interface cannot accept
    // caller-provided gates, admission results, observers, or reapers.
    let process_admission_check: ProcessAdmissionCheck = Arc::new(|_, accept| accept());
    let process_start_gate: ProcessStartGate = Arc::new(|_, spawn| spawn());
    run_composition_inner_with_observer(
        input,
        Some(process_admission_check),
        Some(process_start_gate),
        Some(receipt),
        Some(attempt_id),
        CompositionObservationHooks {
            observe_worktree_process: &verifier_process_using_worktree,
            reap_owned_descendants: &reap_composition_supervisor_descendants,
            final_observation,
        },
    )
}

pub fn record_composition_supervisor_failure(
    input: CompositionInput,
    attempt_id: String,
    receipt: Option<CompositionSupervisorReceipt>,
    failure: String,
) -> Result<CompositionAttempt, CompositionError> {
    let record_path = attempt_record_path(&input.state_dir, &attempt_id);
    if record_path.exists() {
        let bytes = fs::read(&record_path).map_err(|source| CompositionError::Io {
            path: record_path.clone(),
            source,
        })?;
        let attempt: CompositionAttempt = serde_json::from_slice(&bytes)
            .map_err(|error| CompositionError::Serialization(error.to_string()))?;
        return Ok(attempt);
    }
    let recorded_at_unix_nanos = now_unix_nanos();
    let mut attempt = CompositionAttempt {
        schema_version: COMPOSITION_SCHEMA_VERSION,
        attempt_id,
        binding: input.binding.clone(),
        identity: CompositionIdentity::default(),
        preconditions: input.preconditions.clone(),
        isolated_worktree: String::new(),
        text_conflicts: Vec::new(),
        execution_records: Vec::new(),
        processes_spawned: 0,
        reuse_decision: ReuseDecision {
            kind: ReuseDecisionKind::Unknown,
            reason: "supervisor handshake did not authorize execution".into(),
            predecessor_attempt_id: None,
        },
        passed: false,
        failure: Some(failure),
        execution_outcome: CompositionExecutionOutcome::Unknown,
        execution_evidence_complete: false,
        supervisor_receipt: receipt,
        cleanup_disposition: CompositionCleanupDisposition::Cleaned,
        owner_termination_signal: None,
        cleanup: Some(CompositionCleanup {
            attempted: false,
            removed: true,
            error: None,
        }),
        recorded_at_unix_nanos,
        owner_pid: Some(std::process::id()),
        process_observation_schema_version: PROCESS_OBSERVATION_SCHEMA_VERSION,
        active_execution_node: None,
        active_process_group_id: None,
        active_process_group_identity: None,
        owned_tree_termination_unknown: false,
    };
    refresh_execution_records_digest(&mut attempt);
    persist_attempt(&input.state_dir, &attempt)?;
    Ok(attempt)
}

fn run_composition_inner(
    input: CompositionInput,
    process_admission_check: Option<ProcessAdmissionCheck>,
    process_start_gate: Option<ProcessStartGate>,
    supervisor_receipt: Option<CompositionSupervisorReceipt>,
    supervised_attempt_id: Option<String>,
) -> Result<CompositionAttempt, CompositionError> {
    run_composition_inner_with_observer(
        input,
        process_admission_check,
        process_start_gate,
        supervisor_receipt,
        supervised_attempt_id,
        CompositionObservationHooks {
            observe_worktree_process: &verifier_process_using_worktree,
            reap_owned_descendants: &reap_composition_supervisor_descendants,
            final_observation: FinalWorktreeObservation::Real,
        },
    )
}

#[derive(Clone, Copy)]
enum FinalWorktreeObservation {
    Real,
    #[cfg(all(feature = "test-support", target_os = "linux"))]
    KnownEmpty,
}

struct CompositionObservationHooks<'a> {
    observe_worktree_process: &'a dyn Fn(&Path) -> Result<Option<u32>, String>,
    reap_owned_descendants: &'a dyn Fn() -> Result<(), String>,
    final_observation: FinalWorktreeObservation,
}

fn run_composition_inner_with_observer(
    input: CompositionInput,
    process_admission_check: Option<ProcessAdmissionCheck>,
    process_start_gate: Option<ProcessStartGate>,
    supervisor_receipt: Option<CompositionSupervisorReceipt>,
    supervised_attempt_id: Option<String>,
    hooks: CompositionObservationHooks<'_>,
) -> Result<CompositionAttempt, CompositionError> {
    let CompositionObservationHooks {
        observe_worktree_process,
        reap_owned_descendants,
        final_observation,
    } = hooks;
    let mut input = input;
    input.binding.verifier.validate_candidate()?;
    validate_binding(&input.binding)?;
    // Caller identity fields are descriptive only. Start from explicit
    // sentinels so a failed observation cannot persist self-reported claims.
    input.identity = CompositionIdentity::default();
    input.identity.command_digest = composition_commands_digest(&input.commands);

    validate_supervisor_registrations(&input.state_dir, supervisor_receipt.as_ref())?;
    let loaded_predecessor = load_latest_attempt(&input.state_dir, &input.binding)?;
    let deferred_predecessor = loaded_predecessor
        .as_ref()
        .is_some_and(is_deferred_cleanup_attempt);
    let deferred_retry_blocked_reason = if deferred_predecessor {
        loaded_predecessor.as_ref().and_then(|previous| {
            validate_deferred_cleanup_retry(
                previous,
                &input,
                supervisor_receipt.as_ref(),
                observe_worktree_process,
            )
            .err()
        })
    } else {
        None
    };
    let predecessor = if deferred_predecessor {
        // A deferred tree is never passed to abandoned-attempt cleanup. It is
        // either validated as a safe retry predecessor or preserved while
        // the new attempt is rejected below.
        loaded_predecessor
    } else {
        reconcile_abandoned_attempt_with_observer(
            &input.repository_root,
            &input.state_dir,
            loaded_predecessor,
            observe_worktree_process,
        )?
    };
    let fresh_deferred_retry = deferred_predecessor && deferred_retry_blocked_reason.is_none();
    let recorded_at_unix_nanos = now_unix_nanos();
    let attempt_id =
        supervised_attempt_id.unwrap_or_else(|| new_attempt_id(&input, recorded_at_unix_nanos));
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
        execution_outcome: CompositionExecutionOutcome::Unknown,
        execution_evidence_complete: false,
        supervisor_receipt,
        cleanup_disposition: CompositionCleanupDisposition::Unknown,
        owner_termination_signal: None,
        cleanup: None,
        recorded_at_unix_nanos,
        owner_pid: Some(std::process::id()),
        process_observation_schema_version: PROCESS_OBSERVATION_SCHEMA_VERSION,
        active_execution_node: None,
        active_process_group_id: None,
        active_process_group_identity: None,
        owned_tree_termination_unknown: false,
    };

    // This snapshot is the recovery boundary: an interrupted parent leaves a
    // durable in_progress attempt rather than an invisible execution.
    persist_attempt(&input.state_dir, &attempt)?;

    if input.commands.is_empty() {
        fail_without_worktree(&input.state_dir, &mut attempt, "required_checks_empty")?;
        return Ok(attempt);
    }
    if let Some(reason) = deferred_retry_blocked_reason {
        fail_without_worktree(
            &input.state_dir,
            &mut attempt,
            &format!("unsafe_deferred_cleanup_retry:{reason}"),
        )?;
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
        set_cleanup_disposition(&mut attempt);
        set_execution_outcome(&mut attempt);
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
            set_cleanup_disposition(&mut attempt);
            set_execution_outcome(&mut attempt);
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
        set_cleanup_disposition(&mut attempt);
        set_execution_outcome(&mut attempt);
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
    if fresh_deferred_retry {
        let previous = predecessor
            .as_ref()
            .expect("deferred retry has a predecessor");
        let retry_identity_matches = identity_observed
            && previous.identity == input.identity
            && previous
                .execution_records
                .first()
                .zip(input.commands.first())
                .and_then(|(record, command)| {
                    let environment = controlled_command_environment(command)?;
                    let executable = resolve_executable(&worktree, &command.program, &environment)?;
                    let runtime_toolchain_digest = runtime_toolchain_digest.as_ref()?;
                    observed_node_identity(
                        &worktree,
                        &input,
                        command,
                        &[],
                        &executable,
                        &environment,
                        Some(runtime_toolchain_digest),
                    )
                    .map(|identity| identity == record.identity_digest)
                })
                .unwrap_or(false);
        if !retry_identity_matches {
            attempt.failure =
                Some("unsafe_deferred_cleanup_retry:source_or_toolchain_changed".into());
            attempt.cleanup = Some(worktree_guard.cleanup());
            set_cleanup_disposition(&mut attempt);
            set_execution_outcome(&mut attempt);
            persist_attempt(&input.state_dir, &attempt)?;
            return Ok(attempt);
        }
        attempt.reuse_decision = ReuseDecision {
            kind: ReuseDecisionKind::Execute,
            reason: "fresh execution after deferred cleanup; predecessor results are not reused"
                .into(),
            predecessor_attempt_id: Some(previous.attempt_id.clone()),
        };
        persist_attempt(&input.state_dir, &attempt)?;
    }
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
        if let Some(signal) = owner_interruption_signal() {
            persist_owner_interruption(&input.state_dir, &mut attempt, signal)?;
            break;
        }
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
        let reusable = if !fresh_deferred_retry
            && identity_observed
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
            let mut accepted = false;
            let mut persistence_error = None;
            let admission_result = {
                let mut accept_reuse = || {
                    accepted = true;
                    attempt.execution_records.push(record.clone());
                    refresh_execution_records_digest(&mut attempt);
                    match persist_attempt(&input.state_dir, &attempt) {
                        Ok(()) => Ok(()),
                        Err(error) => {
                            persistence_error = Some(error);
                            Err("failed to persist accepted composition reuse".into())
                        }
                    }
                };
                if let Some(check) = &process_admission_check {
                    check(&command.node_id, &mut accept_reuse)
                } else {
                    accept_reuse()
                }
            };
            if let Some(error) = persistence_error {
                return Err(error);
            }
            if let Err(error) = admission_result {
                attempt.failure = Some(format!(
                    "composition_reuse_not_admitted:{}:{error}",
                    command.node_id
                ));
                persist_attempt(&input.state_dir, &attempt)?;
                break;
            }
            if !accepted {
                attempt.failure = Some(format!(
                    "composition_reuse_not_admitted:{}:admission gate did not accept the cached result",
                    command.node_id
                ));
                persist_attempt(&input.state_dir, &attempt)?;
                break;
            }
            continue;
        }
        all_nodes_reused = false;
        attempt.active_execution_node = Some(command.node_id.clone());
        attempt.active_process_group_id = None;
        attempt.active_process_group_identity = None;
        persist_attempt(&input.state_dir, &attempt)?;
        let record = execute_node(
            command,
            NodeExecutionContext {
                worktree: &worktree,
                input: &input,
                executable: &executable,
                environment: &environment,
                identity_digest: current_identity,
                attempt: &attempt,
                process_start_gate: process_start_gate.as_ref(),
            },
        );
        attempt.active_execution_node = None;
        attempt.active_process_group_id = None;
        attempt.active_process_group_identity = None;
        attempt.processes_spawned += usize::from(record.spawned);
        let passed = record.passed;
        attempt.execution_records.push(record);
        refresh_execution_records_digest(&mut attempt);
        persist_attempt(&input.state_dir, &attempt)?;
        if let Some(signal) = owner_interruption_signal() {
            persist_owner_interruption(&input.state_dir, &mut attempt, signal)?;
            break;
        }
        if !passed {
            let failed_record = attempt
                .execution_records
                .last()
                .expect("failed command was just persisted");
            attempt.failure = Some(if let Some(signal) = failed_record.termination_signal {
                format!("command_interrupted:signal={signal}")
            } else {
                format!("command_failed:exit={:?}", failed_record.exit_code)
            });
            persist_attempt(&input.state_dir, &attempt)?;
            break;
        }
    }

    if let Some(signal) = owner_interruption_signal() {
        persist_owner_interruption(&input.state_dir, &mut attempt, signal)?;
    }
    attempt.passed = (attempt.failure.is_none()
        || attempt.failure.as_deref() == Some("in_progress"))
        && attempt.execution_records.len() == input.commands.len()
        && attempt.execution_records.iter().all(|record| record.passed);
    if attempt.passed {
        attempt.failure = None;
    }
    set_execution_outcome(&mut attempt);
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
    if attempt.supervisor_receipt.is_some() {
        if let Err(error) = reap_owned_descendants() {
            attempt.passed = false;
            attempt.failure = Some(format!("composition_supervisor_reap_unknown:{error}"));
            attempt.execution_outcome = CompositionExecutionOutcome::Unknown;
            attempt.execution_evidence_complete = false;
            attempt.cleanup_disposition = CompositionCleanupDisposition::Retained;
            attempt.owned_tree_termination_unknown = true;
            worktree_guard.preserve();
            persist_attempt(&input.state_dir, &attempt)?;
            return Ok(attempt);
        }
        if let Some(receipt) = attempt.supervisor_receipt.as_mut() {
            receipt.descendants_reaped_to_echild =
                receipt.backend == CompositionSupervisorBackend::LinuxSubreaper;
        }
        persist_attempt(&input.state_dir, &attempt)?;
    }
    let completed_worktree_observation = match final_observation {
        FinalWorktreeObservation::Real => observe_worktree_process(&worktree),
        #[cfg(all(feature = "test-support", target_os = "linux"))]
        FinalWorktreeObservation::KnownEmpty => {
            if completed_owned_execution_is_coherent(&attempt, &input) {
                Ok(None)
            } else {
                Err("test KnownEmpty requires coherent, reaped owned execution".into())
            }
        }
    };
    match completed_worktree_observation {
        Ok(Some(process_id)) => {
            attempt.passed = false;
            attempt.failure = Some(format!("verifier_descendant_active:{process_id}"));
            worktree_guard.preserve();
            attempt.cleanup_disposition = CompositionCleanupDisposition::Retained;
            persist_attempt(&input.state_dir, &attempt)?;
            return Ok(attempt);
        }
        Err(error) => {
            let owned_execution_complete = completed_owned_execution_is_coherent(&attempt, &input);
            attempt.passed = false;
            worktree_guard.preserve();
            if owned_execution_complete {
                attempt.cleanup = Some(CompositionCleanup {
                    attempted: false,
                    removed: false,
                    error: Some(format!("verifier_process_state_unknown:{error}")),
                });
                attempt.cleanup_disposition = CompositionCleanupDisposition::Deferred;
                if attempt.execution_outcome == CompositionExecutionOutcome::Passed {
                    attempt.failure = Some("composition_cleanup_deferred".into());
                }
            } else {
                attempt.owned_tree_termination_unknown = true;
                attempt.failure = Some(format!("verifier_process_state_unknown:{error}"));
                attempt.cleanup_disposition = CompositionCleanupDisposition::Retained;
                attempt.execution_outcome = CompositionExecutionOutcome::Unknown;
                attempt.execution_evidence_complete = false;
            }
            persist_attempt(&input.state_dir, &attempt)?;
            return Ok(attempt);
        }
        Ok(None) => {}
    }
    attempt.cleanup = Some(worktree_guard.cleanup());
    set_cleanup_disposition(&mut attempt);
    if !attempt
        .cleanup
        .as_ref()
        .is_some_and(|cleanup| cleanup.removed)
    {
        attempt.passed = false;
        if attempt.execution_outcome == CompositionExecutionOutcome::Passed
            && attempt
                .supervisor_receipt
                .as_ref()
                .is_some_and(|receipt| receipt.descendants_reaped_to_echild)
        {
            attempt.cleanup_disposition = CompositionCleanupDisposition::Deferred;
            attempt.failure = Some("composition_cleanup_deferred".into());
        } else {
            attempt.cleanup_disposition = CompositionCleanupDisposition::Failed;
            attempt.failure = Some("composition_cleanup_failed".into());
        }
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
    if binding.schema_version != COMPOSITION_BINDING_SCHEMA_VERSION
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
    attempt.execution_outcome = CompositionExecutionOutcome::Failed;
    attempt.execution_evidence_complete = true;
    set_cleanup_disposition(attempt);
    persist_attempt(state_dir, attempt)
}

fn set_execution_outcome(attempt: &mut CompositionAttempt) {
    if attempt.owner_termination_signal.is_some()
        || attempt.active_execution_node.is_some()
        || attempt.active_process_group_id.is_some()
        || attempt.failure.as_deref().is_some_and(|failure| {
            failure == "in_progress" || failure.starts_with("verifier_process_state_unknown:")
        })
    {
        attempt.execution_outcome = CompositionExecutionOutcome::Unknown;
        attempt.execution_evidence_complete = false;
    } else if attempt.passed {
        attempt.execution_outcome = CompositionExecutionOutcome::Passed;
        attempt.execution_evidence_complete = true;
    } else if attempt.failure.is_some() {
        attempt.execution_outcome = CompositionExecutionOutcome::Failed;
        attempt.execution_evidence_complete = true;
    } else {
        attempt.execution_outcome = CompositionExecutionOutcome::Unknown;
        attempt.execution_evidence_complete = false;
    }
}

fn set_cleanup_disposition(attempt: &mut CompositionAttempt) {
    attempt.cleanup_disposition = match attempt.cleanup.as_ref() {
        Some(cleanup) if cleanup.removed && cleanup.error.is_none() => {
            CompositionCleanupDisposition::Cleaned
        }
        Some(cleanup) if cleanup.error.is_some() => CompositionCleanupDisposition::Failed,
        Some(_) => CompositionCleanupDisposition::Deferred,
        None => CompositionCleanupDisposition::Unknown,
    };
}

fn refresh_execution_records_digest(attempt: &mut CompositionAttempt) {
    if let Some(receipt) = attempt.supervisor_receipt.as_mut() {
        receipt.execution_records_digest = execution_records_digest(&attempt.execution_records);
    }
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
        if !attempt_record_filename_matches(&path, &candidate.attempt_id) {
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

fn validate_supervisor_registrations(
    state_dir: &Path,
    active_receipt: Option<&CompositionSupervisorReceipt>,
) -> Result<(), CompositionError> {
    let directory = state_dir.join("supervisors");
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(CompositionError::Io {
                path: directory,
                source,
            });
        }
    };
    for entry in entries {
        let entry = entry.map_err(|source| CompositionError::Io {
            path: directory.clone(),
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
                "supervisor registration is not a regular file: {}",
                path.display()
            )));
        }
        let bytes = fs::read(&path).map_err(|source| CompositionError::Io {
            path: path.clone(),
            source,
        })?;
        let registered: CompositionSupervisorReceipt = serde_json::from_slice(&bytes)
            .map_err(|error| CompositionError::Serialization(error.to_string()))?;
        let attempt_path = attempt_record_path(state_dir, &registered.attempt_id);
        let attempt_bytes = match fs::read(&attempt_path) {
            Ok(bytes) => bytes,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                let active_identity = active_receipt
                    .filter(|active| *active == &registered)
                    .and_then(|_| current_composition_process_identity().ok());
                if active_identity.as_ref() == Some(&registered.supervisor) {
                    continue;
                }
                return Err(CompositionError::UnknownAttemptOwner {
                    attempt_id: registered.attempt_id.clone(),
                });
            }
            Err(source) => {
                return Err(CompositionError::Io {
                    path: attempt_path.clone(),
                    source,
                });
            }
        };
        let attempt: CompositionAttempt = serde_json::from_slice(&attempt_bytes)
            .map_err(|error| CompositionError::Serialization(error.to_string()))?;
        let Some(actual) = attempt.supervisor_receipt.as_ref() else {
            return Err(CompositionError::UnknownAttemptOwner {
                attempt_id: registered.attempt_id,
            });
        };
        let mut registered_identity = registered;
        registered_identity.execution_records_digest = actual.execution_records_digest.clone();
        registered_identity.descendants_reaped_to_echild = actual.descendants_reaped_to_echild;
        if attempt.attempt_id != actual.attempt_id || &registered_identity != actual {
            return Err(CompositionError::UnknownAttemptOwner {
                attempt_id: attempt.attempt_id,
            });
        }
    }
    Ok(())
}

fn same_composition_lineage(previous: &CompositionBinding, current: &CompositionBinding) -> bool {
    previous.repository_id == current.repository_id
        && previous.target_branch == current.target_branch
        && previous.participant_work_items == current.participant_work_items
        && previous.verifier == current.verifier
}

fn is_deferred_cleanup_attempt(attempt: &CompositionAttempt) -> bool {
    attempt.cleanup_disposition == CompositionCleanupDisposition::Deferred
        || attempt.failure.as_deref() == Some("composition_cleanup_deferred")
}

fn completed_owned_execution_is_coherent(
    attempt: &CompositionAttempt,
    input: &CompositionInput,
) -> bool {
    let Some(receipt) = attempt.supervisor_receipt.as_ref() else {
        return false;
    };
    attempt.schema_version == COMPOSITION_SCHEMA_VERSION
        && attempt.process_observation_schema_version >= PROCESS_OBSERVATION_SCHEMA_VERSION
        && attempt.cleanup.is_none()
        && attempt.cleanup_disposition == CompositionCleanupDisposition::Unknown
        && attempt.text_conflicts.is_empty()
        && attempt.preconditions == input.preconditions
        && attempt.preconditions.iter().all(|item| item.satisfied)
        && receipt.schema_version == 1
        && receipt.backend == CompositionSupervisorBackend::LinuxSubreaper
        && receipt.attempt_id == attempt.attempt_id
        && !receipt.run_nonce.is_empty()
        && receipt.generation > 0
        && receipt.owner.process_id > 0
        && receipt.owner.start_time_ticks.is_some()
        && receipt.supervisor.process_id > 0
        && receipt.supervisor.start_time_ticks.is_some()
        && receipt.supervisor.process_group_id.is_some()
        && receipt.supervisor.session_id.is_some()
        && current_composition_process_identity().ok().as_ref() == Some(&receipt.supervisor)
        && composition_linux_boot_id().ok().as_ref() == Some(&receipt.linux_boot_id)
        && receipt
            .linux_boot_id
            .as_ref()
            .is_some_and(|id| !id.is_empty())
        && attempt.owner_pid == Some(receipt.supervisor.process_id)
        && receipt.runtime_version == input.binding.verifier.runtime_version
        && receipt.runtime_digest == input.binding.verifier.runtime_digest
        && receipt.repository_id == input.binding.repository_id
        && receipt.target_sha == input.binding.target_sha
        && composition_target_snapshot_digest(input).as_ref() == Some(&receipt.snapshot_digest)
        && receipt.command_plan_digest == composition_commands_digest(&input.commands)
        && receipt.command_plan_digest == attempt.identity.command_digest
        && receipt.input_environment_digest == composition_input_environment_digest(input)
        && receipt.execution_records_digest == execution_records_digest(&attempt.execution_records)
        && receipt.descendants_reaped_to_echild
        && attempt.binding == input.binding
        && attempt.execution_outcome != CompositionExecutionOutcome::Unknown
        && attempt.execution_evidence_complete
        && match attempt.execution_outcome {
            CompositionExecutionOutcome::Passed => {
                attempt.passed
                    && attempt.failure.is_none()
                    && attempt.execution_records.len() == input.commands.len()
                    && attempt.execution_records.iter().all(|record| record.passed)
            }
            CompositionExecutionOutcome::Failed => !attempt.passed && attempt.failure.is_some(),
            CompositionExecutionOutcome::Unknown => false,
        }
        && attempt.owner_termination_signal.is_none()
        && attempt.active_execution_node.is_none()
        && attempt.active_process_group_id.is_none()
        && attempt.active_process_group_identity.is_none()
        && !attempt.owned_tree_termination_unknown
        && !attempt.execution_records.is_empty()
        && attempt.execution_records.len() <= input.commands.len()
        && attempt.processes_spawned
            == attempt
                .execution_records
                .iter()
                .filter(|record| record.spawned)
                .count()
        && attempt
            .execution_records
            .iter()
            .zip(&input.commands)
            .all(|(record, command)| {
                record.node_id == command.node_id
                    && record.program == command.program
                    && record.args == command.args
                    && (record.spawned ^ record.reused)
            })
}

fn validate_deferred_cleanup_retry(
    previous: &CompositionAttempt,
    input: &CompositionInput,
    current_receipt: Option<&CompositionSupervisorReceipt>,
    observe_worktree_process: &dyn Fn(&Path) -> Result<Option<u32>, String>,
) -> Result<(), String> {
    if !cfg!(target_os = "linux") {
        return Err("reliable deferred-tree retry proof is Linux-only".into());
    }
    if !has_concrete_safe_deferred_retry_effects(input) {
        return Err("command effects are outside the safe retry allowlist".into());
    }
    if previous.schema_version != COMPOSITION_SCHEMA_VERSION
        || previous.binding != input.binding
        || previous.execution_outcome != CompositionExecutionOutcome::Passed
        || !previous.execution_evidence_complete
        || previous.cleanup_disposition != CompositionCleanupDisposition::Deferred
        || previous.passed
        || previous.failure.as_deref() != Some("composition_cleanup_deferred")
        || previous.owner_termination_signal.is_some()
        || previous.active_execution_node.is_some()
        || previous.active_process_group_id.is_some()
        || previous.active_process_group_identity.is_some()
        || previous.owned_tree_termination_unknown
        || previous.process_observation_schema_version < PROCESS_OBSERVATION_SCHEMA_VERSION
        || !previous.text_conflicts.is_empty()
        || !previous
            .preconditions
            .iter()
            .all(|precondition| precondition.satisfied)
    {
        return Err("deferred attempt state or exact binding is inconsistent".into());
    }
    let cleanup = previous
        .cleanup
        .as_ref()
        .ok_or_else(|| "deferred attempt has no cleanup evidence".to_string())?;
    if cleanup.removed
        || cleanup.error.as_deref().is_none_or(str::is_empty)
        || (!cleanup.attempted
            && !cleanup
                .error
                .as_deref()
                .is_some_and(|error| error.starts_with("verifier_process_state_unknown:")))
    {
        return Err("deferred attempt cleanup evidence is incomplete".into());
    }

    let current_receipt = current_receipt
        .ok_or_else(|| "current formal supervisor receipt is missing".to_string())?;
    let receipt = previous
        .supervisor_receipt
        .as_ref()
        .ok_or_else(|| "deferred attempt has no supervisor receipt".to_string())?;
    let current_boot_id = composition_linux_boot_id()
        .map_err(|error| format!("cannot verify current Linux boot identity: {error}"))?;
    let target_snapshot_digest = composition_target_snapshot_digest(input)
        .ok_or_else(|| "current target tree snapshot cannot be observed".to_string())?;
    if current_receipt.schema_version != 1
        || current_receipt.backend != CompositionSupervisorBackend::LinuxSubreaper
        || current_receipt.attempt_id.is_empty()
        || current_receipt.run_nonce.is_empty()
        || current_receipt.generation == 0
        || current_receipt.owner.process_id == 0
        || current_receipt.owner.start_time_ticks.is_none()
        || current_receipt.supervisor.process_id == 0
        || current_receipt.supervisor.start_time_ticks.is_none()
        || current_receipt.supervisor.process_group_id.is_none()
        || current_receipt.supervisor.session_id.is_none()
        || current_composition_process_identity().ok().as_ref() != Some(&current_receipt.supervisor)
        || current_receipt.linux_boot_id != current_boot_id
        || current_receipt.runtime_version != input.binding.verifier.runtime_version
        || current_receipt.runtime_digest != input.binding.verifier.runtime_digest
        || current_receipt.repository_id != input.binding.repository_id
        || current_receipt.target_sha != input.binding.target_sha
        || current_receipt.snapshot_digest != target_snapshot_digest
        || current_receipt.command_plan_digest != composition_commands_digest(&input.commands)
        || current_receipt.input_environment_digest != composition_input_environment_digest(input)
        || current_receipt.execution_records_digest != execution_records_digest(&[])
        || current_receipt.descendants_reaped_to_echild
    {
        return Err("current supervisor receipt does not bind a fresh exact attempt".into());
    }
    if receipt.schema_version != 1
        || receipt.backend != CompositionSupervisorBackend::LinuxSubreaper
        || receipt.attempt_id != previous.attempt_id
        || receipt.run_nonce.is_empty()
        || receipt.attempt_id == current_receipt.attempt_id
        || receipt.run_nonce == current_receipt.run_nonce
        || receipt.generation != current_receipt.generation
        || receipt.owner.process_id == 0
        || receipt.owner.start_time_ticks.is_none()
        || receipt.supervisor.process_id == 0
        || receipt.supervisor.start_time_ticks.is_none()
        || receipt.supervisor.process_group_id.is_none()
        || receipt.supervisor.session_id.is_none()
        || previous.owner_pid != Some(receipt.supervisor.process_id)
        || receipt.linux_boot_id.is_none()
        || receipt.linux_boot_id != current_boot_id
        || receipt.runtime_version != input.binding.verifier.runtime_version
        || receipt.runtime_digest != input.binding.verifier.runtime_digest
        || receipt.repository_id != input.binding.repository_id
        || receipt.target_sha != input.binding.target_sha
        || receipt.snapshot_digest != target_snapshot_digest
        || receipt.command_plan_digest != composition_commands_digest(&input.commands)
        || receipt.command_plan_digest != previous.identity.command_digest
        || receipt.input_environment_digest != composition_input_environment_digest(input)
        || receipt.environment_digest != current_receipt.environment_digest
        || receipt.execution_records_digest != execution_records_digest(&previous.execution_records)
        || !receipt.descendants_reaped_to_echild
    {
        return Err(
            "supervisor receipt does not prove a current, fully reaped exact attempt".into(),
        );
    }
    if previous.preconditions != input.preconditions
        || previous.execution_records.len() != input.commands.len()
        || previous.processes_spawned != input.commands.len()
        || previous
            .execution_records
            .iter()
            .zip(&input.commands)
            .any(|(record, command)| {
                record.node_id != command.node_id
                    || record.program != command.program
                    || record.args != command.args
                    || !record.spawned
                    || record.reused
                    || !record.passed
                    || record.exit_code != Some(0)
                    || record.timed_out
                    || record.termination_signal.is_some()
                    || record.predecessor_attempt_id.is_some()
            })
    {
        return Err(
            "deferred attempt execution evidence does not match the current command plan".into(),
        );
    }

    let owner_pid = previous
        .owner_pid
        .ok_or_else(|| "deferred attempt owner PID is missing".to_string())?;
    let (worktree, _) = validated_composition_paths(previous, owner_pid)
        .ok_or_else(|| "deferred worktree is outside its owned temporary namespace".to_string())?;
    let metadata = fs::symlink_metadata(&worktree)
        .map_err(|error| format!("cannot inspect deferred worktree: {error}"))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("deferred worktree is not a real directory".into());
    }
    if !worktree_is_registered(&input.repository_root, &worktree)
        .map_err(|error| format!("cannot verify deferred worktree registration: {error}"))?
    {
        return Err("deferred worktree is not registered to the repository".into());
    }
    linux_prior_supervisor_termination_proven(receipt)?;
    match observe_worktree_process(&worktree) {
        Ok(None) => {}
        Ok(Some(process_id)) => {
            return Err(format!(
                "process {process_id} still uses the deferred worktree"
            ));
        }
        // Exact owned-tree ECHILD proof and the side-effect-safe no-op policy
        // permit a fresh attempt while this old worktree remains untouched.
        Err(_) => {}
    }
    Ok(())
}

fn has_concrete_safe_deferred_retry_effects(input: &CompositionInput) -> bool {
    let [command] = input.commands.as_slice() else {
        return false;
    };
    command.program == "true"
        && command.args.is_empty()
        && command.depends_on.is_empty()
        && command.environment.is_empty()
        && command.input_paths.is_empty()
}

fn composition_target_snapshot_digest(input: &CompositionInput) -> Option<Digest> {
    let tree = format!("{}^{{tree}}", input.binding.target_sha);
    let tree = git_text(&input.repository_root, &["rev-parse", "--verify", &tree])?;
    Some(Digest::sha256_bytes(tree.as_bytes()))
}

fn composition_input_environment_digest(input: &CompositionInput) -> Digest {
    let values = input
        .commands
        .iter()
        .map(|command| (&command.node_id, &command.environment))
        .collect::<Vec<_>>();
    Digest::sha256_bytes(&serde_json::to_vec(&values).expect("command environments serialize"))
}

#[cfg(target_os = "linux")]
fn linux_prior_supervisor_termination_proven(
    receipt: &CompositionSupervisorReceipt,
) -> Result<(), String> {
    let path = format!("/proc/{}/stat", receipt.supervisor.process_id);
    let stat = match fs::read_to_string(&path) {
        Ok(stat) => stat,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("cannot inspect prior supervisor identity: {error}")),
    };
    let current = parse_linux_process_stat(receipt.supervisor.process_id, &stat)
        .map_err(|error| format!("cannot parse prior supervisor identity: {error}"))?;
    if receipt.supervisor.start_time_ticks == Some(current.start_time_ticks)
        && receipt.supervisor.process_group_id == Some(current.process_group_id)
        && receipt.supervisor.session_id == Some(current.session_id)
    {
        return Err("prior composition supervisor is still active".into());
    }
    // A PID reused on the same boot is distinct from the recorded supervisor;
    // the exact process identity, not the numeric PID, controls this proof.
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn linux_prior_supervisor_termination_proven(
    _receipt: &CompositionSupervisorReceipt,
) -> Result<(), String> {
    Err("Linux supervisor termination proof is unavailable".into())
}

fn is_reusable_terminal_attempt(attempt: &CompositionAttempt) -> bool {
    attempt.schema_version == COMPOSITION_SCHEMA_VERSION
        && attempt.supervisor_receipt.as_ref().is_some_and(|receipt| {
            receipt.schema_version == 1
                && receipt.attempt_id == attempt.attempt_id
                && !receipt.run_nonce.is_empty()
                && receipt.owner.process_id > 0
                && receipt.supervisor.process_id > 0
                && receipt.runtime_version == attempt.binding.verifier.runtime_version
                && receipt.runtime_digest == attempt.binding.verifier.runtime_digest
                && receipt.repository_id == attempt.binding.repository_id
                && receipt.target_sha == attempt.binding.target_sha
                && receipt.command_plan_digest == attempt.identity.command_digest
                && receipt.execution_records_digest
                    == execution_records_digest(&attempt.execution_records)
                && receipt.supervisor.process_id == attempt.owner_pid.unwrap_or_default()
                && match receipt.backend {
                    CompositionSupervisorBackend::LinuxSubreaper => {
                        receipt.descendants_reaped_to_echild
                            && receipt
                                .linux_boot_id
                                .as_ref()
                                .is_some_and(|value| !value.is_empty())
                            && receipt.owner.start_time_ticks.is_some()
                            && receipt.supervisor.start_time_ticks.is_some()
                    }
                    CompositionSupervisorBackend::UnixProcessGroup
                    | CompositionSupervisorBackend::WindowsProcessGroup => {
                        !receipt.descendants_reaped_to_echild
                    }
                }
        })
        && attempt.execution_outcome == CompositionExecutionOutcome::Passed
        && attempt.execution_evidence_complete
        && attempt.cleanup_disposition == CompositionCleanupDisposition::Cleaned
        && attempt.owner_termination_signal.is_none()
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
        && attempt.active_process_group_identity.is_none()
        && !attempt.owned_tree_termination_unknown
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

pub fn execution_records_digest(records: &[CompositionExecutionRecord]) -> Digest {
    let bytes = serde_json::to_vec(records)
        .expect("composition execution records always serialize to JSON");
    Digest::sha256_bytes(&bytes)
}

pub fn new_composition_run_nonce() -> String {
    let sequence = NEXT_COMPOSITION_PARENT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    Digest::sha256_bytes(
        format!("{}:{}:{sequence}", std::process::id(), now_unix_nanos()).as_bytes(),
    )
    .to_string()
}

pub fn new_supervised_composition_attempt_id(input: &CompositionInput, run_nonce: &str) -> String {
    let identity = serde_json::to_vec(&(&input.binding, &input.identity, run_nonce))
        .expect("composition attempt identity always serializes to JSON");
    format!("composition-{}", Digest::sha256_bytes(&identity))
}

fn persist_owner_interruption(
    state_dir: &Path,
    attempt: &mut CompositionAttempt,
    signal: i32,
) -> Result<(), CompositionError> {
    attempt.owner_termination_signal = Some(signal);
    attempt.passed = false;
    attempt.failure = Some(format!("composition_interrupted:owner_signal={signal}"));
    persist_attempt(state_dir, attempt)
}

fn reconciliation_trace(message: &str) {
    if std::env::var("AI_COCKPIT_COMPOSITION_RECONCILE_TRACE").as_deref() == Ok("1") {
        eprintln!("composition_reconcile_trace {message}");
    }
}

fn reconciliation_error_category(error: &str) -> &'static str {
    let error = error
        .strip_prefix("verifier_process_state_unknown:")
        .unwrap_or(error);
    if let Some(rest) = error.strip_prefix("cannot inspect process pid=") {
        let top_level = rest
            .split_once(" identity_before=")
            .map_or(rest, |(top_level, _)| top_level);
        let error_kind = top_level
            .split_whitespace()
            .find_map(|field| field.strip_prefix("error_kind="));
        return if error_kind == Some("PermissionDenied") {
            "permission_denied"
        } else {
            "proc_observation_error"
        };
    }
    if error.contains("error_kind=PermissionDenied") || error.contains("Permission denied") {
        "permission_denied"
    } else if error.starts_with("cannot inspect process ") {
        "permission_denied"
    } else if error.contains("digest changed") || error.contains("attempt changed") {
        "attempt_changed"
    } else if error.contains("cannot parse current composition attempt") {
        "attempt_parse_error"
    } else if error.contains("time budget") || error.contains("deadline") {
        "deadline"
    } else if error.contains("entry budget") {
        "entry_budget"
    } else if error.contains("directory race") {
        "directory_race"
    } else if error.contains("cannot enumerate proc root") {
        "proc_enumeration_error"
    } else if error.contains("could not read") {
        "stat_read_error"
    } else if error.contains("could not parse") {
        "stat_parse_error"
    } else if error.contains("disappeared") {
        "stat_disappeared"
    } else if error.contains("inconsistent") {
        "identity_inconsistent"
    } else {
        "other"
    }
}

#[cfg(all(test, target_os = "linux"))]
static LINUX_PROCESS_OBSERVATION_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn reconciliation_known_observer_error(error: &str) -> String {
    let error = error
        .strip_prefix("verifier_process_state_unknown:")
        .unwrap_or(error);
    if let Some(rest) = error.strip_prefix("cannot inspect process pid=") {
        let Some((process_id, fields)) = rest.split_once(' ') else {
            return "opaque".into();
        };
        let field = |name: &str| {
            fields
                .split_whitespace()
                .find_map(|part| part.strip_prefix(name))
        };
        if let (Some(phase), Some(errno), Some(identity_state)) =
            (field("phase="), field("errno="), field("identity_state="))
        {
            return format!(
                "pid={process_id} phase={phase} errno={errno} identity_state={identity_state}"
            );
        }
        return "opaque".into();
    }
    let Some(rest) = error.strip_prefix("cannot inspect process ") else {
        return "opaque".into();
    };
    for suffix in [" working directory", " file descriptors", " open files"] {
        if let Some(pid) = rest.strip_suffix(suffix)
            && let Ok(pid) = pid.parse::<u32>()
        {
            return format!("pid={pid} area={}", suffix.trim());
        }
    }
    "opaque".into()
}

fn reconcile_abandoned_attempt_with_observer(
    repository_root: &Path,
    state_dir: &Path,
    previous: Option<CompositionAttempt>,
    observe_worktree_process: &dyn Fn(&Path) -> Result<Option<u32>, String>,
) -> Result<Option<CompositionAttempt>, CompositionError> {
    let Some(mut attempt) = previous else {
        return Ok(None);
    };
    reconciliation_trace(&format!(
        "stage=loaded attempt={} schema={} owner_pid={:?} observation_schema={} group_id={:?} leader={:?} owned_tree_unknown={}",
        attempt.attempt_id,
        attempt.schema_version,
        attempt.owner_pid,
        attempt.process_observation_schema_version,
        attempt.active_process_group_id,
        attempt.active_process_group_identity,
        attempt.owned_tree_termination_unknown,
    ));
    // External Ok(None) cannot replace a missing proof about descendants
    // owned by the prior supervisor. Keep the old tree and registration until
    // an explicit supported recovery route supplies that proof.
    if !attempt.isolated_worktree.is_empty()
        && (attempt.owned_tree_termination_unknown
            || (attempt.cleanup_disposition == CompositionCleanupDisposition::Retained
                && attempt.execution_outcome == CompositionExecutionOutcome::Unknown
                && attempt
                    .failure
                    .as_deref()
                    .is_some_and(|failure| failure.starts_with("verifier_process_state_unknown:")))
            || attempt.supervisor_receipt.as_ref().is_some_and(|receipt| {
                receipt.backend == CompositionSupervisorBackend::LinuxSubreaper
                    && !receipt.descendants_reaped_to_echild
            }))
    {
        reconciliation_trace("stage=prior_owned_termination_unknown");
        return Err(CompositionError::UnknownAttemptOwner {
            attempt_id: attempt.attempt_id,
        });
    }
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
        reconciliation_trace("stage=missing_owner_pid");
        return Err(CompositionError::UnknownAttemptOwner {
            attempt_id: attempt.attempt_id,
        });
    };
    #[cfg(unix)]
    let (owner_alive, owner_probe_result, owner_probe_errno) = process_liveness_probe(owner_pid);
    #[cfg(unix)]
    reconciliation_trace(&format!(
        "stage=owner_probe owner_pid={owner_pid} kill_result={owner_probe_result} errno={owner_probe_errno:?} alive={owner_alive}"
    ));
    #[cfg(windows)]
    let owner_alive = process_is_alive(owner_pid);
    if owner_alive {
        return Err(CompositionError::ActiveAttempt {
            attempt_id: attempt.attempt_id,
            owner_pid,
        });
    }

    if attempt.failure.as_deref() == Some("in_progress")
        && attempt.process_observation_schema_version < PROCESS_OBSERVATION_SCHEMA_VERSION
    {
        reconciliation_trace("stage=legacy_observation_schema");
        return Err(CompositionError::UnknownAttemptOwner {
            attempt_id: attempt.attempt_id,
        });
    }

    #[cfg(target_os = "linux")]
    let mut attempt_digest_before_group_cleanup = None;
    match (
        attempt.active_execution_node.as_deref(),
        attempt.active_process_group_id,
    ) {
        (Some(_), Some(process_group_id)) => {
            #[cfg(target_os = "linux")]
            {
                match observe_linux_process_group_for_recovery(
                    state_dir,
                    &attempt,
                    process_group_id,
                ) {
                    Ok((LinuxProcessGroupLiveness::Active, _)) => {
                        return Err(CompositionError::ActiveVerifierProcessGroup {
                            attempt_id: attempt.attempt_id,
                            process_group_id,
                        });
                    }
                    Ok((LinuxProcessGroupLiveness::Exited, digest)) => {
                        attempt_digest_before_group_cleanup = Some(digest);
                    }
                    Ok((LinuxProcessGroupLiveness::Unknown, _)) => {
                        reconciliation_trace("stage=group_observation_unknown");
                        return Err(CompositionError::UnknownAttemptOwner {
                            attempt_id: attempt.attempt_id,
                        });
                    }
                    Err(error) => {
                        reconciliation_trace(&format!(
                            "stage=group_observation_error category={}",
                            reconciliation_error_category(&error)
                        ));
                        return Err(CompositionError::UnknownAttemptOwner {
                            attempt_id: attempt.attempt_id,
                        });
                    }
                }
            }
            #[cfg(not(target_os = "linux"))]
            if process_group_is_alive(process_group_id) {
                return Err(CompositionError::ActiveVerifierProcessGroup {
                    attempt_id: attempt.attempt_id,
                    process_group_id,
                });
            }
            attempt.active_execution_node = None;
            attempt.active_process_group_id = None;
            attempt.active_process_group_identity = None;
        }
        (Some(_), None) | (None, Some(_)) => {
            reconciliation_trace("stage=incomplete_active_group_fields");
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
        match observe_worktree_process(worktree) {
            Ok(Some(process_id)) => {
                return Err(CompositionError::ActiveVerifierDescendant {
                    attempt_id: attempt.attempt_id,
                    process_id,
                });
            }
            Err(error) => {
                reconciliation_trace(&format!(
                    "stage=worktree_observer_error category={} detail={}",
                    reconciliation_error_category(&error),
                    reconciliation_known_observer_error(&error)
                ));
                return Err(CompositionError::UnknownAttemptOwner {
                    attempt_id: attempt.attempt_id,
                });
            }
            Ok(None) => {}
        }
    }
    #[cfg(target_os = "linux")]
    if let Some(expected_digest) = attempt_digest_before_group_cleanup
        && require_attempt_digest_unchanged(state_dir, &attempt.attempt_id, &expected_digest)
            .is_err()
    {
        reconciliation_trace("stage=attempt_digest_changed_after_group_observation");
        return Err(CompositionError::UnknownAttemptOwner {
            attempt_id: attempt.attempt_id,
        });
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
    set_cleanup_disposition(&mut attempt);
    if let Some(signal) = attempt.owner_termination_signal {
        attempt.failure = Some(format!("interrupted_owner_terminated:signal={signal}"));
    } else if attempt.failure.as_deref() == Some("in_progress") {
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
    let (pid, suffix) = suffix.split_once('-')?;
    let mut components = suffix.split('-');
    let timestamp = components.next()?.parse::<u128>().ok()?;
    let sequence_is_valid = match components.next() {
        Some(sequence) => sequence.parse::<u64>().is_ok(),
        None => true,
    };
    if pid.parse::<u32>().ok()? != owner_pid
        || timestamp == 0
        || !sequence_is_valid
        || components.next().is_some()
    {
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

#[cfg(all(unix, not(target_os = "linux")))]
fn process_is_alive(pid: u32) -> bool {
    process_liveness_probe(pid).0
}

#[cfg(unix)]
fn process_liveness_probe(pid: u32) -> (bool, i32, Option<i32>) {
    // SAFETY: `kill(pid, 0)` performs no signal delivery and only probes the
    // process table. A permission error is treated as live, failing closed.
    let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
    if result == 0 {
        return (true, result, None);
    }
    let errno = std::io::Error::last_os_error().raw_os_error();
    (errno != Some(libc::ESRCH), result, errno)
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

#[cfg(all(unix, not(target_os = "linux")))]
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
#[derive(Clone, Debug, PartialEq, Eq)]
struct LinuxProcessGroupMember {
    process_id: u32,
    state: char,
    start_time_ticks: u64,
    process_group_id: u32,
    session_id: u32,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LinuxProcessGroupLiveness {
    Active,
    Exited,
    Unknown,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Error)]
enum LinuxProcessGroupScanError {
    #[error("proc scan could not read {path} after enumeration: {source}")]
    StatEntryDisappeared {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{0}")]
    Other(String),
}

#[cfg(target_os = "linux")]
const MAX_LINUX_PROC_SCAN_ENTRIES: usize = 65_536;
#[cfg(target_os = "linux")]
const MAX_LINUX_PROC_SCAN_DURATION: Duration = Duration::from_millis(500);
#[cfg(target_os = "linux")]
const LINUX_PROC_STABILITY_DELAY: Duration = Duration::from_millis(10);
#[cfg(target_os = "linux")]
const MAX_LINUX_PROC_GROUP_SCAN_ATTEMPTS: usize = 3;
#[cfg(target_os = "linux")]
const MAX_LINUX_PROC_GROUP_OBSERVATION_DURATION: Duration = Duration::from_secs(3);

#[cfg(target_os = "linux")]
fn trace_linux_group_scan(
    process_group_id: u32,
    attempt_index: usize,
    phase: &str,
    result: &Result<Vec<LinuxProcessGroupMember>, LinuxProcessGroupScanError>,
) {
    if std::env::var("AI_COCKPIT_COMPOSITION_RECONCILE_TRACE").as_deref() != Ok("1") {
        return;
    }
    match result {
        Ok(members) => {
            let bounded_members = members
                .iter()
                .take(8)
                .map(|member| {
                    format!(
                        "{}:{}:{}:{}:{}",
                        member.process_id,
                        member.state,
                        member.start_time_ticks,
                        member.process_group_id,
                        member.session_id
                    )
                })
                .collect::<Vec<_>>();
            reconciliation_trace(&format!(
                "stage=group_scan pgid={process_group_id} attempt={} phase={phase} target_member_count={} target_members={bounded_members:?}",
                attempt_index + 1,
                members.len()
            ));
        }
        Err(error) => {
            let category = match error {
                LinuxProcessGroupScanError::StatEntryDisappeared { .. } => "stat_disappeared",
                LinuxProcessGroupScanError::Other(message) => {
                    reconciliation_error_category(message)
                }
            };
            reconciliation_trace(&format!(
                "stage=group_scan pgid={process_group_id} attempt={} phase={phase} error_category={category}",
                attempt_index + 1
            ));
        }
    }
}

#[cfg(target_os = "linux")]
fn parse_linux_process_stat(
    process_id: u32,
    stat: &str,
) -> Result<LinuxProcessGroupMember, String> {
    let command_open = stat
        .find('(')
        .ok_or_else(|| "proc stat is missing command opener".to_string())?;
    let parsed_process_id = stat[..command_open]
        .trim()
        .parse::<u32>()
        .map_err(|_| "proc stat has an invalid process id".to_string())?;
    if parsed_process_id != process_id {
        return Err("proc stat process id does not match its directory".into());
    }
    let command_close = stat
        .rfind(')')
        .filter(|close| *close > command_open)
        .ok_or_else(|| "proc stat is missing command terminator".to_string())?;
    let fields = stat[command_close + 1..]
        .split_whitespace()
        .collect::<Vec<_>>();
    if fields.len() <= 19 {
        return Err("proc stat has too few fields".into());
    }
    let state = fields[0]
        .chars()
        .next()
        .ok_or_else(|| "proc stat has an empty state".to_string())?;
    let process_group_id = fields[2]
        .parse::<u32>()
        .map_err(|_| "proc stat has an invalid process group id".to_string())?;
    let session_id = fields[3]
        .parse::<u32>()
        .map_err(|_| "proc stat has an invalid session id".to_string())?;
    let start_time_ticks = fields[19]
        .parse::<u64>()
        .map_err(|_| "proc stat has an invalid starttime".to_string())?;
    Ok(LinuxProcessGroupMember {
        process_id,
        state,
        start_time_ticks,
        process_group_id,
        session_id,
    })
}

#[cfg(target_os = "linux")]
fn parse_linux_process_parent_id(process_id: u32, stat: &str) -> Result<u32, String> {
    let command_open = stat
        .find('(')
        .ok_or_else(|| "proc stat is missing command opener".to_string())?;
    let parsed_process_id = stat[..command_open]
        .trim()
        .parse::<u32>()
        .map_err(|_| "proc stat has an invalid process id".to_string())?;
    if parsed_process_id != process_id {
        return Err("proc stat process id does not match its directory".into());
    }
    let command_close = stat
        .rfind(')')
        .filter(|close| *close > command_open)
        .ok_or_else(|| "proc stat is missing command terminator".to_string())?;
    let fields = stat[command_close + 1..]
        .split_whitespace()
        .collect::<Vec<_>>();
    fields
        .get(1)
        .ok_or_else(|| "proc stat has too few fields to read parent PID".to_string())?
        .parse::<u32>()
        .map_err(|_| "proc stat has an invalid parent PID".to_string())
}

#[cfg(target_os = "linux")]
fn read_linux_process_group_leader_identity(
    proc_root: &Path,
    process_group_id: u32,
) -> Result<ProcessGroupLeaderIdentity, String> {
    let stat_path = proc_root.join(process_group_id.to_string()).join("stat");
    let stat = fs::read_to_string(&stat_path).map_err(|error| {
        format!(
            "cannot read process-group leader stat {}: {error}",
            stat_path.display()
        )
    })?;
    let leader = parse_linux_process_stat(process_group_id, &stat)?;
    if leader.process_id != process_group_id
        || leader.process_group_id != process_group_id
        || leader.start_time_ticks == 0
    {
        return Err("observed process is not the expected process-group leader".into());
    }
    Ok(ProcessGroupLeaderIdentity {
        leader_pid: leader.process_id,
        leader_start_time_ticks: leader.start_time_ticks,
        process_group_id: leader.process_group_id,
        session_id: leader.session_id,
    })
}

#[cfg(target_os = "linux")]
fn scan_linux_process_group_once(
    proc_root: &Path,
    process_group_id: u32,
    maximum_entries: usize,
    scan_deadline: Instant,
) -> Result<Vec<LinuxProcessGroupMember>, LinuxProcessGroupScanError> {
    let entries = fs::read_dir(proc_root).map_err(|error| {
        LinuxProcessGroupScanError::Other(format!(
            "cannot enumerate proc root {}: {error}",
            proc_root.display()
        ))
    })?;
    let mut numeric_entries = 0usize;
    let mut members = Vec::new();
    for entry in entries {
        if Instant::now() >= scan_deadline {
            return Err(LinuxProcessGroupScanError::Other(
                "proc scan exceeded its time budget".into(),
            ));
        }
        let entry = entry.map_err(|error| {
            LinuxProcessGroupScanError::Other(format!("proc scan directory race: {error}"))
        })?;
        let Some(process_id) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        numeric_entries = numeric_entries.saturating_add(1);
        if numeric_entries > maximum_entries {
            return Err(LinuxProcessGroupScanError::Other(
                "proc scan exceeded its entry budget".into(),
            ));
        }
        let stat_path = entry.path().join("stat");
        let stat = fs::read_to_string(&stat_path).map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                LinuxProcessGroupScanError::StatEntryDisappeared {
                    path: stat_path.clone(),
                    source,
                }
            } else {
                LinuxProcessGroupScanError::Other(format!(
                    "proc scan could not read {}: {source}",
                    stat_path.display()
                ))
            }
        })?;
        let process = parse_linux_process_stat(process_id, &stat).map_err(|error| {
            LinuxProcessGroupScanError::Other(format!(
                "proc scan could not parse {}: {error}",
                stat_path.display()
            ))
        })?;
        if process.process_group_id == process_group_id {
            members.push(process);
        }
    }
    if Instant::now() >= scan_deadline {
        return Err(LinuxProcessGroupScanError::Other(
            "proc scan exceeded its time budget".into(),
        ));
    }
    members.sort_by_key(|member| {
        (
            member.process_id,
            member.start_time_ticks,
            member.process_group_id,
        )
    });
    Ok(members)
}

#[cfg(target_os = "linux")]
fn classify_linux_process_group_scans(
    process_group_id: u32,
    leader_identity: Option<&ProcessGroupLeaderIdentity>,
    first: &[LinuxProcessGroupMember],
    second: &[LinuxProcessGroupMember],
) -> LinuxProcessGroupLiveness {
    let identity_keys = |members: &[LinuxProcessGroupMember]| {
        members
            .iter()
            .map(|member| {
                (
                    member.process_id,
                    member.start_time_ticks,
                    member.process_group_id,
                    member.session_id,
                )
            })
            .collect::<Vec<_>>()
    };
    if identity_keys(first) != identity_keys(second) {
        return LinuxProcessGroupLiveness::Unknown;
    }
    if first.is_empty() {
        return LinuxProcessGroupLiveness::Exited;
    }
    let Some(leader_identity) = leader_identity else {
        return LinuxProcessGroupLiveness::Unknown;
    };
    if leader_identity.leader_pid != process_group_id
        || leader_identity.process_group_id != process_group_id
        || leader_identity.leader_start_time_ticks == 0
        || leader_identity.session_id == 0
    {
        return LinuxProcessGroupLiveness::Unknown;
    }
    let Some(leader) = first
        .iter()
        .find(|member| member.process_id == leader_identity.leader_pid)
    else {
        return LinuxProcessGroupLiveness::Unknown;
    };
    if leader.start_time_ticks != leader_identity.leader_start_time_ticks
        || leader.process_group_id != leader_identity.process_group_id
        || leader.session_id != leader_identity.session_id
        || first
            .iter()
            .any(|member| member.session_id != leader_identity.session_id)
    {
        return LinuxProcessGroupLiveness::Unknown;
    }
    if first
        .iter()
        .chain(second.iter())
        .any(|member| !matches!(member.state, 'Z' | 'X' | 'x'))
    {
        LinuxProcessGroupLiveness::Active
    } else {
        LinuxProcessGroupLiveness::Exited
    }
}

#[cfg(target_os = "linux")]
fn observe_linux_process_group_with_retries<S, V, W, N>(
    process_group_id: u32,
    leader_identity: Option<&ProcessGroupLeaderIdentity>,
    total_deadline: Instant,
    mut scan_once: S,
    mut validate_attempt: V,
    mut wait: W,
    mut now: N,
) -> Result<LinuxProcessGroupLiveness, String>
where
    S: FnMut(Instant) -> Result<Vec<LinuxProcessGroupMember>, LinuxProcessGroupScanError>,
    V: FnMut() -> Result<(), String>,
    W: FnMut(Duration),
    N: FnMut() -> Instant,
{
    let deadline_error = "process-group observation exceeded its total deadline";
    for attempt_index in 0..MAX_LINUX_PROC_GROUP_SCAN_ATTEMPTS {
        if now() >= total_deadline {
            return Err(deadline_error.into());
        }
        validate_attempt()?;
        if now() >= total_deadline {
            return Err(deadline_error.into());
        }

        let first_scan_deadline = (now() + MAX_LINUX_PROC_SCAN_DURATION).min(total_deadline);
        let first_result = scan_once(first_scan_deadline);
        trace_linux_group_scan(process_group_id, attempt_index, "first", &first_result);
        if now() >= total_deadline {
            validate_attempt()?;
            return Err(deadline_error.into());
        }
        let first = match first_result {
            Ok(members) => members,
            Err(error) => {
                validate_attempt()?;
                if now() >= total_deadline {
                    return Err(deadline_error.into());
                }
                match error {
                    LinuxProcessGroupScanError::StatEntryDisappeared { .. }
                        if attempt_index + 1 < MAX_LINUX_PROC_GROUP_SCAN_ATTEMPTS =>
                    {
                        continue;
                    }
                    error => return Err(error.to_string()),
                }
            }
        };

        if total_deadline.saturating_duration_since(now()) < LINUX_PROC_STABILITY_DELAY {
            validate_attempt()?;
            return Err(deadline_error.into());
        }
        wait(LINUX_PROC_STABILITY_DELAY);
        if now() >= total_deadline {
            validate_attempt()?;
            return Err(deadline_error.into());
        }

        let second_scan_deadline = (now() + MAX_LINUX_PROC_SCAN_DURATION).min(total_deadline);
        let second_result = scan_once(second_scan_deadline);
        trace_linux_group_scan(process_group_id, attempt_index, "second", &second_result);
        if now() >= total_deadline {
            validate_attempt()?;
            return Err(deadline_error.into());
        }
        let second = match second_result {
            Ok(members) => members,
            Err(error) => {
                validate_attempt()?;
                if now() >= total_deadline {
                    return Err(deadline_error.into());
                }
                match error {
                    LinuxProcessGroupScanError::StatEntryDisappeared { .. }
                        if attempt_index + 1 < MAX_LINUX_PROC_GROUP_SCAN_ATTEMPTS =>
                    {
                        continue;
                    }
                    error => return Err(error.to_string()),
                }
            }
        };

        let liveness =
            classify_linux_process_group_scans(process_group_id, leader_identity, &first, &second);
        reconciliation_trace(&format!(
            "stage=group_classification pgid={process_group_id} attempt={} leader={leader_identity:?} liveness={liveness:?}",
            attempt_index + 1
        ));
        validate_attempt()?;
        if now() >= total_deadline {
            return Err(deadline_error.into());
        }
        return match liveness {
            LinuxProcessGroupLiveness::Unknown => {
                Err("paired process-group member set or leader identity is inconsistent".into())
            }
            LinuxProcessGroupLiveness::Active | LinuxProcessGroupLiveness::Exited => Ok(liveness),
        };
    }
    Err("process-group observation exhausted its retry budget".into())
}

#[cfg(target_os = "linux")]
fn attempt_file_bytes(state_dir: &Path, attempt_id: &str) -> Result<Vec<u8>, String> {
    let path = attempt_record_path(state_dir, attempt_id);
    let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err("composition attempt is not a regular non-symlink file".into());
    }
    fs::read(&path).map_err(|error| error.to_string())
}

#[cfg(target_os = "linux")]
fn current_attempt_file_digest(state_dir: &Path, attempt_id: &str) -> Result<Digest, String> {
    Ok(Digest::sha256_bytes(&attempt_file_bytes(
        state_dir, attempt_id,
    )?))
}

#[cfg(target_os = "linux")]
fn require_attempt_digest_unchanged(
    state_dir: &Path,
    attempt_id: &str,
    expected: &Digest,
) -> Result<(), String> {
    if &current_attempt_file_digest(state_dir, attempt_id)? == expected {
        Ok(())
    } else {
        Err("latest composition attempt digest changed".into())
    }
}

#[cfg(target_os = "linux")]
fn observe_linux_process_group_for_recovery(
    state_dir: &Path,
    attempt: &CompositionAttempt,
    process_group_id: u32,
) -> Result<(LinuxProcessGroupLiveness, Digest), String> {
    let total_deadline = Instant::now() + MAX_LINUX_PROC_GROUP_OBSERVATION_DURATION;
    let initial_bytes = attempt_file_bytes(state_dir, &attempt.attempt_id)?;
    let stored_attempt: CompositionAttempt = serde_json::from_slice(&initial_bytes)
        .map_err(|error| format!("cannot parse current composition attempt: {error}"))?;
    if stored_attempt != *attempt
        || stored_attempt.active_process_group_id != Some(process_group_id)
    {
        reconciliation_trace("stage=initial_attempt_fields_changed");
        return Err(
            "latest composition attempt changed before process-group reconciliation".into(),
        );
    }
    let initial_digest = Digest::sha256_bytes(&initial_bytes);
    let validate_attempt = || -> Result<(), String> {
        let current_bytes =
            attempt_file_bytes(state_dir, &attempt.attempt_id).inspect_err(|error| {
                reconciliation_trace(&format!(
                    "stage=attempt_validation_read_error category={}",
                    reconciliation_error_category(error)
                ));
            })?;
        if Digest::sha256_bytes(&current_bytes) != initial_digest {
            reconciliation_trace("stage=attempt_validation_digest_changed");
            return Err("latest composition attempt digest changed".into());
        }
        let current_attempt: CompositionAttempt = serde_json::from_slice(&current_bytes)
            .map_err(|error| format!("cannot parse current composition attempt: {error}"))?;
        if current_attempt != *attempt
            || current_attempt.active_process_group_id != Some(process_group_id)
        {
            reconciliation_trace("stage=attempt_validation_fields_changed");
            return Err(
                "latest composition attempt changed during process-group observation".into(),
            );
        }
        Ok(())
    };
    let liveness = observe_linux_process_group_with_retries(
        process_group_id,
        attempt.active_process_group_identity.as_ref(),
        total_deadline,
        |scan_deadline| {
            scan_linux_process_group_once(
                Path::new("/proc"),
                process_group_id,
                MAX_LINUX_PROC_SCAN_ENTRIES,
                scan_deadline,
            )
        },
        validate_attempt,
        std::thread::sleep,
        Instant::now,
    );
    Ok((liveness?, initial_digest))
}

#[cfg(target_os = "linux")]
fn verifier_process_using_worktree(worktree: &Path) -> Result<Option<u32>, String> {
    use std::os::unix::fs::MetadataExt;

    let worktree = fs::canonicalize(worktree).map_err(|error| error.to_string())?;
    // Procfs may deny a caller access to its runner/launcher ancestors. Those
    // processes existed before the private composition directory was created
    // and cannot have inherited its cwd or file descriptors. Keep inspecting
    // every process that could have inherited or opened this private path so
    // escaped verifiers still protect it.
    let caller_ancestors = linux_process_ancestor_ids(Path::new("/proc"), std::process::id())?;
    let private_created_at = private_worktree_created_at_unix_nanos(&worktree);
    let proc_entries = fs::read_dir("/proc").map_err(|error| error.to_string())?;
    'processes: for entry in proc_entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let Some(process_id) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        if process_id == std::process::id() || caller_ancestors.contains(&process_id) {
            continue;
        }
        let process_dir = entry.path();
        let metadata = match fs::metadata(&process_dir) {
            Ok(metadata) if metadata.uid() == unsafe { libc::getuid() } => metadata,
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                let identity_before =
                    Err("process identity was not observed before metadata".into());
                return Err(linux_process_observation_error(
                    Path::new("/proc"),
                    process_id,
                    None,
                    "procfs.process-metadata",
                    &error,
                    &identity_before,
                ));
            }
        };
        // This is the proc-directory UID used to select same-UID candidates;
        // it is not part of, or cryptographically bound to, the stat identity.
        let process_uid = metadata.uid();
        let identity_before = read_linux_process_identity(Path::new("/proc"), process_id);
        let cwd = process_dir.join("cwd");
        match fs::read_link(&cwd) {
            Ok(path) if path_is_within(&worktree, &path) => return Ok(Some(process_id)),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                if private_created_at.is_some_and(|created_at| {
                    linux_process_started_before(Path::new("/proc"), process_id, created_at)
                        .unwrap_or(false)
                }) {
                    continue 'processes;
                }
                return Err(linux_process_observation_error(
                    Path::new("/proc"),
                    process_id,
                    Some(process_uid),
                    "procfs.cwd",
                    &error,
                    &identity_before,
                ));
            }
            Err(error) => {
                return Err(linux_process_observation_error(
                    Path::new("/proc"),
                    process_id,
                    Some(process_uid),
                    "procfs.cwd",
                    &error,
                    &identity_before,
                ));
            }
        }
        let descriptors = process_dir.join("fd");
        let descriptors = match fs::read_dir(&descriptors) {
            Ok(descriptors) => descriptors,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                if private_created_at.is_some_and(|created_at| {
                    linux_process_started_before(Path::new("/proc"), process_id, created_at)
                        .unwrap_or(false)
                }) {
                    continue 'processes;
                }
                return Err(linux_process_observation_error(
                    Path::new("/proc"),
                    process_id,
                    Some(process_uid),
                    "procfs.fd-directory",
                    &error,
                    &identity_before,
                ));
            }
            Err(error) => {
                return Err(linux_process_observation_error(
                    Path::new("/proc"),
                    process_id,
                    Some(process_uid),
                    "procfs.fd-directory",
                    &error,
                    &identity_before,
                ));
            }
        };
        for descriptor in descriptors {
            let descriptor = descriptor.map_err(|error| {
                linux_process_observation_error(
                    Path::new("/proc"),
                    process_id,
                    Some(process_uid),
                    "procfs.fd-entry",
                    &error,
                    &identity_before,
                )
            })?;
            let descriptor_name = descriptor.file_name();
            let descriptor_phase =
                format!("procfs.fd-target:{}", descriptor_name.to_string_lossy());
            match fs::read_link(descriptor.path()) {
                Ok(path) if path_is_within(&worktree, &path) => return Ok(Some(process_id)),
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                    if private_created_at.is_some_and(|created_at| {
                        linux_process_started_before(Path::new("/proc"), process_id, created_at)
                            .unwrap_or(false)
                    }) {
                        continue 'processes;
                    }
                    return Err(linux_process_observation_error(
                        Path::new("/proc"),
                        process_id,
                        Some(process_uid),
                        &descriptor_phase,
                        &error,
                        &identity_before,
                    ));
                }
                Err(error) => {
                    return Err(linux_process_observation_error(
                        Path::new("/proc"),
                        process_id,
                        Some(process_uid),
                        &descriptor_phase,
                        &error,
                        &identity_before,
                    ));
                }
            }
        }
    }
    Ok(None)
}

#[cfg(target_os = "linux")]
fn read_linux_process_identity(
    proc_root: &Path,
    process_id: u32,
) -> Result<LinuxProcessGroupMember, String> {
    let stat_path = proc_root.join(process_id.to_string()).join("stat");
    let stat = fs::read_to_string(&stat_path).map_err(|error| {
        let errno = error
            .raw_os_error()
            .map_or_else(|| "unknown".to_owned(), |value| value.to_string());
        format!(
            "identity stat read failed error_kind={:?} errno={errno}: {error}",
            error.kind()
        )
    })?;
    parse_linux_process_stat(process_id, &stat)
        .map_err(|error| format!("identity stat parse failed: {error}"))
}

#[cfg(target_os = "linux")]
fn linux_process_identity_matches(
    before: &LinuxProcessGroupMember,
    after: &LinuxProcessGroupMember,
) -> bool {
    before.process_id == after.process_id
        && before.start_time_ticks == after.start_time_ticks
        && before.process_group_id == after.process_group_id
        && before.session_id == after.session_id
}

#[cfg(target_os = "linux")]
fn render_linux_process_identity(identity: &Result<LinuxProcessGroupMember, String>) -> String {
    match identity {
        Ok(identity) => format!(
            "observed(pid={},starttime_ticks={},pgid={},sid={},state={})",
            identity.process_id,
            identity.start_time_ticks,
            identity.process_group_id,
            identity.session_id,
            identity.state
        ),
        Err(error) => format!("unknown({error})"),
    }
}

#[cfg(target_os = "linux")]
fn linux_process_observation_error(
    proc_root: &Path,
    process_id: u32,
    process_uid: Option<u32>,
    phase: &str,
    error: &std::io::Error,
    identity_before: &Result<LinuxProcessGroupMember, String>,
) -> String {
    let identity_after = read_linux_process_identity(proc_root, process_id);
    let identity_state = match (identity_before, &identity_after) {
        (Ok(before), Ok(after)) if linux_process_identity_matches(before, after) => "observed",
        _ => "unknown",
    };
    let errno = error
        .raw_os_error()
        .map_or_else(|| "unknown".to_owned(), |value| value.to_string());
    let legacy_area = match phase {
        "procfs.cwd" => " working directory",
        "procfs.fd-directory" => " file descriptors",
        phase if phase.starts_with("procfs.fd-target:") => " open files",
        _ => "",
    };
    format!(
        "cannot inspect process pid={process_id} filter_uid={process_uid:?} phase={phase} error_kind={:?} errno={errno} identity_state={identity_state} identity_before={} identity_after={} message={error}{legacy_area}",
        error.kind(),
        render_linux_process_identity(identity_before),
        render_linux_process_identity(&identity_after),
    )
}

#[cfg(target_os = "linux")]
fn private_worktree_created_at_unix_nanos(worktree: &Path) -> Option<u128> {
    if worktree.file_name()?.to_str()? != "composition" {
        return None;
    }
    let parent_name = worktree.parent()?.file_name()?.to_str()?;
    let suffix = parent_name.strip_prefix("ai-cockpit-composition-")?;
    let mut components = suffix.split('-');
    components.next()?.parse::<u32>().ok()?;
    let created_at = components.next()?.parse::<u128>().ok()?;
    components.next()?.parse::<u64>().ok()?;
    components.next().is_none().then_some(created_at)
}

#[cfg(target_os = "linux")]
fn linux_process_started_before(
    proc_root: &Path,
    process_id: u32,
    private_created_at_unix_nanos: u128,
) -> Result<bool, String> {
    let stat_path = proc_root.join(process_id.to_string()).join("stat");
    let stat = match fs::read_to_string(&stat_path) {
        Ok(stat) => stat,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(true),
        Err(error) => return Err(error.to_string()),
    };
    let command_end = stat
        .rfind(')')
        .ok_or_else(|| format!("cannot inspect process {process_id} start identity"))?;
    // Fields after the closing command name begin with field 3 (state); field
    // 22 (starttime) is therefore the twentieth whitespace-delimited value.
    let start_ticks = stat[command_end + 1..]
        .split_whitespace()
        .nth(19)
        .and_then(|value| value.parse::<u128>().ok())
        .ok_or_else(|| format!("cannot inspect process {process_id} start time"))?;
    let boot_time_seconds = fs::read_to_string(proc_root.join("stat"))
        .map_err(|error| error.to_string())?
        .lines()
        .find_map(|line| line.strip_prefix("btime "))
        .and_then(|value| value.parse::<u128>().ok())
        .ok_or_else(|| "cannot inspect Linux boot time".to_owned())?;
    // SAFETY: sysconf is a read-only query for the kernel clock-tick rate.
    let ticks_per_second = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if ticks_per_second <= 0 {
        return Err("cannot inspect Linux process clock-tick rate".into());
    }
    let process_started_at = boot_time_seconds
        .saturating_mul(1_000_000_000)
        .saturating_add(start_ticks.saturating_mul(1_000_000_000) / ticks_per_second as u128);
    // /proc/stat's boot-time epoch has one-second resolution. Require a full
    // two-second separation before treating an inaccessible process as
    // pre-existing; recent or unobservable processes remain fail-closed.
    Ok(process_started_at.saturating_add(2_000_000_000) < private_created_at_unix_nanos)
}

#[cfg(target_os = "linux")]
fn linux_process_ancestor_ids(proc_root: &Path, process_id: u32) -> Result<BTreeSet<u32>, String> {
    let mut ancestors = BTreeSet::new();
    let mut current = process_id;
    loop {
        let status_path = proc_root.join(current.to_string()).join("status");
        let status = match fs::read_to_string(&status_path) {
            Ok(status) => status,
            Err(error) if current != process_id && error.kind() == std::io::ErrorKind::NotFound => {
                break;
            }
            Err(error) => return Err(error.to_string()),
        };
        let parent = status
            .lines()
            .find_map(|line| {
                line.strip_prefix("PPid:")
                    .and_then(|value| value.trim().parse::<u32>().ok())
            })
            .ok_or_else(|| format!("cannot inspect process {current} parent identity"))?;
        if parent == 0 || parent == current || !ancestors.insert(parent) {
            break;
        }
        current = parent;
    }
    Ok(ancestors)
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
    let candidate = Path::new(candidate);
    #[cfg(windows)]
    {
        let mut candidate_components = candidate.components();
        root.components().all(|root_component| {
            candidate_components
                .next()
                .is_some_and(|candidate_component| {
                    windows_components_equal(root_component, candidate_component)
                })
        })
    }
    #[cfg(not(windows))]
    {
        candidate.starts_with(root)
    }
}

#[cfg(windows)]
fn windows_components_equal(
    left: std::path::Component<'_>,
    right: std::path::Component<'_>,
) -> bool {
    match (left, right) {
        (std::path::Component::Prefix(left), std::path::Component::Prefix(right)) => {
            windows_os_str_eq_ignore_case(left.as_os_str(), right.as_os_str())
        }
        (std::path::Component::RootDir, std::path::Component::RootDir)
        | (std::path::Component::CurDir, std::path::Component::CurDir)
        | (std::path::Component::ParentDir, std::path::Component::ParentDir) => true,
        (std::path::Component::Normal(left), std::path::Component::Normal(right)) => {
            windows_os_str_eq_ignore_case(left, right)
        }
        _ => false,
    }
}

#[cfg(windows)]
fn windows_os_str_eq_ignore_case(left: &std::ffi::OsStr, right: &std::ffi::OsStr) -> bool {
    use std::os::windows::ffi::OsStrExt;

    let left = left.encode_wide().collect::<Vec<_>>();
    let right = right.encode_wide().collect::<Vec<_>>();
    if left.len() > i32::MAX as usize || right.len() > i32::MAX as usize {
        return false;
    }
    // CompareStringOrdinal uses Windows' ordinal case-insensitive rules for
    // path components, unlike Rust's lexical Path comparisons.
    unsafe {
        compare_string_ordinal(
            left.as_ptr(),
            left.len() as i32,
            right.as_ptr(),
            right.len() as i32,
            1,
        ) == 2
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    #[link_name = "CompareStringOrdinal"]
    fn compare_string_ordinal(
        left: *const u16,
        left_count: i32,
        right: *const u16,
        right_count: i32,
        ignore_case: i32,
    ) -> i32;
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
        termination_signal: previous.termination_signal,
        stdout: previous.stdout.clone(),
        stderr: previous.stderr.clone(),
        output_digest: previous.output_digest.clone(),
        timed_out: false,
        predecessor_attempt_id: Some(predecessor_attempt_id.into()),
    }
}

struct NodeExecutionContext<'a> {
    worktree: &'a Path,
    input: &'a CompositionInput,
    executable: &'a ResolvedExecutable,
    environment: &'a BTreeMap<String, String>,
    identity_digest: Option<Digest>,
    attempt: &'a CompositionAttempt,
    process_start_gate: Option<&'a ProcessStartGate>,
}

fn execute_node(
    command: &CompositionCommand,
    context: NodeExecutionContext<'_>,
) -> CompositionExecutionRecord {
    let NodeExecutionContext {
        worktree,
        input,
        executable,
        environment,
        identity_digest,
        attempt,
        process_start_gate,
    } = context;
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
    let observer = move |node_id: &str, process_group_id: u32, active: bool| {
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
    };
    let receipt = if let Some(process_start_gate) = process_start_gate {
        execute_bounded_with_process_observer_and_start_gate(
            vec![verification_command],
            1,
            observer,
            process_start_gate.clone(),
        )
    } else {
        execute_bounded_with_process_observer(vec![verification_command], 1, observer)
    };
    match receipt {
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
                termination_signal: execution.and_then(|record| record.termination_signal),
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
            termination_signal: None,
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
    let path = attempt_record_path(state_dir, attempt_id);
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
    #[cfg(target_os = "linux")]
    let process_group_identity = if active {
        Some(read_linux_process_group_leader_identity(
            Path::new("/proc"),
            process_group_id,
        )?)
    } else {
        None
    };
    #[cfg(not(target_os = "linux"))]
    let process_group_identity = None;
    if active {
        if attempt.active_process_group_id.is_some()
            || attempt.active_process_group_identity.is_some()
        {
            return Err("composition already records an active process group".into());
        }
        attempt.active_process_group_id = Some(process_group_id);
        attempt.active_process_group_identity = process_group_identity;
    } else {
        if attempt.active_process_group_id != Some(process_group_id) {
            return Err("composition active process group does not match".into());
        }
        attempt.active_process_group_id = None;
        attempt.active_process_group_identity = None;
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
    let cargo_home = environment.get("CARGO_HOME").map(PathBuf::from);
    let (cargo_configuration_root, cargo_configuration_paths) = if let Some(cargo_home) = cargo_home
    {
        let metadata = fs::symlink_metadata(&cargo_home).ok()?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return None;
        }
        (
            fs::canonicalize(&cargo_home).ok()?,
            ["config", "config.toml"],
        )
    } else {
        (home.clone(), [".cargo/config", ".cargo/config.toml"])
    };
    for path in cargo_configuration_paths {
        let bytes = read_optional_regular_file_beneath(&cargo_configuration_root, Path::new(path))?;
        cargo_configuration.push((path, bytes.as_deref().map(Digest::sha256_bytes)));
    }

    let workspace_channel = read_workspace_toolchain_channel(worktree)?;
    let rustup_home = environment
        .get("RUSTUP_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".rustup"));
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

    let rustup_home = fs::canonicalize(rustup_home).ok()?;
    let settings = read_regular_file_beneath(&rustup_home, Path::new("settings.toml"))?;
    let default_channel = parse_rustup_default_toolchain(&settings)?;
    let requested_channel = workspace_channel.unwrap_or(default_channel);
    if !is_safe_toolchain_channel(&requested_channel) {
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
    let inside_worktree = path_is_within(&canonical_worktree, &identity_path);
    let inside_trusted_root = trusted_roots
        .iter()
        .any(|directory| path_is_within(directory, &identity_path));
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
        if let Some(cargo_home) = std::env::var_os("CARGO_HOME") {
            let cargo_home = PathBuf::from(cargo_home);
            if cargo_home.is_absolute() {
                candidates.push(cargo_home.join("bin"));
            }
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
            candidates.push(
                PathBuf::from(&system_root)
                    .join("System32")
                    .join("WindowsPowerShell")
                    .join("v1.0"),
            );
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
    for key in ["CARGO_HOME", "RUSTUP_HOME"] {
        if let Some(directory) = std::env::var_os(key) {
            let directory = PathBuf::from(directory);
            if !directory.is_absolute() {
                return None;
            }
            let metadata = fs::symlink_metadata(&directory).ok()?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return None;
            }
            let directory = fs::canonicalize(directory).ok()?;
            environment.insert(key.into(), directory.to_string_lossy().into_owned());
        }
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
    let path = attempt_record_path(state_dir, &attempt.attempt_id);
    let temporary = state_dir.join(format!(
        ".{}.{}.tmp",
        portable_attempt_stem(&attempt.attempt_id),
        now_unix_nanos()
    ));
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
    fs::rename(&temporary, &path).map_err(|source| CompositionError::Io {
        path: path.clone(),
        source,
    })?;
    #[cfg(unix)]
    File::open(state_dir)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| CompositionError::Io {
            path: state_dir.to_path_buf(),
            source,
        })?;
    Ok(())
}

fn attempt_record_path(state_dir: &Path, attempt_id: &str) -> PathBuf {
    let portable = state_dir.join(format!("{}.json", portable_attempt_stem(attempt_id)));
    if portable.exists() {
        return portable;
    }
    #[cfg(unix)]
    {
        // Preserve and continue updating attempt records written by earlier
        // Unix Runtime versions, whose filenames used the logical ID verbatim.
        let legacy = state_dir.join(format!("{attempt_id}.json"));
        if legacy.exists() {
            return legacy;
        }
    }
    portable
}

fn attempt_record_filename_matches(path: &Path, attempt_id: &str) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if file_name == format!("{}.json", portable_attempt_stem(attempt_id)) {
        return true;
    }
    #[cfg(unix)]
    {
        file_name == format!("{attempt_id}.json")
    }
    #[cfg(not(unix))]
    {
        false
    }
}

fn portable_attempt_stem(attempt_id: &str) -> String {
    let mut stem = String::with_capacity("composition-".len() + attempt_id.len() * 2);
    stem.push_str("composition-");
    for byte in attempt_id.bytes() {
        stem.push_str(&format!("{byte:02x}"));
    }
    stem
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
    composition_parent_path(std::process::id(), now_unix_nanos())
}

fn composition_parent_path(owner_pid: u32, timestamp: u128) -> PathBuf {
    let sequence = NEXT_COMPOSITION_PARENT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "ai-cockpit-composition-{owner_pid}-{timestamp}-{sequence}"
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

#[cfg(test)]
mod composition_parent_tests {
    use super::composition_parent_path;
    use std::collections::BTreeSet;

    #[test]
    fn concurrent_composition_parents_remain_unique_at_the_same_clock_tick() {
        let paths = (0..32)
            .map(|_| composition_parent_path(1234, 1_800_000_000_000_000_000))
            .collect::<BTreeSet<_>>();

        assert_eq!(paths.len(), 32, "same-tick parents must not collide");
    }
}

#[cfg(all(test, target_os = "linux"))]
mod linux_process_group_reconciliation_tests {
    use super::{
        LinuxProcessGroupLiveness, LinuxProcessGroupMember, LinuxProcessGroupScanError,
        ProcessGroupLeaderIdentity, classify_linux_process_group_scans,
        current_attempt_file_digest, observe_linux_process_group_with_retries,
        parse_linux_process_stat, require_attempt_digest_unchanged, scan_linux_process_group_once,
    };
    use std::cell::Cell;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    static NEXT_PROC_ROOT: AtomicU64 = AtomicU64::new(0);

    struct FakeProcRoot(PathBuf);

    impl FakeProcRoot {
        fn new() -> Self {
            let sequence = NEXT_PROC_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "cockpit-fake-proc-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("fake proc root");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn add_process(&self, process: LinuxProcessGroupMember) {
            let process_dir = self.0.join(process.process_id.to_string());
            fs::create_dir_all(&process_dir).expect("fake process directory");
            self.write_stat(&process);
        }

        fn write_stat(&self, process: &LinuxProcessGroupMember) {
            let mut fields = vec![
                process.state.to_string(),
                "1".into(),
                process.process_group_id.to_string(),
                process.session_id.to_string(),
            ];
            while fields.len() < 19 {
                fields.push("0".into());
            }
            fields.push(process.start_time_ticks.to_string());
            let stat = format!(
                "{} (test command (with parens)) {}\n",
                process.process_id,
                fields.join(" ")
            );
            fs::write(
                self.0.join(process.process_id.to_string()).join("stat"),
                stat,
            )
            .expect("write fake process stat");
        }
    }

    impl Drop for FakeProcRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn member(pid: u32, state: char, start: u64, pgid: u32, sid: u32) -> LinuxProcessGroupMember {
        LinuxProcessGroupMember {
            process_id: pid,
            state,
            start_time_ticks: start,
            process_group_id: pgid,
            session_id: sid,
        }
    }

    fn leader_identity(pid: u32, start: u64, pgid: u32, sid: u32) -> ProcessGroupLeaderIdentity {
        ProcessGroupLeaderIdentity {
            leader_pid: pid,
            leader_start_time_ticks: start,
            process_group_id: pgid,
            session_id: sid,
        }
    }

    fn disappeared_stat(path: &str) -> LinuxProcessGroupScanError {
        LinuxProcessGroupScanError::StatEntryDisappeared {
            path: PathBuf::from(path),
            source: std::io::Error::from_raw_os_error(libc::ENOENT),
        }
    }

    fn observe_with_injected_scans<S, V, W, N>(
        process_group_id: u32,
        identity: Option<&ProcessGroupLeaderIdentity>,
        total_deadline: Instant,
        scan: S,
        validate_attempt: V,
        wait: W,
        now: N,
    ) -> Result<LinuxProcessGroupLiveness, String>
    where
        S: FnMut(Instant) -> Result<Vec<LinuxProcessGroupMember>, LinuxProcessGroupScanError>,
        V: FnMut() -> Result<(), String>,
        W: FnMut(Duration),
        N: FnMut() -> Instant,
    {
        observe_linux_process_group_with_retries(
            process_group_id,
            identity,
            total_deadline,
            scan,
            validate_attempt,
            wait,
            now,
        )
    }

    #[test]
    fn one_disappeared_stat_in_a_partial_pair_restarts_and_stable_active_group_succeeds() {
        let leader = member(4051, 'S', 51, 4051, 4051);
        let identity = leader_identity(4051, 51, 4051, 4051);
        let clock = Cell::new(Instant::now());
        let scan_count = Cell::new(0);
        let validation_count = Cell::new(0);
        let wait_count = Cell::new(0);
        let deadline = clock.get() + Duration::from_secs(3);

        let observed = observe_with_injected_scans(
            4051,
            Some(&identity),
            deadline,
            |scan_deadline| {
                assert!(scan_deadline <= deadline);
                let index = scan_count.get();
                scan_count.set(index + 1);
                match index {
                    0 => Ok(vec![leader.clone()]),
                    1 => Err(disappeared_stat("/proc/4999/stat")),
                    2 | 3 => Ok(vec![leader.clone()]),
                    _ => panic!("unexpected extra process scan"),
                }
            },
            || {
                validation_count.set(validation_count.get() + 1);
                Ok(())
            },
            |delay| {
                wait_count.set(wait_count.get() + 1);
                clock.set(clock.get() + delay);
            },
            || clock.get(),
        )
        .expect("a fresh stable pair should recover from one unrelated PID exit");

        assert_eq!(observed, LinuxProcessGroupLiveness::Active);
        assert_eq!(scan_count.get(), 4, "retry starts over from the first scan");
        assert_eq!(
            validation_count.get(),
            4,
            "each pair is bound before and after"
        );
        assert_eq!(
            wait_count.get(),
            2,
            "the abandoned partial pair is discarded"
        );
    }

    #[test]
    fn three_disappeared_stat_entries_leave_the_process_group_unknown() {
        let identity = leader_identity(4052, 52, 4052, 4052);
        let clock = Cell::new(Instant::now());
        let scan_count = Cell::new(0);
        let validation_count = Cell::new(0);
        let wait_count = Cell::new(0);
        let deadline = clock.get() + Duration::from_secs(3);

        let error = observe_with_injected_scans(
            4052,
            Some(&identity),
            deadline,
            |_| {
                scan_count.set(scan_count.get() + 1);
                Err(disappeared_stat("/proc/4998/stat"))
            },
            || {
                validation_count.set(validation_count.get() + 1);
                Ok(())
            },
            |_| wait_count.set(wait_count.get() + 1),
            || clock.get(),
        )
        .expect_err("a process group remains unknown after three incomplete attempts");

        assert!(error.contains("/proc/4998/stat"));
        assert_eq!(
            scan_count.get(),
            3,
            "the retry budget is exactly three attempts"
        );
        assert_eq!(
            validation_count.get(),
            6,
            "each incomplete pair is revalidated"
        );
        assert_eq!(
            wait_count.get(),
            0,
            "no stability wait follows an incomplete scan"
        );
    }

    #[test]
    fn permission_and_parse_errors_are_not_retried() {
        for message in [
            "proc scan could not read /proc/4997/stat: permission denied",
            "proc scan could not parse /proc/4996/stat: malformed stat",
        ] {
            let identity = leader_identity(4053, 53, 4053, 4053);
            let clock = Cell::new(Instant::now());
            let scan_count = Cell::new(0);
            let validation_count = Cell::new(0);
            let deadline = clock.get() + Duration::from_secs(3);
            let error = observe_with_injected_scans(
                4053,
                Some(&identity),
                deadline,
                |_| {
                    scan_count.set(scan_count.get() + 1);
                    Err(LinuxProcessGroupScanError::Other(message.into()))
                },
                || {
                    validation_count.set(validation_count.get() + 1);
                    Ok(())
                },
                |_| panic!("non-ENOENT scanner failures do not retry"),
                || clock.get(),
            )
            .expect_err("non-ENOENT proc scan errors remain unknown");

            assert!(error.contains(message));
            assert_eq!(scan_count.get(), 1);
            assert_eq!(validation_count.get(), 2);
        }
    }

    #[test]
    fn stable_active_and_zombie_only_groups_return_their_normal_liveness() {
        for (process, expected) in [
            (
                member(4054, 'R', 54, 4054, 4054),
                LinuxProcessGroupLiveness::Active,
            ),
            (
                member(4055, 'Z', 55, 4055, 4055),
                LinuxProcessGroupLiveness::Exited,
            ),
        ] {
            let identity = leader_identity(
                process.process_id,
                process.start_time_ticks,
                process.process_group_id,
                process.session_id,
            );
            let clock = Cell::new(Instant::now());
            let scan_count = Cell::new(0);
            let validation_count = Cell::new(0);
            let deadline = clock.get() + Duration::from_secs(3);
            let observed = observe_with_injected_scans(
                process.process_group_id,
                Some(&identity),
                deadline,
                |_| {
                    scan_count.set(scan_count.get() + 1);
                    Ok(vec![process.clone()])
                },
                || {
                    validation_count.set(validation_count.get() + 1);
                    Ok(())
                },
                |delay| clock.set(clock.get() + delay),
                || clock.get(),
            )
            .expect("stable group observation should complete");

            assert_eq!(observed, expected);
            assert_eq!(scan_count.get(), 2, "stable results do not retry");
            assert_eq!(validation_count.get(), 2);
        }
    }

    #[test]
    fn member_or_leader_identity_inconsistency_is_unknown_without_retry() {
        let live = member(4056, 'S', 56, 4056, 4056);
        let clock = Cell::new(Instant::now());
        let scan_count = Cell::new(0);
        let validation_count = Cell::new(0);
        let deadline = clock.get() + Duration::from_secs(3);
        let error = observe_with_injected_scans(
            4056,
            Some(&leader_identity(4056, 999, 4056, 4056)),
            deadline,
            |_| {
                scan_count.set(scan_count.get() + 1);
                Ok(vec![live.clone()])
            },
            || {
                validation_count.set(validation_count.get() + 1);
                Ok(())
            },
            |_| {},
            || clock.get(),
        )
        .expect_err("a changed leader identity must remain unknown");
        assert!(error.contains("identity") || error.contains("Unknown"));
        assert_eq!(scan_count.get(), 2, "identity mismatch does not retry");
        assert_eq!(validation_count.get(), 2);

        let clock = Cell::new(Instant::now());
        let scan_count = Cell::new(0);
        let deadline = clock.get() + Duration::from_secs(3);
        let error = observe_with_injected_scans(
            4056,
            Some(&leader_identity(4056, 56, 4056, 4056)),
            deadline,
            |_| {
                let index = scan_count.get();
                scan_count.set(index + 1);
                if index == 0 {
                    Ok(vec![live.clone()])
                } else {
                    Ok(Vec::new())
                }
            },
            || Ok(()),
            |_| {},
            || clock.get(),
        )
        .expect_err("membership changes between scans must remain unknown");
        assert!(error.contains("member") || error.contains("Unknown"));
        assert_eq!(scan_count.get(), 2, "membership mismatch does not retry");
    }

    #[test]
    fn attempt_digest_change_after_a_pair_rejects_the_observation() {
        let live = member(4057, 'S', 57, 4057, 4057);
        let identity = leader_identity(4057, 57, 4057, 4057);
        let clock = Cell::new(Instant::now());
        let scan_count = Cell::new(0);
        let validation_count = Cell::new(0);
        let deadline = clock.get() + Duration::from_secs(3);
        let error = observe_with_injected_scans(
            4057,
            Some(&identity),
            deadline,
            |_| {
                scan_count.set(scan_count.get() + 1);
                Ok(vec![live.clone()])
            },
            || {
                validation_count.set(validation_count.get() + 1);
                if validation_count.get() == 2 {
                    Err("latest composition attempt digest changed".into())
                } else {
                    Ok(())
                }
            },
            |delay| clock.set(clock.get() + delay),
            || clock.get(),
        )
        .expect_err("a changed attempt digest rejects even a stable process scan");

        assert!(error.contains("digest changed"));
        assert_eq!(scan_count.get(), 2);
        assert_eq!(validation_count.get(), 2);
    }

    #[test]
    fn total_deadline_includes_the_stability_delay() {
        let live = member(4058, 'S', 58, 4058, 4058);
        let identity = leader_identity(4058, 58, 4058, 4058);
        let start = Instant::now();
        let clock = Cell::new(start);
        let scan_count = Cell::new(0);
        let validation_count = Cell::new(0);
        let deadline = start + Duration::from_secs(1);
        let error = observe_with_injected_scans(
            4058,
            Some(&identity),
            deadline,
            |_| {
                scan_count.set(scan_count.get() + 1);
                Ok(vec![live.clone()])
            },
            || {
                validation_count.set(validation_count.get() + 1);
                Ok(())
            },
            |delay| clock.set(clock.get() + delay + Duration::from_secs(1)),
            || clock.get(),
        )
        .expect_err("the shared deadline also bounds the stability wait");

        assert!(error.contains("deadline"));
        assert_eq!(scan_count.get(), 1, "no second scan starts after deadline");
        assert_eq!(
            validation_count.get(),
            2,
            "the attempt is rebound after timeout"
        );
    }

    #[test]
    fn total_deadline_includes_scan_execution() {
        let live = member(4059, 'S', 59, 4059, 4059);
        let identity = leader_identity(4059, 59, 4059, 4059);
        let start = Instant::now();
        let clock = Cell::new(start);
        let scan_count = Cell::new(0);
        let validation_count = Cell::new(0);
        let deadline = start + Duration::from_secs(1);
        let error = observe_with_injected_scans(
            4059,
            Some(&identity),
            deadline,
            |_| {
                scan_count.set(scan_count.get() + 1);
                clock.set(clock.get() + Duration::from_secs(2));
                Ok(vec![live.clone()])
            },
            || {
                validation_count.set(validation_count.get() + 1);
                Ok(())
            },
            |_| {},
            || clock.get(),
        )
        .expect_err("the overall deadline also bounds scan execution");

        assert!(error.contains("deadline"));
        assert_eq!(scan_count.get(), 1, "no second scan starts after deadline");
        assert_eq!(
            validation_count.get(),
            2,
            "the attempt is rebound after timeout"
        );
    }

    #[test]
    fn stable_zombie_only_group_exits_but_a_mixed_group_is_active() {
        let zombie_leader = member(4101, 'Z', 91, 4101, 4101);
        let identity = leader_identity(4101, 91, 4101, 4101);
        assert_eq!(
            classify_linux_process_group_scans(
                4101,
                Some(&identity),
                std::slice::from_ref(&zombie_leader),
                std::slice::from_ref(&zombie_leader),
            ),
            LinuxProcessGroupLiveness::Exited
        );

        let live_child = member(4102, 'S', 92, 4101, 4101);
        let mixed = vec![zombie_leader.clone(), live_child];
        assert_eq!(
            classify_linux_process_group_scans(4101, Some(&identity), &mixed, &mixed),
            LinuxProcessGroupLiveness::Active
        );
    }

    #[test]
    fn live_group_is_active_then_empty_reaped_group_is_exited() {
        let identity = leader_identity(4201, 101, 4201, 4201);
        let live = member(4201, 'R', 101, 4201, 4201);
        assert_eq!(
            classify_linux_process_group_scans(
                4201,
                Some(&identity),
                std::slice::from_ref(&live),
                std::slice::from_ref(&live),
            ),
            LinuxProcessGroupLiveness::Active
        );
        let changed_session = member(4201, 'R', 101, 4201, 4202);
        assert_eq!(
            classify_linux_process_group_scans(
                4201,
                Some(&identity),
                std::slice::from_ref(&live),
                std::slice::from_ref(&changed_session),
            ),
            LinuxProcessGroupLiveness::Unknown,
            "a session change between scans is a PID/process-group reuse race"
        );
        assert_eq!(
            classify_linux_process_group_scans(4201, Some(&identity), &[], &[]),
            LinuxProcessGroupLiveness::Exited
        );
        assert_eq!(
            classify_linux_process_group_scans(4201, Some(&identity), &[live], &[]),
            LinuxProcessGroupLiveness::Unknown,
            "membership changing during the paired scan is a race"
        );
    }

    #[test]
    fn missing_reused_or_inconsistent_leader_identity_is_unknown_while_members_remain() {
        let zombie = member(4301, 'Z', 111, 4301, 4301);
        assert_eq!(
            classify_linux_process_group_scans(
                4301,
                None,
                std::slice::from_ref(&zombie),
                std::slice::from_ref(&zombie),
            ),
            LinuxProcessGroupLiveness::Unknown
        );
        assert_eq!(
            classify_linux_process_group_scans(
                4301,
                Some(&leader_identity(4301, 112, 4301, 4301)),
                std::slice::from_ref(&zombie),
                std::slice::from_ref(&zombie),
            ),
            LinuxProcessGroupLiveness::Unknown,
            "a reused leader PID has a different starttime"
        );
        assert_eq!(
            classify_linux_process_group_scans(
                4301,
                Some(&leader_identity(4301, 111, 4301, 7)),
                std::slice::from_ref(&zombie),
                std::slice::from_ref(&zombie),
            ),
            LinuxProcessGroupLiveness::Unknown,
            "the session identity must also match"
        );
    }

    #[test]
    fn proc_stat_parser_handles_parentheses_and_rejects_malformed_identity() {
        let mut fields = vec!["Z".to_string(), "1".into(), "4401".into(), "4401".into()];
        while fields.len() < 19 {
            fields.push("0".into());
        }
        fields.push("121".into());
        let stat = format!("4401 (command (worker)) {}\n", fields.join(" "));
        assert_eq!(
            parse_linux_process_stat(4401, &stat).expect("well-formed stat"),
            member(4401, 'Z', 121, 4401, 4401)
        );
        assert!(parse_linux_process_stat(4401, "4401 (broken) Z 1 2").is_err());
        assert!(parse_linux_process_stat(4402, &stat).is_err());
    }

    #[test]
    fn proc_scanner_fails_closed_on_unreadable_racing_and_over_budget_entries() {
        let malformed = FakeProcRoot::new();
        let malformed_stat = malformed.path().join("4501");
        fs::create_dir_all(&malformed_stat).expect("numeric proc entry");
        fs::create_dir_all(malformed_stat.join("stat")).expect("unreadable stat directory");
        assert!(matches!(
            scan_linux_process_group_once(
                malformed.path(),
                4501,
                10,
                Instant::now() + Duration::from_secs(1)
            ),
            Err(LinuxProcessGroupScanError::Other(message))
                if message.contains("proc scan could not read")
        ));

        let missing_root = malformed.path().join("missing-proc-root");
        assert!(matches!(
            scan_linux_process_group_once(
                &missing_root,
                4501,
                10,
                Instant::now() + Duration::from_secs(1)
            ),
            Err(LinuxProcessGroupScanError::Other(message))
                if message.contains("cannot enumerate proc root")
        ));

        let racing = FakeProcRoot::new();
        let racing_entry = racing.path().join("4502");
        fs::create_dir_all(&racing_entry).expect("racing proc entry");
        assert!(matches!(
            scan_linux_process_group_once(
                racing.path(),
                4502,
                10,
                Instant::now() + Duration::from_secs(1)
            ),
            Err(LinuxProcessGroupScanError::StatEntryDisappeared { path, source })
                if path == racing_entry.join("stat")
                    && source.kind() == std::io::ErrorKind::NotFound
        ));

        let over_budget = FakeProcRoot::new();
        over_budget.add_process(member(4503, 'S', 1, 4503, 4503));
        over_budget.add_process(member(4504, 'S', 2, 4504, 4504));
        assert!(matches!(
            scan_linux_process_group_once(
                over_budget.path(),
                4503,
                1,
                Instant::now() + Duration::from_secs(1)
            ),
            Err(LinuxProcessGroupScanError::Other(message))
                if message.contains("entry budget")
        ));

        let malformed_stat = FakeProcRoot::new();
        malformed_stat.add_process(member(4505, 'S', 3, 4505, 4505));
        fs::write(
            malformed_stat.path().join("4505/stat"),
            "4505 (malformed stat) Z 1 2\n",
        )
        .expect("write malformed stat contents");
        assert!(matches!(
            scan_linux_process_group_once(
                malformed_stat.path(),
                4505,
                10,
                Instant::now() + Duration::from_secs(1)
            ),
            Err(LinuxProcessGroupScanError::Other(message))
                if message.contains("proc scan could not parse")
        ));
    }

    #[test]
    fn proc_scans_detect_pid_reuse_between_snapshots() {
        let proc_root = FakeProcRoot::new();
        let original = member(4601, 'Z', 131, 4601, 4601);
        proc_root.add_process(original.clone());
        let first = scan_linux_process_group_once(
            proc_root.path(),
            4601,
            10,
            Instant::now() + Duration::from_secs(1),
        )
        .expect("first complete scan");
        proc_root.write_stat(&member(4601, 'Z', 132, 4601, 4601));
        let second = scan_linux_process_group_once(
            proc_root.path(),
            4601,
            10,
            Instant::now() + Duration::from_secs(1),
        )
        .expect("second complete scan");
        assert_eq!(
            classify_linux_process_group_scans(
                4601,
                Some(&leader_identity(4601, 131, 4601, 4601)),
                &first,
                &second,
            ),
            LinuxProcessGroupLiveness::Unknown
        );
    }

    #[test]
    fn cleanup_guard_rejects_a_changed_latest_attempt_digest() {
        let state = FakeProcRoot::new();
        let attempt_id = "digest-stability";
        let path = state.path().join(format!("{attempt_id}.json"));
        fs::write(&path, br#"{"activeProcessGroupId":77}"#).expect("initial attempt");
        let observed = current_attempt_file_digest(state.path(), attempt_id)
            .expect("read initial attempt digest");
        require_attempt_digest_unchanged(state.path(), attempt_id, &observed)
            .expect("unchanged attempt digest");

        fs::write(&path, br#"{"activeProcessGroupId":78}"#).expect("changed attempt");
        assert!(require_attempt_digest_unchanged(state.path(), attempt_id, &observed).is_err());
    }
}

#[cfg(all(test, target_os = "linux"))]
mod external_observer_deferred_tests {
    use super::*;
    use cockpit_protocol::{COLLABORATION_CAPABILITY, RuntimeCapabilityBinding};
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    static NEXT_CASE: AtomicU64 = AtomicU64::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(label: &str) -> Self {
            let sequence = NEXT_CASE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "cockpit-a12-{label}-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("create composition case directory");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct OwnedWorktrees {
        repository: PathBuf,
        paths: Vec<PathBuf>,
    }

    struct InaccessibleObserverPeer(libc::pid_t);

    impl Drop for InaccessibleObserverPeer {
        fn drop(&mut self) {
            // SAFETY: this test owns the child PID returned by fork.
            unsafe {
                libc::kill(self.0, libc::SIGKILL);
                libc::waitpid(self.0, std::ptr::null_mut(), 0);
            }
        }
    }

    impl OwnedWorktrees {
        fn new(repository: &Path) -> Self {
            Self {
                repository: repository.to_path_buf(),
                paths: Vec::new(),
            }
        }

        fn retain_for_assertions(&mut self, path: &str) {
            if !path.is_empty() {
                self.paths.push(PathBuf::from(path));
            }
        }
    }

    impl Drop for OwnedWorktrees {
        fn drop(&mut self) {
            for path in &self.paths {
                let _ = Command::new("git")
                    .args(["-C"])
                    .arg(&self.repository)
                    .args(["worktree", "remove", "--force"])
                    .arg(path)
                    .output();
                if let Some(parent) = path.parent() {
                    let _ = fs::remove_dir_all(parent);
                }
            }
        }
    }

    fn git(root: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .expect("run fixture git command");
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("git fixture output is UTF-8")
            .trim()
            .into()
    }

    fn composition_case() -> (TestDir, TestDir, CompositionInput) {
        let repository = TestDir::new("repository");
        git(repository.path(), &["init", "-q"]);
        git(
            repository.path(),
            &["config", "user.email", "test@example.invalid"],
        );
        git(repository.path(), &["config", "user.name", "Test"]);
        fs::write(repository.path().join("README.md"), "fixture\n").expect("write fixture");
        git(repository.path(), &["add", "."]);
        git(repository.path(), &["commit", "-qm", "fixture"]);
        git(repository.path(), &["branch", "-M", "main"]);
        let head = git(repository.path(), &["rev-parse", "HEAD"]);
        let state = TestDir::new("state");
        let command = CompositionCommand {
            node_id: "safe-noop".into(),
            program: "true".into(),
            args: Vec::new(),
            depends_on: Vec::new(),
            environment: Default::default(),
            input_paths: Vec::new(),
            covered_scenarios: Vec::new(),
            covered_constraints: Vec::new(),
        };
        let input = CompositionInput {
            repository_root: repository.path().to_path_buf(),
            state_dir: state.path().to_path_buf(),
            binding: CompositionBinding {
                schema_version: 1,
                repository_id: Digest::sha256_bytes(b"a12 repository"),
                binding_id: "a12-observer-unknown".into(),
                target_branch: "main".into(),
                target_sha: head.clone(),
                participant_work_items: vec!["WI-A12-PROVIDER".into(), "WI-A12-CONSUMER".into()],
                participant_heads: vec![head.clone(), head],
                contract_digests: vec![
                    Digest::sha256_bytes(b"provider Contract"),
                    Digest::sha256_bytes(b"consumer Contract"),
                ],
                verifier: RuntimeCapabilityBinding {
                    schema_version: 1,
                    runtime_version: "0.2.113".into(),
                    runtime_digest: Digest::sha256_bytes(b"a12 Runtime"),
                    capability: COLLABORATION_CAPABILITY.into(),
                },
            },
            identity: CompositionIdentity::default(),
            reusable_node_ids: vec!["safe-noop".into()],
            commands: vec![command],
            preconditions: vec![CompositionPrecondition::satisfied("bound")],
            timeout_seconds: 30,
        };
        (repository, state, input)
    }

    fn real_eacces_is_testable_without_ptrace_capability() -> bool {
        if unsafe { libc::geteuid() } == 0 {
            eprintln!("skipping real EACCES observer case: root may bypass procfs ptrace checks");
            false
        } else {
            true
        }
    }

    fn require_controlled_eacces_observation(
        child_pid: libc::pid_t,
        observation: Result<Option<u32>, String>,
    ) -> Result<Option<u32>, String> {
        match observation {
            Err(error) if error.contains(&format!("pid={child_pid}")) => Err(error),
            Err(error) => panic!(
                "observer encountered another process instead of controlled child pid={child_pid}: {error}"
            ),
            Ok(Some(process_id)) => {
                panic!("expected unknown EACCES observation, found readable process {process_id}")
            }
            Ok(None) => panic!("expected EACCES observation, found no process"),
        }
    }

    fn observe_real_eacces_worktree_process(worktree: &Path) -> Result<Option<u32>, String> {
        let worktree = fs::canonicalize(worktree).map_err(|error| error.to_string())?;
        let worktree_c =
            CString::new(worktree.as_os_str().as_bytes()).map_err(|error| error.to_string())?;
        let mut ready_pipe = [0; 2];
        // SAFETY: ready_pipe points to two writable file descriptors.
        if unsafe { libc::pipe(ready_pipe.as_mut_ptr()) } != 0 {
            return Err(format!(
                "cannot create EACCES child readiness pipe: {}",
                std::io::Error::last_os_error()
            ));
        }
        // SAFETY: the child uses only async-signal-safe libc calls after fork.
        let child_pid = unsafe { libc::fork() };
        if child_pid < 0 {
            // SAFETY: both descriptors were created by this test process.
            unsafe {
                libc::close(ready_pipe[0]);
                libc::close(ready_pipe[1]);
            }
            return Err(format!(
                "cannot fork EACCES child: {}",
                std::io::Error::last_os_error()
            ));
        }
        if child_pid == 0 {
            // SAFETY: the child owns the pipe write end and inherited path bytes.
            unsafe {
                libc::close(ready_pipe[0]);
                if libc::chdir(worktree_c.as_ptr()) != 0 {
                    libc::_exit(111);
                }
                if libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) != 0 {
                    libc::_exit(112);
                }
                let ready = b'R';
                if libc::write(ready_pipe[1], (&ready as *const u8).cast(), 1) != 1 {
                    libc::_exit(113);
                }
                libc::close(ready_pipe[1]);
                loop {
                    libc::pause();
                }
            }
        }
        let peer = InaccessibleObserverPeer(child_pid);
        // SAFETY: the parent closes the unused write end and waits until the
        // child has entered the worktree and disabled dumpability.
        unsafe {
            libc::close(ready_pipe[1]);
        }
        let mut ready = 0_u8;
        // SAFETY: ready points to one writable byte and the parent owns the
        // pipe read end.
        let read_result = unsafe { libc::read(ready_pipe[0], (&mut ready as *mut u8).cast(), 1) };
        // SAFETY: the parent owns the pipe read descriptor.
        unsafe {
            libc::close(ready_pipe[0]);
        }
        if read_result != 1 || ready != b'R' {
            drop(peer);
            return Err("EACCES child failed to report its controlled state".into());
        }
        let process_stat = fs::read_to_string(format!("/proc/{child_pid}/stat"))
            .map_err(|error| format!("cannot read controlled process stat: {error}"))?;
        parse_linux_process_stat(child_pid as u32, &process_stat)
            .map_err(|error| format!("cannot parse controlled process identity: {error}"))?;
        let denied_cwd = match fs::read_link(format!("/proc/{child_pid}/cwd")) {
            Ok(path) => {
                drop(peer);
                return Err(format!(
                    "controlled child cwd was unexpectedly readable: {}",
                    path.display()
                ));
            }
            Err(error) => error,
        };
        if denied_cwd.raw_os_error() != Some(libc::EACCES) {
            drop(peer);
            return Err(format!(
                "controlled child did not produce EACCES for cwd: {denied_cwd}"
            ));
        }
        let observation = verifier_process_using_worktree(&worktree);
        drop(peer);
        require_controlled_eacces_observation(child_pid, observation)
    }

    fn run_supervisor_case(input: &CompositionInput, unknown_path: &str) -> CompositionAttempt {
        run_supervisor_case_with_generation(input, unknown_path, 1)
    }

    fn run_supervisor_case_with_generation(
        input: &CompositionInput,
        unknown_path: &str,
        generation: u64,
    ) -> CompositionAttempt {
        run_supervisor_case_result(input, unknown_path, generation)
            .expect("supervisor fixture attempt")
    }

    fn run_supervisor_case_result(
        input: &CompositionInput,
        unknown_path: &str,
        generation: u64,
    ) -> Result<CompositionAttempt, String> {
        let io = TestDir::new("supervisor-io");
        let input_path = io.path().join("input.json");
        let output_path = io.path().join("output.json");
        fs::write(
            &input_path,
            serde_json::to_vec(input).expect("serialize input"),
        )
        .expect("write input");
        let output = Command::new(std::env::current_exe().expect("unit test binary"))
            .args(["external_observer_test_child_entry", "--nocapture"])
            .env("AI_COCKPIT_A12_CHILD_INPUT", &input_path)
            .env("AI_COCKPIT_A12_CHILD_OUTPUT", &output_path)
            .env("AI_COCKPIT_A12_UNKNOWN_PATH", unknown_path)
            .env("AI_COCKPIT_A12_GENERATION", generation.to_string())
            .output()
            .expect("run isolated composition supervisor fixture");
        assert!(
            output.status.success(),
            "supervisor fixture: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let result: Result<CompositionAttempt, String> =
            serde_json::from_slice(&fs::read(&output_path).expect("supervisor fixture output"))
                .expect("decode supervisor fixture result");
        result
    }

    #[derive(Clone, Copy)]
    enum DeniedReuseFinalObservation {
        Clean,
        ProcfsCwdEacces,
        Active(u32),
    }

    fn injected_procfs_cwd_eacces(process_id: u32) -> String {
        let identity = format!(
            "observed(pid={process_id},starttime_ticks=7,pgid={process_id},sid={process_id},state=S)"
        );
        format!(
            "cannot inspect process pid={process_id} filter_uid=Some({}) phase=procfs.cwd error_kind=PermissionDenied errno={} identity_state=observed identity_before={identity} identity_after={identity} message=Permission denied (os error {}) working directory",
            unsafe { libc::geteuid() },
            libc::EACCES,
            libc::EACCES
        )
    }

    fn run_denied_reuse_with_final_observation(
        input: CompositionInput,
        previous_worktree: PathBuf,
        final_observation: DeniedReuseFinalObservation,
    ) -> CompositionAttempt {
        const DENIAL: &str =
            "composition_reuse_not_admitted:safe-noop:coordination_safely_paused:late-request";

        let admission_checked = Arc::new(AtomicBool::new(false));
        let admission_checked_in_gate = Arc::clone(&admission_checked);
        let admission_check: ProcessAdmissionCheck = Arc::new(move |node_id, _accept| {
            assert_eq!(node_id, "safe-noop");
            admission_checked_in_gate.store(true, Ordering::SeqCst);
            Err("coordination_safely_paused:late-request".into())
        });
        let start_gate_called = Arc::new(AtomicBool::new(false));
        let start_gate_called_in_gate = Arc::clone(&start_gate_called);
        let process_start_gate: ProcessStartGate = Arc::new(move |_, _spawn| {
            start_gate_called_in_gate.store(true, Ordering::SeqCst);
            Err("process start gate must not run for a denied reuse".into())
        });
        let state_dir = input.state_dir.clone();
        let observe = move |worktree: &Path| {
            if worktree == previous_worktree {
                return Ok(None);
            }

            assert!(worktree.is_dir(), "final observer sees the live worktree");
            let persisted = fs::read_dir(&state_dir)
                .expect("enumerate attempt state before final observation")
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    let path = entry.path();
                    if !path
                        .extension()
                        .is_some_and(|extension| extension == "json")
                    {
                        return None;
                    }
                    let bytes = fs::read(path).ok()?;
                    let attempt: CompositionAttempt = serde_json::from_slice(&bytes).ok()?;
                    (Path::new(&attempt.isolated_worktree) == worktree).then_some(attempt)
                })
                .next()
                .expect("durable attempt for final-observed worktree");
            assert_eq!(persisted.failure.as_deref(), Some(DENIAL));
            assert!(!persisted.passed);
            assert_eq!(persisted.processes_spawned, 0);
            assert!(persisted.execution_records.is_empty());
            assert!(persisted.supervisor_receipt.is_none());

            match final_observation {
                DeniedReuseFinalObservation::Clean => Ok(None),
                DeniedReuseFinalObservation::ProcfsCwdEacces => {
                    Err(injected_procfs_cwd_eacces(4242))
                }
                DeniedReuseFinalObservation::Active(process_id) => Ok(Some(process_id)),
            }
        };
        let attempt = run_composition_inner_with_observer(
            input,
            Some(admission_check),
            Some(process_start_gate),
            None,
            None,
            CompositionObservationHooks {
                observe_worktree_process: &observe,
                reap_owned_descendants: &|| {
                    panic!("an attempt without a supervisor receipt must not reap descendants")
                },
                final_observation: FinalWorktreeObservation::Real,
            },
        )
        .expect("denied reuse returns a durable fail-closed attempt");

        assert!(admission_checked.load(Ordering::SeqCst));
        assert!(!start_gate_called.load(Ordering::SeqCst));
        assert!(!attempt.passed);
        assert_ne!(
            attempt.execution_outcome,
            CompositionExecutionOutcome::Passed
        );
        assert_eq!(attempt.processes_spawned, 0);
        assert!(attempt.execution_records.is_empty());
        attempt
    }

    fn save_attempt_json(state: &Path, attempt_id: &str, name: &str) -> serde_json::Value {
        let path = attempt_record_path(state, attempt_id);
        let bytes = fs::read(path).expect("durable attempt bytes");
        if let Ok(directory) = std::env::var("AI_COCKPIT_A12_EVIDENCE_DIR") {
            fs::create_dir_all(&directory).expect("create A12 diagnostic directory");
            fs::write(Path::new(&directory).join(name), &bytes)
                .expect("save exact A12 diagnostic attempt bytes");
        }
        serde_json::from_slice(&bytes).expect("durable attempt JSON")
    }

    #[test]
    fn external_observer_test_child_entry() {
        let (Ok(input_path), Ok(output_path), Ok(unknown_path)) = (
            std::env::var("AI_COCKPIT_A12_CHILD_INPUT"),
            std::env::var("AI_COCKPIT_A12_CHILD_OUTPUT"),
            std::env::var("AI_COCKPIT_A12_UNKNOWN_PATH"),
        ) else {
            return;
        };
        let input: CompositionInput =
            serde_json::from_slice(&fs::read(input_path).expect("child input"))
                .expect("child input JSON");
        let backend = initialize_composition_supervisor_backend().expect("Linux subreaper");
        assert_eq!(backend, CompositionSupervisorBackend::LinuxSubreaper);
        let process = current_composition_process_identity().expect("supervisor identity");
        let run_nonce = new_composition_run_nonce();
        let attempt_id = new_supervised_composition_attempt_id(&input, &run_nonce);
        let generation = std::env::var("AI_COCKPIT_A12_GENERATION")
            .expect("child generation")
            .parse()
            .expect("numeric child generation");
        let receipt = CompositionSupervisorReceipt {
            schema_version: 1,
            backend,
            attempt_id: attempt_id.clone(),
            run_nonce,
            generation,
            owner: process.clone(),
            supervisor: process,
            linux_boot_id: composition_linux_boot_id().expect("Linux boot identity"),
            environment_digest: Digest::sha256_bytes(b"a12 supervisor environment"),
            runtime_version: input.binding.verifier.runtime_version.clone(),
            runtime_digest: input.binding.verifier.runtime_digest.clone(),
            repository_id: input.binding.repository_id.clone(),
            target_sha: input.binding.target_sha.clone(),
            snapshot_digest: composition_target_snapshot_digest(&input)
                .expect("bound target snapshot"),
            command_plan_digest: composition_commands_digest(&input.commands),
            input_environment_digest: composition_input_environment_digest(&input),
            execution_records_digest: execution_records_digest(&[]),
            descendants_reaped_to_echild: false,
        };
        let check: ProcessAdmissionCheck = Arc::new(|_, accept| accept());
        let spawn_marker = input.state_dir.join("a12-verifier-spawns");
        let gate: ProcessStartGate = if unknown_path.starts_with("deny:") {
            Arc::new(|_, _| Err("injected current admission denial".into()))
        } else {
            Arc::new(move |_, spawn| {
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&spawn_marker)
                    .and_then(|mut marker| marker.write_all(b"x"))
                    .map_err(|error| format!("cannot mark A12 verifier spawn: {error}"))?;
                spawn()
            })
        };
        let observe = |path: &Path| {
            if unknown_path == "real-eacces" {
                observe_real_eacces_worktree_process(path)
            } else if let Some(active_path) = unknown_path.strip_prefix("active:")
                && path.to_string_lossy() == active_path
            {
                Ok(Some(std::process::id()))
            } else if unknown_path == "*"
                || path.to_string_lossy() == unknown_path
                || unknown_path
                    .strip_prefix("deny:")
                    .is_some_and(|denied_path| path.to_string_lossy() == denied_path)
            {
                Err("injected external /proc cwd observation error".into())
            } else {
                Ok(None)
            }
        };
        let result = run_composition_inner_with_observer(
            input,
            Some(check),
            Some(gate),
            Some(receipt),
            Some(attempt_id),
            CompositionObservationHooks {
                observe_worktree_process: &observe,
                reap_owned_descendants: &|| {
                    if unknown_path == "owned-proof-unknown" {
                        Err("injected owned descendant reap uncertainty".into())
                    } else {
                        reap_composition_supervisor_descendants()
                    }
                },
                final_observation: FinalWorktreeObservation::Real,
            },
        )
        .map_err(|error| error.to_string());
        fs::write(
            output_path,
            serde_json::to_vec(&result).expect("serialize child result"),
        )
        .expect("write child result");
    }

    #[test]
    fn real_eacces_fixture_rejects_a_diagnostic_for_another_pid() {
        let mismatch = std::panic::catch_unwind(|| {
            require_controlled_eacces_observation(
                4242,
                Err("cannot inspect process pid=4343 phase=procfs.cwd errno=13".into()),
            )
        });

        assert!(
            mismatch.is_err(),
            "a sibling process diagnostic must fail the fixture, not become expected Unknown"
        );
    }

    #[test]
    fn admission_change_blocks_reuse_with_controlled_clean_unknown_and_active_observations() {
        for observation in [
            DeniedReuseFinalObservation::Clean,
            DeniedReuseFinalObservation::ProcfsCwdEacces,
            DeniedReuseFinalObservation::Active(4242),
        ] {
            let (repository, _state, input) = composition_case();
            let first = run_supervisor_case(&input, "");
            assert!(first.passed);
            assert!(is_reusable_terminal_attempt(&first));
            assert_eq!(
                first.cleanup_disposition,
                CompositionCleanupDisposition::Cleaned
            );
            assert_eq!(first.processes_spawned, 1);
            assert_eq!(first.execution_records.len(), 1);
            let previous_worktree = PathBuf::from(&first.isolated_worktree);
            assert!(!previous_worktree.exists());

            let mut worktrees = OwnedWorktrees::new(repository.path());
            worktrees.retain_for_assertions(&first.isolated_worktree);
            let blocked =
                run_denied_reuse_with_final_observation(input, previous_worktree, observation);
            worktrees.retain_for_assertions(&blocked.isolated_worktree);
            assert!(blocked.supervisor_receipt.is_none());

            let worktree = Path::new(&blocked.isolated_worktree);
            match observation {
                DeniedReuseFinalObservation::Clean => {
                    assert_eq!(
                        blocked.failure.as_deref(),
                        Some(
                            "composition_reuse_not_admitted:safe-noop:coordination_safely_paused:late-request"
                        )
                    );
                    assert_eq!(
                        blocked.cleanup_disposition,
                        CompositionCleanupDisposition::Cleaned
                    );
                    assert_eq!(
                        blocked.execution_outcome,
                        CompositionExecutionOutcome::Failed
                    );
                    assert!(blocked.execution_evidence_complete);
                    assert!(
                        blocked
                            .cleanup
                            .as_ref()
                            .is_some_and(|cleanup| cleanup.removed && cleanup.error.is_none())
                    );
                    assert!(!worktree.exists());
                    assert!(
                        !worktree_is_registered(repository.path(), worktree)
                            .expect("verify clean worktree registration")
                    );
                }
                DeniedReuseFinalObservation::ProcfsCwdEacces => {
                    assert_eq!(
                        blocked.failure.as_deref(),
                        Some(
                            format!(
                                "verifier_process_state_unknown:{}",
                                injected_procfs_cwd_eacces(4242)
                            )
                            .as_str()
                        )
                    );
                    assert_eq!(
                        blocked.cleanup_disposition,
                        CompositionCleanupDisposition::Retained
                    );
                    assert_eq!(
                        blocked.execution_outcome,
                        CompositionExecutionOutcome::Unknown
                    );
                    assert!(!blocked.execution_evidence_complete);
                    assert!(blocked.owned_tree_termination_unknown);
                    assert!(blocked.cleanup.is_none());
                    assert!(worktree.is_dir());
                    assert!(
                        worktree_is_registered(repository.path(), worktree)
                            .expect("verify unknown worktree registration")
                    );
                }
                DeniedReuseFinalObservation::Active(process_id) => {
                    assert_eq!(
                        blocked.failure.as_deref(),
                        Some(format!("verifier_descendant_active:{process_id}").as_str())
                    );
                    assert_eq!(
                        blocked.cleanup_disposition,
                        CompositionCleanupDisposition::Retained
                    );
                    assert_eq!(
                        blocked.execution_outcome,
                        CompositionExecutionOutcome::Failed
                    );
                    assert!(blocked.execution_evidence_complete);
                    assert!(!blocked.owned_tree_termination_unknown);
                    assert!(blocked.cleanup.is_none());
                    assert!(worktree.is_dir());
                    assert!(
                        worktree_is_registered(repository.path(), worktree)
                            .expect("verify active worktree registration")
                    );
                }
            }
        }
    }

    #[test]
    fn complete_owned_execution_survives_external_unknown_and_retries_fresh_noop() {
        let _guard = LINUX_PROCESS_OBSERVATION_TEST_LOCK
            .lock()
            .expect("real EACCES observer tests are serialized");
        if !real_eacces_is_testable_without_ptrace_capability() {
            return;
        }
        let (repository, state, input) = composition_case();
        let mut worktrees = OwnedWorktrees::new(repository.path());
        let first = run_supervisor_case(&input, "real-eacces");
        worktrees.retain_for_assertions(&first.isolated_worktree);
        let first_raw = save_attempt_json(state.path(), &first.attempt_id, "first.raw.json");
        assert_eq!(first_raw["executionRecords"][0]["exitCode"], 0);
        assert_eq!(
            first_raw["supervisorReceipt"]["descendantsReapedToEchild"],
            true
        );
        assert_eq!(
            first_raw["supervisorReceipt"]["attemptId"],
            first.attempt_id
        );
        assert_eq!(first_raw["processesSpawned"], 1);
        assert!(Path::new(&first.isolated_worktree).is_dir());
        assert!(
            worktree_is_registered(repository.path(), Path::new(&first.isolated_worktree))
                .expect("old worktree registration")
        );

        let second = run_supervisor_case(&input, &first.isolated_worktree);
        worktrees.retain_for_assertions(&second.isolated_worktree);
        let second_raw = save_attempt_json(state.path(), &second.attempt_id, "second.raw.json");

        assert_eq!(first_raw["executionOutcome"], "passed");
        assert_eq!(first_raw["executionEvidenceComplete"], true);
        assert_eq!(first_raw["cleanupDisposition"], "deferred");
        assert_eq!(first_raw["cleanup"]["attempted"], false);
        assert_eq!(first_raw["cleanup"]["removed"], false);
        let diagnostic = first_raw["cleanup"]["error"]
            .as_str()
            .expect("durable process observation diagnostic");
        assert!(diagnostic.contains("phase=procfs.cwd"), "{diagnostic}");
        assert!(
            diagnostic.contains(&format!("errno={}", libc::EACCES)),
            "{diagnostic}"
        );
        assert!(
            diagnostic.contains("identity_state=observed"),
            "{diagnostic}"
        );
        assert_eq!(second_raw["processesSpawned"], 1);
        assert_eq!(second_raw["executionOutcome"], "passed");
        assert_ne!(second.attempt_id, first.attempt_id);
        assert_ne!(second.isolated_worktree, first.isolated_worktree);
        assert_ne!(
            second
                .supervisor_receipt
                .as_ref()
                .map(|receipt| &receipt.run_nonce),
            first
                .supervisor_receipt
                .as_ref()
                .map(|receipt| &receipt.run_nonce)
        );
        assert_eq!(second.execution_records.len(), 1);
        assert!(!second.execution_records[0].reused);
        assert!(Path::new(&first.isolated_worktree).is_dir());
        assert!(
            worktree_is_registered(repository.path(), Path::new(&first.isolated_worktree))
                .expect("old registration remains after fresh retry")
        );
    }

    #[test]
    fn external_unknown_without_current_owned_proof_retains_unknown() {
        let (repository, state, input) = composition_case();
        let mut worktrees = OwnedWorktrees::new(repository.path());
        let first = run_supervisor_case(&input, "owned-proof-unknown");
        worktrees.retain_for_assertions(&first.isolated_worktree);
        let raw = save_attempt_json(state.path(), &first.attempt_id, "no-proof.raw.json");
        assert_eq!(raw["executionRecords"][0]["exitCode"], 0);
        assert_eq!(raw["supervisorReceipt"]["descendantsReapedToEchild"], false);
        assert_eq!(raw["ownedTreeTerminationUnknown"], true);
        assert_eq!(raw["executionOutcome"], "unknown");
        assert_eq!(raw["executionEvidenceComplete"], false);
        assert_eq!(raw["cleanupDisposition"], "retained");
        assert!(Path::new(&first.isolated_worktree).is_dir());
        let retry = run_supervisor_case_result(&input, "", 1)
            .expect_err("clean external observer cannot replace missing owned-tree proof");
        assert!(
            retry.contains("has no verifiable owner; preserving its worktree"),
            "unexpected recovery rejection: {retry}"
        );
        assert_eq!(
            fs::read(state.path().join("a12-verifier-spawns")).expect("spawn marker"),
            b"x",
            "retry must not start another verifier"
        );
        assert!(Path::new(&first.isolated_worktree).is_dir());
        assert!(
            worktree_is_registered(repository.path(), Path::new(&first.isolated_worktree))
                .expect("old worktree registration")
        );
    }

    #[test]
    fn external_unknown_without_coherent_owned_binding_blocks_clean_reconcile() {
        let _guard = LINUX_PROCESS_OBSERVATION_TEST_LOCK
            .lock()
            .expect("real EACCES observer tests are serialized");
        if !real_eacces_is_testable_without_ptrace_capability() {
            return;
        }
        let (repository, state, input) = composition_case();
        let mut worktrees = OwnedWorktrees::new(repository.path());
        let first = run_supervisor_case_with_generation(&input, "real-eacces", 0);
        worktrees.retain_for_assertions(&first.isolated_worktree);
        let raw = save_attempt_json(state.path(), &first.attempt_id, "unbound-proof.raw.json");
        assert_eq!(raw["supervisorReceipt"]["descendantsReapedToEchild"], true);
        assert_eq!(raw["ownedTreeTerminationUnknown"], true);
        assert_eq!(raw["cleanupDisposition"], "retained");
        let diagnostic = raw["failure"]
            .as_str()
            .expect("durable process observation diagnostic");
        assert!(diagnostic.contains("phase=procfs.cwd"), "{diagnostic}");
        assert!(
            diagnostic.contains(&format!("errno={}", libc::EACCES)),
            "{diagnostic}"
        );
        assert!(
            diagnostic.contains("identity_state=observed"),
            "{diagnostic}"
        );
        let mut legacy_json = raw.clone();
        legacy_json
            .as_object_mut()
            .expect("attempt JSON object")
            .remove("ownedTreeTerminationUnknown");
        let legacy: CompositionAttempt =
            serde_json::from_value(legacy_json).expect("pre-guard attempt format");
        let legacy_error = reconcile_abandoned_attempt_with_observer(
            repository.path(),
            state.path(),
            Some(legacy),
            &|_| Ok(None),
        )
        .expect_err("legacy retained unknown cannot be cleaned by external Ok(None)");
        assert!(matches!(
            legacy_error,
            CompositionError::UnknownAttemptOwner { .. }
        ));
        let retry = run_supervisor_case_result(&input, "", 1)
            .expect_err("external Ok(None) cannot repair incoherent owned proof");
        assert!(
            retry.contains("has no verifiable owner; preserving its worktree"),
            "unexpected recovery rejection: {retry}"
        );
        assert_eq!(
            fs::read(state.path().join("a12-verifier-spawns")).expect("spawn marker"),
            b"x"
        );
        assert!(Path::new(&first.isolated_worktree).is_dir());
        assert!(
            worktree_is_registered(repository.path(), Path::new(&first.isolated_worktree))
                .expect("old worktree registration")
        );
    }

    #[test]
    fn failed_execution_result_survives_external_unknown_with_owned_proof() {
        let _guard = LINUX_PROCESS_OBSERVATION_TEST_LOCK
            .lock()
            .expect("real EACCES observer tests are serialized");
        if !real_eacces_is_testable_without_ptrace_capability() {
            return;
        }
        let (repository, state, mut input) = composition_case();
        input.commands[0].program = "false".into();
        let mut worktrees = OwnedWorktrees::new(repository.path());
        let first = run_supervisor_case(&input, "real-eacces");
        worktrees.retain_for_assertions(&first.isolated_worktree);
        let raw = save_attempt_json(state.path(), &first.attempt_id, "failed-execution.raw.json");
        assert_eq!(raw["executionRecords"][0]["exitCode"], 1);
        assert_eq!(raw["executionOutcome"], "failed");
        assert_eq!(raw["executionEvidenceComplete"], true);
        assert_eq!(raw["cleanupDisposition"], "deferred");
        let diagnostic = raw["cleanup"]["error"]
            .as_str()
            .expect("durable process observation diagnostic");
        assert!(diagnostic.contains("phase=procfs.cwd"), "{diagnostic}");
        assert!(
            diagnostic.contains(&format!("errno={}", libc::EACCES)),
            "{diagnostic}"
        );
        assert!(
            diagnostic.contains("identity_state=observed"),
            "{diagnostic}"
        );
        assert!(
            raw["failure"]
                .as_str()
                .is_some_and(|failure| failure.starts_with("command_failed:"))
        );
        assert!(Path::new(&first.isolated_worktree).is_dir());
    }

    #[test]
    fn node_without_observable_inputs_executes_again_under_clean_observer() {
        let (_repository, _state, mut input) = composition_case();
        input.commands[0].program = "sh".into();
        input.commands[0].args = vec!["-c".into(), "true".into()];
        let first = run_supervisor_case(&input, "");
        assert!(first.passed, "first clean observation: {first:?}");
        let second = run_supervisor_case(&input, "");
        assert!(second.passed, "second clean observation: {second:?}");
        assert_eq!(second.processes_spawned, 1);
        assert!(!second.execution_records[0].reused);
        assert_eq!(second.reuse_decision.kind, ReuseDecisionKind::Unknown);
        assert_ne!(first.attempt_id, second.attempt_id);
        assert!(!Path::new(&first.isolated_worktree).exists());
        assert!(!Path::new(&second.isolated_worktree).exists());
    }

    #[test]
    fn external_unknown_retry_rejects_known_active_process_and_unsafe_command() {
        for mode in ["active", "unsafe", "denied", "stale"] {
            let (repository, state, input) = composition_case();
            let mut worktrees = OwnedWorktrees::new(repository.path());
            let first = run_supervisor_case(&input, "*");
            worktrees.retain_for_assertions(&first.isolated_worktree);
            assert_eq!(
                first.cleanup_disposition,
                CompositionCleanupDisposition::Deferred
            );
            let mut retry_input = input.clone();
            let observation = if mode == "active" {
                format!("active:{}", first.isolated_worktree)
            } else if mode == "unsafe" {
                retry_input.commands[0].program = "sh".into();
                retry_input.commands[0].args = vec!["-c".into(), "true".into()];
                first.isolated_worktree.clone()
            } else if mode == "denied" {
                format!("deny:{}", first.isolated_worktree)
            } else {
                first.isolated_worktree.clone()
            };
            let second = run_supervisor_case_with_generation(
                &retry_input,
                &observation,
                if mode == "stale" { 0 } else { 1 },
            );
            let raw = save_attempt_json(
                state.path(),
                &second.attempt_id,
                &format!("{mode}-retry.raw.json"),
            );
            assert_eq!(raw["processesSpawned"], 0);
            let failure = raw["failure"].as_str().expect("failed retry reason");
            if mode == "denied" {
                assert_eq!(failure, "command_failed:exit=None");
                assert_eq!(
                    raw["executionRecords"][0]["stderr"],
                    "injected current admission denial"
                );
                assert!(!Path::new(&second.isolated_worktree).exists());
            } else {
                assert_eq!(raw["isolatedWorktree"], "");
                assert!(failure.starts_with("unsafe_deferred_cleanup_retry:"));
            }
            assert!(Path::new(&first.isolated_worktree).is_dir());
            assert!(
                worktree_is_registered(repository.path(), Path::new(&first.isolated_worktree))
                    .expect("old worktree registration stays intact")
            );
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod verifier_process_observation_tests {
    use super::{
        LINUX_PROCESS_OBSERVATION_TEST_LOCK, create_private_composition_parent,
        linux_process_observation_error, parse_linux_process_stat, read_linux_process_identity,
        reconciliation_error_category, reconciliation_known_observer_error,
        unique_composition_parent, verifier_process_using_worktree,
    };
    use std::ffi::CString;
    use std::fs;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::time::Duration;

    struct TestCompositionParent(PathBuf);

    impl Drop for TestCompositionParent {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct InaccessibleProcess(libc::pid_t);

    impl Drop for InaccessibleProcess {
        fn drop(&mut self) {
            // SAFETY: this test owns the child PID returned by fork.
            unsafe {
                libc::kill(self.0, libc::SIGKILL);
                libc::waitpid(self.0, std::ptr::null_mut(), 0);
            }
        }
    }

    #[test]
    fn inaccessible_runner_ancestors_do_not_block_fresh_worktree_cleanup() {
        let _guard = LINUX_PROCESS_OBSERVATION_TEST_LOCK
            .lock()
            .expect("process observation tests are serialized");
        let parent = unique_composition_parent();
        create_private_composition_parent(&parent).expect("private composition parent");
        let worktree = parent.join("composition");
        fs::create_dir(&worktree).expect("composition worktree directory");

        let observed = verifier_process_using_worktree(&worktree);
        let removed = fs::remove_dir_all(&parent);

        assert!(removed.is_ok(), "test composition parent is cleaned up");
        assert_eq!(
            observed,
            Ok(None),
            "an ancestor that cannot have inherited a newly created private worktree must not make its process state unknown"
        );
    }

    #[test]
    fn inaccessible_process_started_before_private_worktree_does_not_block_cleanup() {
        let _guard = LINUX_PROCESS_OBSERVATION_TEST_LOCK
            .lock()
            .expect("process observation tests are serialized");
        if unsafe { libc::geteuid() } == 0 {
            return;
        }

        let mut ready_pipe = [0; 2];
        // SAFETY: ready_pipe points to two writable file descriptors.
        assert_eq!(unsafe { libc::pipe(ready_pipe.as_mut_ptr()) }, 0);
        // SAFETY: the test child uses only async-signal-safe libc calls after
        // fork, and exits before this test returns.
        let child_pid = unsafe { libc::fork() };
        assert!(child_pid >= 0, "fork test process");
        if child_pid == 0 {
            // SAFETY: the child owns the write end and does not use Rust APIs.
            unsafe {
                libc::close(ready_pipe[0]);
                if libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) != 0 {
                    libc::_exit(111);
                }
                let ready = b'R';
                if libc::write(ready_pipe[1], (&ready as *const u8).cast(), 1) != 1 {
                    libc::_exit(112);
                }
                libc::close(ready_pipe[1]);
                loop {
                    libc::pause();
                }
            }
        }
        let child = InaccessibleProcess(child_pid);
        // SAFETY: the parent closes the unused write end and waits for the
        // child's readiness byte before creating the private directory.
        unsafe {
            libc::close(ready_pipe[1]);
        }
        let mut ready = 0_u8;
        // SAFETY: ready points to one writable byte and the read descriptor is
        // open in the parent.
        assert_eq!(
            unsafe { libc::read(ready_pipe[0], (&mut ready as *mut u8).cast(), 1) },
            1
        );
        // SAFETY: the parent owns the read descriptor.
        unsafe {
            libc::close(ready_pipe[0]);
        }
        assert_eq!(ready, b'R');
        std::thread::sleep(Duration::from_millis(3_100));

        let parent = unique_composition_parent();
        create_private_composition_parent(&parent).expect("private composition parent");
        let worktree = parent.join("composition");
        fs::create_dir(&worktree).expect("composition worktree directory");
        let observed = verifier_process_using_worktree(&worktree);
        drop(child);
        let removed = fs::remove_dir_all(&parent);

        assert!(removed.is_ok(), "test composition parent is cleaned up");
        assert_eq!(
            observed,
            Ok(None),
            "an inaccessible process that predates the private worktree cannot own its cwd or inherited file descriptors"
        );
    }

    #[test]
    fn fresh_same_uid_nondumpable_process_reports_eacces_and_observable_identity() {
        let _guard = LINUX_PROCESS_OBSERVATION_TEST_LOCK
            .lock()
            .expect("process observation tests are serialized");
        if unsafe { libc::geteuid() } == 0 {
            return;
        }

        // Let any unrelated inaccessible same-UID process predate this test's
        // private directory, so the controlled child is the only new candidate.
        std::thread::sleep(Duration::from_millis(3_100));
        let parent = TestCompositionParent(unique_composition_parent());
        create_private_composition_parent(&parent.0).expect("private composition parent");
        let worktree = parent.0.join("composition");
        fs::create_dir(&worktree).expect("composition worktree directory");
        let worktree_c =
            CString::new(worktree.as_os_str().as_bytes()).expect("worktree path contains no NUL");

        let mut ready_pipe = [0; 2];
        // SAFETY: ready_pipe points to two writable file descriptors.
        assert_eq!(unsafe { libc::pipe(ready_pipe.as_mut_ptr()) }, 0);
        // SAFETY: the child uses only async-signal-safe libc calls after fork.
        let child_pid = unsafe { libc::fork() };
        assert!(child_pid >= 0, "fork controlled inaccessible process");
        if child_pid == 0 {
            // SAFETY: the child owns the pipe write end and the worktree path
            // points to inherited, NUL-terminated bytes.
            unsafe {
                libc::close(ready_pipe[0]);
                if libc::chdir(worktree_c.as_ptr()) != 0 {
                    libc::_exit(111);
                }
                if libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) != 0 {
                    libc::_exit(112);
                }
                let ready = b'R';
                if libc::write(ready_pipe[1], (&ready as *const u8).cast(), 1) != 1 {
                    libc::_exit(113);
                }
                libc::close(ready_pipe[1]);
                loop {
                    libc::pause();
                }
            }
        }
        let child = InaccessibleProcess(child_pid);
        // SAFETY: the parent closes the unused write end and waits for the
        // child's chdir and non-dumpable transition before observing procfs.
        unsafe {
            libc::close(ready_pipe[1]);
        }
        let mut ready = 0_u8;
        // SAFETY: ready points to one writable byte and the parent owns the
        // pipe read end.
        assert_eq!(
            unsafe { libc::read(ready_pipe[0], (&mut ready as *mut u8).cast(), 1) },
            1
        );
        // SAFETY: the parent owns the pipe read descriptor.
        unsafe {
            libc::close(ready_pipe[0]);
        }
        assert_eq!(ready, b'R');

        let process_stat = fs::read_to_string(format!("/proc/{child_pid}/stat"))
            .expect("controlled process stat remains readable");
        let identity = parse_linux_process_stat(child_pid as u32, &process_stat)
            .expect("controlled process identity");
        let denied_cwd = fs::read_link(format!("/proc/{child_pid}/cwd"))
            .expect_err("non-dumpable child cwd must be inaccessible");
        assert_eq!(denied_cwd.raw_os_error(), Some(libc::EACCES));

        let observed = verifier_process_using_worktree(&worktree)
            .expect_err("real EACCES must remain an unknown process observation");
        assert!(
            observed.contains(&format!("pid={child_pid}")),
            "diagnostic must bind the inaccessible PID: {observed}"
        );
        assert!(
            observed.contains("phase=procfs.cwd"),
            "diagnostic must identify the failed observation phase: {observed}"
        );
        assert!(
            observed.contains(&format!("errno={}", libc::EACCES)),
            "diagnostic must retain raw EACCES: {observed}"
        );
        assert!(
            observed.contains("identity_state=observed")
                && observed.contains(&format!("filter_uid=Some({})", unsafe { libc::getuid() }))
                && !observed.contains(&format!(" uid=Some({}", unsafe { libc::getuid() }))
                && observed.contains(&format!("starttime_ticks={}", identity.start_time_ticks))
                && observed.contains(&format!("pgid={}", identity.process_group_id))
                && observed.contains(&format!("sid={}", identity.session_id)),
            "diagnostic must retain the readable process identity: {observed}"
        );

        drop(child);
        drop(parent);
    }

    #[test]
    fn unreadable_process_identity_is_explicitly_unknown_in_diagnostic() {
        if unsafe { libc::geteuid() } == 0 {
            return;
        }
        let parent = TestCompositionParent(unique_composition_parent());
        let process_id = 42_424_u32;
        let stat_path = parent.0.join(process_id.to_string()).join("stat");
        fs::create_dir_all(stat_path.parent().expect("stat parent"))
            .expect("create synthetic proc entry");
        fs::write(&stat_path, "synthetic identity bytes").expect("write identity fixture");
        fs::set_permissions(&stat_path, fs::Permissions::from_mode(0o000))
            .expect("make synthetic process identity unreadable");
        let before = read_linux_process_identity(&parent.0, process_id);
        assert!(
            before
                .as_ref()
                .is_err_and(|error| error.contains("errno=13")),
            "synthetic proc stat read should fail with EACCES: {before:?}"
        );
        let denied = std::io::Error::from_raw_os_error(libc::EACCES);

        let diagnostic = linux_process_observation_error(
            &parent.0,
            process_id,
            Some(unsafe { libc::getuid() }),
            "procfs.cwd",
            &denied,
            &before,
        );

        assert!(diagnostic.contains("errno=13"), "{diagnostic}");
        assert!(
            diagnostic.contains("identity_state=unknown"),
            "{diagnostic}"
        );
        assert!(
            diagnostic.contains("identity_before=unknown("),
            "{diagnostic}"
        );
        assert!(
            diagnostic.contains("identity_after=unknown("),
            "{diagnostic}"
        );
        drop(parent);
    }

    #[test]
    fn recovery_trace_extracts_structured_process_observation_fields() {
        let error = "verifier_process_state_unknown:cannot inspect process pid=4242 filter_uid=Some(1000) phase=procfs.cwd error_kind=PermissionDenied errno=13 identity_state=observed identity_before=observed(pid=4242,starttime_ticks=7,pgid=41,sid=41,state=S) identity_after=observed(pid=4242,starttime_ticks=7,pgid=41,sid=41,state=S) message=Permission denied (os error 13)";

        assert_eq!(
            reconciliation_known_observer_error(error),
            "pid=4242 phase=procfs.cwd errno=13 identity_state=observed"
        );
    }

    #[test]
    fn recovery_trace_category_ignores_nested_identity_read_error_kind() {
        let error = "cannot inspect process pid=4242 filter_uid=Some(1000) phase=procfs.cwd error_kind=Other errno=5 identity_state=unknown identity_before=unknown(identity stat read failed error_kind=PermissionDenied errno=13) identity_after=unknown(identity stat read failed error_kind=PermissionDenied errno=13)";

        assert_eq!(
            reconciliation_error_category(error),
            "proc_observation_error"
        );
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

#[cfg(all(test, windows))]
mod path_containment_tests {
    use super::{
        CompositionCommand, controlled_command_environment, path_is_within, resolve_executable,
        trusted_executable_directories,
    };
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;

    #[test]
    fn trusted_windows_path_prefix_ignores_case_but_respects_component_boundaries() {
        assert!(path_is_within(
            Path::new(r"C:\Windows\System32"),
            Path::new(r"c:\windows\system32\cmd.exe")
        ));
        assert!(!path_is_within(
            Path::new(r"C:\Windows\System32"),
            Path::new(r"C:\Windows\System32-evil\cmd.exe")
        ));
    }

    #[test]
    fn controlled_composition_environment_can_resolve_windows_powershell() {
        let system_root = std::env::var_os("SystemRoot").expect("Windows SystemRoot");
        let powershell_dir =
            fs::canonicalize(Path::new(&system_root).join(r"System32\WindowsPowerShell\v1.0"))
                .expect("Windows PowerShell installation directory");
        let command = CompositionCommand {
            node_id: "powershell".into(),
            program: "powershell.exe".into(),
            args: Vec::new(),
            depends_on: Vec::new(),
            environment: BTreeMap::new(),
            input_paths: Vec::new(),
            covered_scenarios: Vec::new(),
            covered_constraints: Vec::new(),
        };
        let environment =
            controlled_command_environment(&command).expect("controlled composition environment");

        assert!(
            trusted_executable_directories().contains(&powershell_dir),
            "PowerShell is explicitly trusted without inheriting ambient PATH"
        );
        assert!(
            resolve_executable(&std::env::temp_dir(), &command.program, &environment).is_some(),
            "Windows composition fixtures can resolve their explicit PowerShell command"
        );
    }
}
