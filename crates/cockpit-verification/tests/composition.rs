use cockpit_core::Digest;
use cockpit_protocol::{COLLABORATION_CAPABILITY, CompositionBinding, RuntimeCapabilityBinding};
use cockpit_verification::composition::CompositionCleanup;
use cockpit_verification::{
    CompositionAttempt, CompositionCleanupDisposition, CompositionCommand, CompositionError,
    CompositionExecutionOutcome, CompositionExecutionRecord, CompositionIdentity, CompositionInput,
    CompositionPrecondition, CompositionSupervisorBackend, CompositionSupervisorReceipt,
    ProcessAdmissionCheck, ProcessStartGate, ReuseDecision, ReuseDecisionKind, classify_reuse,
    composition_commands_digest, composition_linux_boot_id, current_composition_process_identity,
    execution_records_digest, initialize_composition_supervisor_backend, new_composition_run_nonce,
    new_supervised_composition_attempt_id, reap_composition_supervisor_descendants,
    run_composition, run_composition_with_process_gates, run_composition_with_supervisor_receipt,
};
#[cfg(target_os = "linux")]
use cockpit_verification::{
    TestCompletedWorktreeObservation, run_composition_with_test_completed_worktree_observation,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn tempdir(label: &str) -> TempDir {
    let sequence = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "cockpit-composition-{label}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("temp directory");
    TempDir(path)
}

fn run_composition_with_test_supervisor(
    input: CompositionInput,
) -> Result<cockpit_verification::CompositionAttempt, String> {
    run_composition_with_named_test_supervisor(input, "composition_supervisor_run_test_helper")
}

fn run_composition_with_pure_cache_test_supervisor(
    input: CompositionInput,
) -> Result<cockpit_verification::CompositionAttempt, String> {
    run_composition_with_named_test_supervisor(
        input,
        "composition_pure_cache_supervisor_run_test_helper",
    )
}

#[cfg(target_os = "linux")]
#[test]
fn pure_cache_observation_interface_cannot_supply_admission_or_start_gate() {
    let _: fn(
        CompositionInput,
        CompositionSupervisorReceipt,
        TestCompletedWorktreeObservation,
    ) -> Result<CompositionAttempt, CompositionError> =
        run_composition_with_test_completed_worktree_observation;
}

fn run_composition_with_named_test_supervisor(
    input: CompositionInput,
    helper_name: &str,
) -> Result<cockpit_verification::CompositionAttempt, String> {
    let helper_files = tempdir("supervisor-runner");
    let input_path = helper_files.path().join("input.json");
    let output_path = helper_files.path().join("result.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&input).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let output = Command::new(std::env::current_exe().map_err(|error| error.to_string())?)
        .args(["--exact", helper_name, "--nocapture"])
        .env("AI_COCKPIT_RUN_COMPOSITION_TEST_HELPER_INPUT", &input_path)
        .env(
            "AI_COCKPIT_RUN_COMPOSITION_TEST_HELPER_OUTPUT",
            &output_path,
        )
        .env("AI_COCKPIT_COMPOSITION_RECONCILE_TRACE", "1")
        .output()
        .map_err(|error| format!("run isolated composition supervisor helper: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "composition supervisor helper failed: stdout={}; stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let result: Result<cockpit_verification::CompositionAttempt, String> = serde_json::from_slice(
        &fs::read(&output_path)
            .map_err(|error| format!("read composition supervisor helper result: {error}"))?,
    )
    .map_err(|error| format!("decode composition supervisor helper result: {error}"))?;
    result.map_err(|error| {
        format!(
            "{error}; isolated helper diagnostic: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[test]
fn composition_supervisor_run_test_helper() {
    run_composition_supervisor_test_helper(false);
}

#[test]
fn composition_pure_cache_supervisor_run_test_helper() {
    run_composition_supervisor_test_helper(true);
}

fn run_composition_supervisor_test_helper(pure_cache: bool) {
    let (Ok(input_path), Ok(output_path)) = (
        std::env::var("AI_COCKPIT_RUN_COMPOSITION_TEST_HELPER_INPUT"),
        std::env::var("AI_COCKPIT_RUN_COMPOSITION_TEST_HELPER_OUTPUT"),
    ) else {
        return;
    };
    let input: CompositionInput =
        serde_json::from_slice(&fs::read(&input_path).expect("composition helper input bytes"))
            .expect("composition helper input JSON");
    let backend = initialize_composition_supervisor_backend().expect("initialize subreaper");
    let process_identity = current_composition_process_identity().expect("process identity");
    let run_nonce = new_composition_run_nonce();
    let attempt_id = new_supervised_composition_attempt_id(&input, &run_nonce);
    let target_tree = run(
        &input.repository_root,
        &[
            "rev-parse",
            "--verify",
            &format!("{}^{{tree}}", input.binding.target_sha),
        ],
    );
    let input_environment = input
        .commands
        .iter()
        .map(|command| (&command.node_id, &command.environment))
        .collect::<Vec<_>>();
    let receipt = CompositionSupervisorReceipt {
        schema_version: 1,
        backend,
        attempt_id,
        run_nonce,
        generation: 1,
        owner: process_identity.clone(),
        supervisor: process_identity,
        linux_boot_id: composition_linux_boot_id().expect("Linux boot identity"),
        environment_digest: digest("test supervisor environment"),
        runtime_version: input.binding.verifier.runtime_version.clone(),
        runtime_digest: input.binding.verifier.runtime_digest.clone(),
        repository_id: input.binding.repository_id.clone(),
        target_sha: input.binding.target_sha.clone(),
        snapshot_digest: Digest::sha256_bytes(target_tree.as_bytes()),
        command_plan_digest: composition_commands_digest(&input.commands),
        input_environment_digest: Digest::sha256_bytes(
            &serde_json::to_vec(&input_environment).expect("input environments serialize"),
        ),
        execution_records_digest: execution_records_digest(&[]),
        descendants_reaped_to_echild: false,
    };
    let admission_check: ProcessAdmissionCheck = Arc::new(|_node_id, accept| accept());
    let process_start_gate: ProcessStartGate = Arc::new(|_node_id, spawn| spawn());
    let result = if pure_cache {
        #[cfg(target_os = "linux")]
        {
            run_composition_with_test_completed_worktree_observation(
                input,
                receipt,
                TestCompletedWorktreeObservation::KnownEmpty,
            )
        }
        #[cfg(not(target_os = "linux"))]
        {
            run_composition_with_supervisor_receipt(
                input,
                admission_check,
                process_start_gate,
                receipt,
            )
        }
    } else {
        run_composition_with_supervisor_receipt(input, admission_check, process_start_gate, receipt)
    }
    .map_err(|error| error.to_string());
    fs::write(
        output_path,
        serde_json::to_vec(&result).expect("helper result serializes"),
    )
    .expect("write composition helper result");
}

#[cfg(target_os = "linux")]
fn linux_process_group_proc_observation(process_group_id: u32) -> String {
    let group_probe = unsafe { libc::kill(-(process_group_id as libc::pid_t), 0) };
    let group_probe_error = (group_probe < 0).then(std::io::Error::last_os_error);
    let leader_stat = fs::read_to_string(format!("/proc/{process_group_id}/stat"));
    let mut members = Vec::new();
    let mut unreadable_stats = Vec::new();
    let mut unreadable_stat_count = 0usize;
    let mut malformed_stats = Vec::new();
    let mut malformed_stat_count = 0usize;
    let mut directory_errors = Vec::new();
    let mut directory_error_count = 0usize;

    match fs::read_dir("/proc") {
        Ok(entries) => {
            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(error) => {
                        directory_error_count += 1;
                        if directory_errors.len() < 8 {
                            directory_errors.push(error.to_string());
                        }
                        continue;
                    }
                };
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
                    Err(error) => {
                        unreadable_stat_count += 1;
                        if unreadable_stats.len() < 8 {
                            unreadable_stats.push(format!("{}: {error}", stat_path.display()));
                        }
                        continue;
                    }
                };
                let Some(command_open) = stat.find('(') else {
                    malformed_stat_count += 1;
                    if malformed_stats.len() < 8 {
                        malformed_stats.push(format!("{process_id}: missing command opener"));
                    }
                    continue;
                };
                let Some(command_close) = stat.rfind(')').filter(|close| *close > command_open)
                else {
                    malformed_stat_count += 1;
                    if malformed_stats.len() < 8 {
                        malformed_stats.push(format!("{process_id}: missing command terminator"));
                    }
                    continue;
                };
                let fields = stat[command_close + 1..]
                    .split_whitespace()
                    .collect::<Vec<_>>();
                if fields.len() <= 19 {
                    malformed_stat_count += 1;
                    if malformed_stats.len() < 8 {
                        malformed_stats.push(format!("{process_id}: too few stat fields"));
                    }
                    continue;
                }
                if fields[2].parse::<u32>().ok() == Some(process_group_id) {
                    members.push(format!(
                        "pid={process_id} state={} ppid={} pgid={} sid={} start={}",
                        fields[0], fields[1], fields[2], fields[3], fields[19]
                    ));
                }
            }
        }
        Err(error) => {
            directory_error_count += 1;
            directory_errors.push(error.to_string());
        }
    }

    format!(
        "kill(-{process_group_id}, 0)={group_probe} error={group_probe_error:?}; \
         leader_stat={leader_stat:?}; members={members:?}; \
         unreadable_stat_count={unreadable_stat_count} samples={unreadable_stats:?}; \
         malformed_stat_count={malformed_stat_count} samples={malformed_stats:?}; \
         proc_directory_error_count={directory_error_count} samples={directory_errors:?}"
    )
}

fn digest(label: &str) -> Digest {
    Digest::sha256_bytes(label.as_bytes())
}

fn run(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("git command");
    assert!(output.status.success(), "git {:?} failed", args);
    String::from_utf8(output.stdout)
        .expect("utf8")
        .trim()
        .into()
}

fn repository() -> TempDir {
    let root = tempdir("repository");
    run(root.path(), &["init", "-q"]);
    run(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    run(root.path(), &["config", "user.name", "Test"]);
    fs::write(root.path().join("README.md"), "initial\n").expect("write");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "initial"]);
    run(root.path(), &["branch", "-M", "main"]);
    root
}

fn runtime() -> RuntimeCapabilityBinding {
    RuntimeCapabilityBinding {
        schema_version: 1,
        runtime_version: "0.2.113".into(),
        runtime_digest: digest("candidate-runtime"),
        capability: COLLABORATION_CAPABILITY.into(),
    }
}

fn identity(label: &str) -> CompositionIdentity {
    CompositionIdentity {
        source_digest: digest(&format!("source-{label}")),
        dependency_digest: digest(&format!("dependency-{label}")),
        interface_digest: digest(&format!("interface-{label}")),
        configuration_digest: digest(&format!("configuration-{label}")),
        toolchain_digest: digest(&format!("toolchain-{label}")),
        lockfile_digest: digest(&format!("lockfile-{label}")),
        generated_input_digest: digest(&format!("generated-{label}")),
        environment_digest: digest(&format!("environment-{label}")),
        verifier_digest: digest(&format!("verifier-{label}")),
        command_digest: digest("placeholder-command-digest"),
    }
}

fn binding(target_sha: &str, participant_heads: Vec<String>) -> CompositionBinding {
    CompositionBinding {
        schema_version: 1,
        repository_id: digest("repository"),
        binding_id: "composition-test".into(),
        target_branch: "main".into(),
        target_sha: target_sha.into(),
        participant_work_items: vec!["WI-PROVIDER".into(), "WI-CONSUMER".into()],
        participant_heads,
        contract_digests: vec![digest("provider-contract"), digest("consumer-contract")],
        verifier: runtime(),
    }
}

fn command(node_id: &str, program: &str, args: &[&str]) -> CompositionCommand {
    #[cfg(windows)]
    let (program, args): (&str, Vec<String>) = match (program, args) {
        ("true", []) | ("sh", ["-c", "true"]) => ("cmd.exe", vec!["/C".into(), "exit 0".into()]),
        ("false", []) | ("sh", ["-c", "false"]) => ("cmd.exe", vec!["/C".into(), "exit 1".into()]),
        ("sh", ["-c", "exit 17"]) => ("cmd.exe", vec!["/C".into(), "exit 17".into()]),
        ("sh", ["-c", "test -f required-interface.txt"]) => (
            "powershell.exe",
            vec![
                "-NoProfile".into(),
                "-Command".into(),
                "if (Test-Path required-interface.txt) { exit 0 } else { exit 1 }".into(),
            ],
        ),
        ("sh", ["-c", "sleep 2"]) => (
            "powershell.exe",
            vec![
                "-NoProfile".into(),
                "-Command".into(),
                "Start-Sleep -Seconds 2".into(),
            ],
        ),
        ("sh", ["-c", command]) if command.starts_with("test \"$COMPOSITION_FLAVOR\"") => (
            "powershell.exe",
            vec![
                "-NoProfile".into(),
                "-Command".into(),
                "if ($env:COMPOSITION_FLAVOR -eq 'one') { exit 0 } else { exit 1 }".into(),
            ],
        ),
        ("sh", ["-c", command]) if command.starts_with("touch ") => (
            "powershell.exe",
            vec![
                "-NoProfile".into(),
                "-Command".into(),
                format!(
                    "New-Item -ItemType File -Path '{}' -Force | Out-Null",
                    command.trim_start_matches("touch ")
                ),
            ],
        ),
        ("sh", ["-c", script, "sh", path]) if script.starts_with("printf started >") => (
            "powershell.exe",
            vec![
                "-NoProfile".into(),
                "-Command".into(),
                format!("Set-Content -NoNewline -Path '{}' -Value started", path),
            ],
        ),
        ("env", []) => ("cmd.exe", vec!["/C".into(), "set".into()]),
        ("cat", [path]) => ("cmd.exe", vec!["/C".into(), "type".into(), (*path).into()]),
        // Unknown shell snippets remain unbound instead of being silently
        // replaced by a successful no-op on Windows.
        ("sh", _) => ("ai-cockpit-unsupported-shell-command", Vec::new()),
        (program, args) => (program, args.iter().map(|arg| (*arg).into()).collect()),
    };
    #[cfg(not(windows))]
    let (program, args) = (program, args.iter().map(|arg| (*arg).into()).collect());
    CompositionCommand {
        node_id: node_id.into(),
        program: program.into(),
        args,
        depends_on: Vec::new(),
        environment: Default::default(),
        input_paths: vec!["README.md".into()],
        covered_scenarios: Vec::new(),
        covered_constraints: Vec::new(),
    }
}

#[test]
fn composition_commands_accept_explicit_upstream_dependencies() {
    let command: CompositionCommand = serde_json::from_value(serde_json::json!({
        "nodeId": "consumer",
        "program": "sh",
        "args": ["-c", "true"],
        "dependsOn": ["provider"]
    }))
    .expect("command dependency protocol");

    assert_eq!(command.node_id, "consumer");
    assert_eq!(
        serde_json::to_value(command)
            .expect("serialize command")
            .get("dependsOn")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(1)
    );
}

#[test]
fn composition_rejects_dependencies_that_are_not_prior_nodes_before_spawn() {
    let root = repository();
    let head = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let mut consumer = command("consumer", "sh", &["-c", "true"]);
    consumer.depends_on = vec!["provider".into()];
    let attempt = run_composition(input(
        root.path(),
        state.path(),
        binding(&head, vec![head.clone(), head.clone()]),
        vec![consumer, command("provider", "sh", &["-c", "true"])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("invalid composition is recorded");

    assert!(!attempt.passed);
    assert_eq!(attempt.processes_spawned, 0);
    assert_eq!(
        attempt.failure.as_deref(),
        Some("invalid_composition_command_graph")
    );
    assert!(attempt.isolated_worktree.is_empty());
}

fn input(
    root: &Path,
    state_dir: &Path,
    binding: CompositionBinding,
    commands: Vec<CompositionCommand>,
    preconditions: Vec<CompositionPrecondition>,
) -> CompositionInput {
    let mut identity = identity("stable");
    identity.command_digest = composition_commands_digest(&commands);
    CompositionInput {
        repository_root: root.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        binding,
        identity,
        reusable_node_ids: commands
            .iter()
            .map(|command| command.node_id.clone())
            .collect(),
        commands,
        preconditions,
        timeout_seconds: 30,
    }
}

fn attempt_record_path(state_dir: &Path, attempt_id: &str) -> PathBuf {
    for entry in fs::read_dir(state_dir).expect("attempt state directory") {
        let path = entry.expect("attempt entry").path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let Ok(attempt) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            continue;
        };
        if attempt["attemptId"].as_str() == Some(attempt_id) {
            return path;
        }
    }
    panic!("attempt record not found for {attempt_id}");
}

#[test]
fn exact_composition_uses_real_linked_worktree_and_finds_interface_error() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-qb", "provider"]);
    fs::write(root.path().join("provider-api.txt"), "api\n").expect("provider file");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "provider"]);
    let provider_head = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-q", "-b", "consumer", &base]);
    fs::write(root.path().join("consumer.txt"), "consumer\n").expect("consumer file");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "consumer"]);
    let consumer_head = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-q", "main"]);
    let state = tempdir("state");
    let missing_interface = "test -f required-interface.txt";
    let attempt = run_composition(input(
        root.path(),
        state.path(),
        binding(&base, vec![provider_head, consumer_head]),
        vec![command("interface-check", "sh", &["-c", missing_interface])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("composition attempt");
    assert!(!attempt.passed);
    assert!(attempt.isolated_worktree.ends_with("composition"));
    assert_eq!(attempt.binding.target_sha, base);
    assert_eq!(attempt.binding.participant_work_items.len(), 2);
    assert_eq!(attempt.execution_records.len(), 1);
    assert!(attempt.execution_records[0].spawned);
    assert!(
        attempt
            .failure
            .as_deref()
            .unwrap_or_default()
            .contains("exit")
    );
}

#[test]
fn failed_precondition_spawns_zero_expensive_processes_and_persists_attempt() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let sentinel = state.path().join("should-not-exist");
    let attempt = run_composition(input(
        root.path(),
        state.path(),
        binding(&base, vec![base.clone(), base.clone()]),
        vec![command(
            "expensive",
            "sh",
            &["-c", &format!("touch {}", sentinel.display())],
        )],
        vec![CompositionPrecondition::unsatisfied(
            "dependency-ready",
            "dependency impact is unresolved",
        )],
    ))
    .expect("precondition attempt");
    assert!(!attempt.passed);
    assert_eq!(attempt.processes_spawned, 0);
    assert!(attempt.execution_records.is_empty());
    assert!(!sentinel.exists());
    assert!(attempt_record_path(state.path(), &attempt.attempt_id).exists());
}

#[test]
fn attempt_record_filename_is_portable_and_preserves_logical_identity() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("portable-attempt-name");
    #[cfg(windows)]
    let portable_name_command = command("portable-name", "cmd", &["/C", "exit 0"]);
    #[cfg(not(windows))]
    let portable_name_command = command("portable-name", "sh", &["-c", "true"]);
    let attempt = run_composition(input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![portable_name_command],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("composition attempt");

    assert!(
        attempt.passed,
        "portable filename attempt failed: {attempt:?}"
    );
    let entries = fs::read_dir(state.path())
        .expect("attempt state directory")
        .map(|entry| entry.expect("attempt entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 1, "one attempt record is expected");
    let file_name = entries[0]
        .file_name()
        .and_then(|name| name.to_str())
        .expect("UTF-8 attempt filename");
    assert!(
        file_name
            .chars()
            .all(|character| character.is_ascii_alphanumeric()
                || matches!(character, '.' | '-' | '_')),
        "attempt filename must use portable characters: {file_name}"
    );
    let persisted: serde_json::Value =
        serde_json::from_slice(&fs::read(&entries[0]).expect("persisted attempt bytes"))
            .expect("persisted attempt JSON");
    assert_eq!(persisted["attemptId"], attempt.attempt_id);
}

#[test]
fn process_start_gate_rejection_is_persisted_without_running_the_node() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let marker = state.path().join("must-not-start");
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command(
            "late-admission",
            "sh",
            &[
                "-c",
                "printf started > \"$1\"",
                "sh",
                marker.to_str().unwrap(),
            ],
        )],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let admission_check: ProcessAdmissionCheck =
        std::sync::Arc::new(|_node_id: &str, accept| accept());
    let gate: ProcessStartGate = std::sync::Arc::new(
        |_node_id: &str, _spawn: &mut dyn FnMut() -> Result<Child, String>| {
            Err("coordination_safely_paused:late-request".into())
        },
    );

    let attempt = run_composition_with_process_gates(composition, admission_check, gate)
        .expect("denial remains a durable composition attempt");

    assert!(!attempt.passed);
    assert_eq!(attempt.processes_spawned, 0);
    assert_eq!(attempt.execution_records.len(), 1);
    assert!(!attempt.execution_records[0].spawned);
    assert!(
        !marker.exists(),
        "the denied node must not create its marker"
    );
    assert!(
        attempt.execution_records[0]
            .stderr
            .contains("coordination_safely_paused")
    );
}

#[test]
fn composition_attempt_reports_execution_and_cleanup_separately() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("execution-cleanup-outcomes");
    let attempt = run_composition(input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("successful", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("composition attempt");

    assert!(
        attempt.passed,
        "composition execution did not pass: {attempt:?}"
    );
    let value = serde_json::to_value(&attempt).expect("serialize composition attempt");
    assert_eq!(value["schemaVersion"], 3);
    assert_eq!(value["executionOutcome"], "passed");
    assert_eq!(value["executionEvidenceComplete"], true);
    assert_eq!(value["cleanupDisposition"], "cleaned");
}

#[test]
fn execution_pass_without_a_supervisor_receipt_is_not_terminal_or_reusable() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("missing-supervisor-receipt");
    let attempt = run_composition(input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("successful", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("composition attempt");

    assert!(
        attempt.passed,
        "composition execution did not pass: {attempt:?}"
    );
    let value = serde_json::to_value(&attempt).expect("serialize composition attempt");
    assert_eq!(value["executionOutcome"], "passed");
    assert_eq!(value["executionEvidenceComplete"], true);
    assert_eq!(value["cleanupDisposition"], "cleaned");
    assert!(attempt.supervisor_receipt.is_none());
    assert!(!attempt.is_coherent_successful_terminal());
}

#[test]
fn coherent_supervisor_receipt_is_required_for_a_successful_terminal() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("coherent-supervisor-receipt");
    let mut attempt = run_composition(input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("successful", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("composition attempt");
    let process_identity =
        current_composition_process_identity().expect("current process identity");
    let boot_id = composition_linux_boot_id().expect("Linux boot identity");
    #[cfg(target_os = "linux")]
    let backend = CompositionSupervisorBackend::LinuxSubreaper;
    #[cfg(all(unix, not(target_os = "linux")))]
    let backend = CompositionSupervisorBackend::UnixProcessGroup;
    #[cfg(windows)]
    let backend = CompositionSupervisorBackend::WindowsProcessGroup;
    let receipt = CompositionSupervisorReceipt {
        schema_version: 1,
        backend,
        attempt_id: attempt.attempt_id.clone(),
        run_nonce: "test-run-nonce".into(),
        generation: 1,
        owner: process_identity.clone(),
        supervisor: process_identity,
        linux_boot_id: if backend == CompositionSupervisorBackend::LinuxSubreaper {
            boot_id
        } else {
            None
        },
        environment_digest: digest("environment"),
        runtime_version: attempt.binding.verifier.runtime_version.clone(),
        runtime_digest: attempt.binding.verifier.runtime_digest.clone(),
        repository_id: attempt.binding.repository_id.clone(),
        target_sha: attempt.binding.target_sha.clone(),
        snapshot_digest: digest("snapshot"),
        command_plan_digest: attempt.identity.command_digest.clone(),
        input_environment_digest: digest("input-environment"),
        execution_records_digest: execution_records_digest(&attempt.execution_records),
        descendants_reaped_to_echild: backend == CompositionSupervisorBackend::LinuxSubreaper,
    };
    attempt.supervisor_receipt = Some(receipt);

    assert!(attempt.is_coherent_successful_terminal());
    attempt.cleanup_disposition = CompositionCleanupDisposition::Deferred;
    assert!(!attempt.is_coherent_successful_terminal());
    attempt.cleanup_disposition = CompositionCleanupDisposition::Cleaned;
    attempt.execution_records[0].stdout.push_str("tampered");
    assert!(!attempt.is_coherent_successful_terminal());
}

#[test]
fn process_group_backend_does_not_claim_linux_echild_proof() {
    let process_identity =
        current_composition_process_identity().expect("current process identity");
    let execution = CompositionExecutionRecord {
        node_id: "successful".into(),
        program: "true".into(),
        args: Vec::new(),
        identity_digest: digest("process-group-execution"),
        spawned: true,
        reused: false,
        passed: true,
        exit_code: Some(0),
        termination_signal: None,
        stdout: String::new(),
        stderr: String::new(),
        output_digest: digest("process-group-output"),
        timed_out: false,
        predecessor_attempt_id: None,
    };
    let identity = identity("process-group-reap-proof");
    let mut attempt = CompositionAttempt {
        schema_version: 3,
        attempt_id: "composition-process-group-reap-proof".into(),
        binding: binding("target-sha", vec!["participant-sha".into()]),
        identity: identity.clone(),
        preconditions: vec![CompositionPrecondition::satisfied("identity-bound")],
        isolated_worktree: "/tmp/composition-process-group-reap-proof".into(),
        text_conflicts: Vec::new(),
        execution_records: vec![execution],
        processes_spawned: 1,
        reuse_decision: ReuseDecision {
            kind: ReuseDecisionKind::Execute,
            reason: "test execution".into(),
            predecessor_attempt_id: None,
        },
        passed: true,
        failure: None,
        execution_outcome: CompositionExecutionOutcome::Passed,
        execution_evidence_complete: true,
        supervisor_receipt: None,
        cleanup_disposition: CompositionCleanupDisposition::Cleaned,
        owner_termination_signal: None,
        cleanup: Some(CompositionCleanup {
            attempted: true,
            removed: true,
            error: None,
        }),
        recorded_at_unix_nanos: 1,
        owner_pid: Some(process_identity.process_id),
        process_observation_schema_version: 1,
        active_execution_node: None,
        active_process_group_id: None,
        active_process_group_identity: None,
        owned_tree_termination_unknown: false,
    };
    for (backend, nonce) in [
        (
            CompositionSupervisorBackend::UnixProcessGroup,
            "unix-process-group",
        ),
        (
            CompositionSupervisorBackend::WindowsProcessGroup,
            "windows-process-group",
        ),
    ] {
        attempt.supervisor_receipt = Some(CompositionSupervisorReceipt {
            schema_version: 1,
            backend,
            attempt_id: attempt.attempt_id.clone(),
            run_nonce: nonce.into(),
            generation: 1,
            owner: process_identity.clone(),
            supervisor: process_identity.clone(),
            linux_boot_id: None,
            environment_digest: digest("environment"),
            runtime_version: attempt.binding.verifier.runtime_version.clone(),
            runtime_digest: attempt.binding.verifier.runtime_digest.clone(),
            repository_id: attempt.binding.repository_id.clone(),
            target_sha: attempt.binding.target_sha.clone(),
            snapshot_digest: digest("snapshot"),
            command_plan_digest: identity.command_digest.clone(),
            input_environment_digest: digest("input-environment"),
            execution_records_digest: execution_records_digest(&attempt.execution_records),
            descendants_reaped_to_echild: false,
        });

        let receipt = attempt
            .supervisor_receipt
            .as_ref()
            .expect("supervisor receipt");
        assert!(!receipt.descendants_reaped_to_echild);
        assert!(
            attempt.is_coherent_successful_terminal(),
            "{backend:?} receipts must retain their platform semantics without claiming Linux ECHILD"
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn composition_subreaper_test_helper() {
    if std::env::var_os("AI_COCKPIT_RUN_COMPOSITION_SUBREAPER_HELPER").is_none() {
        return;
    }
    assert_eq!(
        initialize_composition_supervisor_backend().expect("initialize subreaper"),
        CompositionSupervisorBackend::LinuxSubreaper
    );
    let child = unsafe { libc::fork() };
    assert!(child >= 0, "fork direct verifier-like child");
    if child == 0 {
        let grandchild = unsafe { libc::fork() };
        if grandchild == 0 {
            unsafe {
                libc::close(libc::STDIN_FILENO);
                libc::close(libc::STDOUT_FILENO);
                libc::close(libc::STDERR_FILENO);
                libc::alarm(3);
            }
            loop {
                unsafe { libc::pause() };
            }
        }
        unsafe { libc::_exit(i32::from(grandchild < 0)) };
    }
    let mut status = 0;
    assert_eq!(unsafe { libc::waitpid(child, &mut status, 0) }, child);
    assert!(libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0);

    let own_pid = std::process::id();
    let owned_children = fs::read_dir("/proc")
        .expect("enumerate Linux process table")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let process_id = entry.file_name().to_str()?.parse::<u32>().ok()?;
            let stat = fs::read_to_string(entry.path().join("stat")).ok()?;
            let close = stat.rfind(')')?;
            let parent_id = stat[close + 1..]
                .split_whitespace()
                .nth(1)?
                .parse::<u32>()
                .ok()?;
            (parent_id == own_pid).then_some(process_id)
        })
        .collect::<Vec<_>>();
    assert!(
        !owned_children.is_empty(),
        "the orphaned verifier descendant must be reparented to this subreaper"
    );
    reap_composition_supervisor_descendants().expect("reap all adopted descendants");
    let mut status = 0;
    assert_eq!(unsafe { libc::waitpid(-1, &mut status, libc::WNOHANG) }, -1);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ECHILD)
    );
}

#[cfg(target_os = "linux")]
#[test]
fn composition_supervisor_reaps_orphaned_descendants_to_echild() {
    let output = Command::new(std::env::current_exe().expect("test binary path"))
        .args([
            "--exact",
            "composition_subreaper_test_helper",
            "--nocapture",
        ])
        .env("AI_COCKPIT_RUN_COMPOSITION_SUBREAPER_HELPER", "1")
        .output()
        .expect("run isolated subreaper test helper");
    assert!(
        output.status.success(),
        "subreaper helper failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn failed_attempts_are_append_only_and_exact_identity_controls_reuse() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("failure", "sh", &["-c", "exit 17"])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let first = run_composition(composition.clone()).expect("first attempt");
    std::thread::sleep(std::time::Duration::from_millis(1));
    let second = run_composition(composition.clone()).expect("retry attempt");
    assert_ne!(first.attempt_id, second.attempt_id);
    assert!(attempt_record_path(state.path(), &first.attempt_id).exists());
    assert!(attempt_record_path(state.path(), &second.attempt_id).exists());
    assert_eq!(
        classify_reuse(&first, &composition).kind,
        ReuseDecisionKind::Unknown
    );

    let mut changed = composition;
    changed.identity.interface_digest = digest("changed-interface");
    assert_eq!(
        classify_reuse(&first, &changed).kind,
        ReuseDecisionKind::Unknown
    );
}

#[test]
fn successful_exact_identity_allows_reuse_but_command_change_reexecutes() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("cheap", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let passed = run_composition(composition.clone()).expect("successful attempt");
    assert!(passed.passed);
    assert_eq!(
        classify_reuse(&passed, &composition).kind,
        ReuseDecisionKind::Unknown
    );

    let mut changed = composition;
    changed.commands = vec![command("cheap", "sh", &["-c", "false"])];
    changed.identity.command_digest = composition_commands_digest(&changed.commands);
    assert_eq!(
        classify_reuse(&passed, &changed).kind,
        ReuseDecisionKind::Unknown
    );
}

#[test]
fn repeated_exact_composition_reuses_only_when_inputs_are_observable() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("cheap", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );

    let first = run_composition_with_pure_cache_test_supervisor(composition.clone())
        .expect("first attempt");
    let second =
        run_composition_with_pure_cache_test_supervisor(composition).expect("second attempt");

    assert!(first.passed);
    assert!(second.passed);
    assert_eq!(second.execution_records.len(), 1);
    if cfg!(unix) {
        assert_eq!(second.processes_spawned, 0);
        assert!(!second.execution_records[0].spawned);
        assert!(second.execution_records[0].reused);
        assert_eq!(
            second.execution_records[0]
                .predecessor_attempt_id
                .as_deref(),
            Some(first.attempt_id.as_str())
        );
    } else {
        assert_eq!(second.processes_spawned, 1);
        assert!(second.execution_records[0].spawned);
        assert!(!second.execution_records[0].reused);
        assert_eq!(second.reuse_decision.kind, ReuseDecisionKind::Unknown);
    }
}

#[cfg(unix)]
#[test]
fn admission_change_blocks_reuse_even_when_no_process_would_start() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("reusable", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let first =
        run_composition_with_test_supervisor(composition.clone()).expect("seed reusable receipt");
    assert!(first.passed);

    let admission_check: ProcessAdmissionCheck = std::sync::Arc::new(|_node_id: &str, _accept| {
        Err("coordination_safely_paused:late-request".into())
    });
    let start_gate: ProcessStartGate = std::sync::Arc::new(
        |_node_id: &str, _spawn: &mut dyn FnMut() -> Result<Child, String>| {
            Err("start gate must not run for a reused node".into())
        },
    );
    let blocked = run_composition_with_process_gates(composition, admission_check, start_gate)
        .expect("a denied reuse is preserved as a failed attempt");

    assert!(!blocked.passed);
    assert_eq!(blocked.processes_spawned, 0);
    assert!(blocked.execution_records.is_empty());
    assert!(
        blocked
            .failure
            .as_deref()
            .is_some_and(|failure| failure.contains("coordination_safely_paused")),
        "actual failure={:?}; reuse={:?}; execution records={:?}; cleanup={:?}",
        blocked.failure,
        blocked.reuse_decision,
        blocked.execution_records,
        blocked.cleanup_disposition,
    );
}

#[cfg(unix)]
#[test]
fn pause_write_cannot_commit_between_reuse_admission_and_receipt_acceptance() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("reuse-pause-race");
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("reusable", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let first =
        run_composition_with_test_supervisor(composition.clone()).expect("seed reusable receipt");
    assert!(first.passed);

    let paused = Arc::new(Mutex::new(false));
    let check_paused = Arc::clone(&paused);
    let (checked_tx, checked_rx) = mpsc::channel();
    let (resume_tx, resume_rx) = mpsc::channel();
    let resume_rx = Mutex::new(resume_rx);
    let admission_check: ProcessAdmissionCheck = Arc::new(move |_node_id: &str, accept| {
        let pause_state = check_paused.lock().expect("pause state");
        if *pause_state {
            return Err("coordination_safely_paused:late-request".into());
        }
        checked_tx
            .send(())
            .expect("signal completed admission check");
        resume_rx
            .lock()
            .expect("resume receiver")
            .recv()
            .expect("wait for pause commit");
        accept()
    });
    let start_gate: ProcessStartGate = Arc::new(
        |_node_id: &str, _spawn: &mut dyn FnMut() -> Result<Child, String>| {
            Err("start gate must not run for a reused node".into())
        },
    );

    let runner = std::thread::spawn(move || {
        run_composition_with_process_gates(composition, admission_check, start_gate)
            .expect("race attempt is durably recorded")
    });
    checked_rx
        .recv()
        .expect("reuse admission must be checked before acceptance");
    let pause_writer_state = Arc::clone(&paused);
    let (pause_attempting_tx, pause_attempting_rx) = mpsc::channel();
    let pause_writer = std::thread::spawn(move || {
        pause_attempting_tx
            .send(())
            .expect("signal pause write attempt");
        *pause_writer_state.lock().expect("pause state") = true;
    });
    pause_attempting_rx
        .recv()
        .expect("pause writer starts while admission synchronization is held");
    assert!(matches!(
        paused.try_lock(),
        Err(std::sync::TryLockError::WouldBlock)
    ));
    resume_tx
        .send(())
        .expect("allow the reuse path to continue");
    let attempt = runner.join().expect("composition worker");
    pause_writer.join().expect("pause writer");

    assert!(
        attempt.passed,
        "the cached receipt is accepted before a concurrent pause can commit"
    );
    assert!(attempt.execution_records[0].reused);
    assert!(*paused.lock().expect("pause state"));
}

#[test]
fn failed_attempt_cannot_become_reusable_by_tampering_with_its_pass_flag() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("tampered-attempt-state");
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("must-fail", "false", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );

    let failed = run_composition(composition.clone()).expect("failed attempt is recorded");
    assert!(!failed.passed);
    let attempt_path = attempt_record_path(state.path(), &failed.attempt_id);
    let mut persisted: serde_json::Value =
        serde_json::from_slice(&fs::read(&attempt_path).expect("persisted attempt bytes"))
            .expect("persisted attempt JSON");
    persisted["executionRecords"][0]["passed"] = serde_json::Value::Bool(true);
    fs::write(
        &attempt_path,
        serde_json::to_vec_pretty(&persisted).expect("serialize tampered attempt"),
    )
    .expect("tamper only the pass flag");

    let retry = run_composition(composition).expect("retry executes the required node");

    assert!(!retry.passed);
    assert_eq!(retry.processes_spawned, 1);
    assert!(retry.execution_records[0].spawned);
    assert!(!retry.execution_records[0].reused);
}

#[cfg(unix)]
#[test]
fn inherited_path_cannot_substitute_a_composition_verifier() {
    use std::os::unix::fs::PermissionsExt;

    if std::env::var_os("COMPOSITION_HOSTILE_PATH_CHILD").is_some() {
        let root = PathBuf::from(std::env::var_os("COMPOSITION_HOSTILE_PATH_ROOT").expect("root"));
        let state =
            PathBuf::from(std::env::var_os("COMPOSITION_HOSTILE_PATH_STATE").expect("state"));
        let base = run(&root, &["rev-parse", "refs/heads/main"]);
        let marker =
            PathBuf::from(std::env::var_os("COMPOSITION_FAKE_CARGO_MARKER").expect("marker path"));
        let attempt = run_composition(input(
            &root,
            &state,
            binding(&base, vec![base.clone(), base.clone()]),
            vec![command("required-cargo", "cargo", &["--version"])],
            vec![CompositionPrecondition::satisfied("identity-bound")],
        ))
        .expect("composition records the verifier result");
        assert!(
            !marker.exists(),
            "an inherited PATH entry must not substitute the Contract-required verifier"
        );
        assert!(
            attempt.passed,
            "the Runtime-bound cargo verifier must pass: {attempt:?}"
        );
        assert_eq!(attempt.processes_spawned, 1);

        let mut unbound_override = input(
            &root,
            &state,
            binding(&base, vec![base.clone(), base.clone()]),
            vec![command("override-cargo", "cargo", &["--version"])],
            vec![CompositionPrecondition::satisfied("identity-bound")],
        );
        unbound_override.commands[0].environment.insert(
            "RUSTUP_TOOLCHAIN".into(),
            "missing-runtime-toolchain".into(),
        );
        let rejected = run_composition(unbound_override).expect("reject unbound override");
        assert!(!rejected.passed);
        assert_eq!(rejected.processes_spawned, 0);
        assert!(
            !marker.exists(),
            "an unbound toolchain overlay must be rejected before process start"
        );
        return;
    }

    let root = repository();
    let state = tempdir("hostile-path-state");
    let fake_bin = tempdir("hostile-path-bin");
    let marker = state.path().join("fake-cargo-ran");
    let fake_cargo = fake_bin.path().join("cargo");
    fs::write(
        &fake_cargo,
        "#!/bin/sh\nprintf forged > \"$COMPOSITION_FAKE_CARGO_MARKER\"\nexit 0\n",
    )
    .expect("write fake cargo");
    fs::set_permissions(&fake_cargo, fs::Permissions::from_mode(0o755))
        .expect("make fake cargo executable");

    let mut paths = vec![fake_bin.path().to_path_buf()];
    if let Some(system_path) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&system_path));
    }
    let inherited_path = std::env::join_paths(paths).expect("hostile PATH");
    let child = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "inherited_path_cannot_substitute_a_composition_verifier",
        ])
        .env("COMPOSITION_HOSTILE_PATH_CHILD", "1")
        .env("COMPOSITION_HOSTILE_PATH_ROOT", root.path())
        .env("COMPOSITION_HOSTILE_PATH_STATE", state.path())
        .env("COMPOSITION_FAKE_CARGO_MARKER", &marker)
        .env("PATH", inherited_path)
        .env("RUSTUP_TOOLCHAIN", "missing-runtime-toolchain")
        .env("RUSTFLAGS", "--runtime-must-not-inherit-this")
        .output()
        .expect("run isolated child with hostile inherited PATH");

    assert!(
        child.status.success(),
        "hostile PATH child failed; stdout: {}; stderr: {}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    assert!(!marker.exists(), "fake cargo must never run");
}

#[cfg(unix)]
#[test]
fn read_set_under_a_parent_symlink_is_not_reusable() {
    use std::os::unix::fs::symlink;

    let root = repository();
    let external = tempdir("read-set-symlink-target");
    fs::write(external.path().join("input.txt"), "external bytes\n").expect("external input");
    symlink(external.path(), root.path().join("linked"))
        .expect("symlink parent into external directory");
    run(root.path(), &["add", "linked"]);
    run(root.path(), &["commit", "-qm", "add linked input"]);
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("read-set-symlink-state");
    let mut external_read = command("external-read", "cat", &["linked/input.txt"]);
    external_read.input_paths = vec!["linked/input.txt".into()];
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![external_read],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );

    let first = run_composition(composition.clone()).expect("first composition");
    let second = run_composition(composition).expect("second composition");

    assert!(first.passed);
    assert!(second.passed, "second composition attempt: {second:?}");
    assert_eq!(second.processes_spawned, 1);
    assert!(second.execution_records[0].spawned);
    assert!(!second.execution_records[0].reused);
}

#[cfg(unix)]
#[test]
fn persistence_error_after_worktree_creation_cleans_the_temporary_worktree() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("composition-write-failure-state");
    let backup = state.path().with_file_name(format!(
        "{}-backup",
        state.path().file_name().unwrap().to_string_lossy()
    ));
    let worktrees_before = run(root.path(), &["worktree", "list", "--porcelain"]);
    let mut sabotage = command(
        "sabotage-state-store",
        "sh",
        &[
            "-c",
            r#"for attempt in "$COMPOSITION_STATE_DIR"/composition-*.json; do i=0; while ! grep -Eq '"activeProcessGroupId": [1-9][0-9]*' "$attempt"; do i=$((i + 1)); [ "$i" -lt 400 ] || exit 77; sleep 0.01; done; done; mv "$COMPOSITION_STATE_DIR" "$COMPOSITION_STATE_BACKUP" && touch "$COMPOSITION_STATE_DIR""#,
        ],
    );
    sabotage.environment.insert(
        "COMPOSITION_STATE_DIR".into(),
        state.path().to_string_lossy().into_owned(),
    );
    sabotage.environment.insert(
        "COMPOSITION_STATE_BACKUP".into(),
        backup.to_string_lossy().into_owned(),
    );

    let mut composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![sabotage],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    composition.timeout_seconds = 10;
    let result = run_composition(composition);
    assert!(
        backup.is_dir(),
        "sabotage command should move the state directory"
    );
    assert!(
        state.path().is_file(),
        "sabotage command should replace it with a file"
    );
    let worktrees_after = run(root.path(), &["worktree", "list", "--porcelain"]);
    let root_identity = fs::canonicalize(root.path()).expect("canonical test repository");
    let leaked_worktrees = worktrees_after
        .lines()
        .filter_map(|line| line.strip_prefix("worktree "))
        .filter_map(|worktree| fs::canonicalize(worktree).ok())
        .filter(|worktree| worktree != &root_identity)
        .collect::<Vec<_>>();

    for worktree in &leaked_worktrees {
        assert_eq!(
            worktree.file_name().and_then(|name| name.to_str()),
            Some("composition"),
            "only the uniquely named test composition worktree may be cleaned"
        );
        let parent = worktree.parent().expect("composition parent");
        let temp_root = fs::canonicalize(std::env::temp_dir()).expect("canonical temp root");
        assert_eq!(
            parent.parent(),
            Some(temp_root.as_path()),
            "composition cleanup is limited to a direct child of the temp root"
        );
        assert!(
            parent
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("ai-cockpit-composition-")),
            "composition parent must use the Runtime's unique prefix"
        );
        let _ = Command::new("git")
            .args(["worktree", "remove", "--force"])
            .arg(worktree)
            .current_dir(root.path())
            .status();
        fs::remove_dir_all(parent).expect("remove only the isolated composition parent");
    }
    if state.path().is_file() {
        fs::remove_file(state.path()).expect("remove sabotaging state file");
    }
    if backup.exists() {
        fs::remove_dir_all(&backup).expect("remove isolated state backup");
    }

    assert!(
        result.is_err(),
        "persistence failure should be preserved: {result:?}"
    );
    assert!(
        leaked_worktrees.is_empty(),
        "temporary worktree leaked after a state persistence failure: {leaked_worktrees:?}"
    );
    assert_eq!(worktrees_before, worktrees_after);
}

#[test]
fn retry_does_not_remove_a_worktree_owned_by_a_live_process() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("live-owner-state");
    let mut composition = input(
        root.path(),
        state.path(),
        binding(&base, vec![base.clone(), base.clone()]),
        vec![command("owner", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    composition.timeout_seconds = 30;
    let (spawned_tx, spawned_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    let start_gate: ProcessStartGate = Arc::new(move |_node_id, spawn| {
        let child = spawn()?;
        spawned_tx
            .send(())
            .map_err(|error| format!("signal that the process was spawned: {error}"))?;
        release_rx
            .lock()
            .map_err(|error| format!("lock process release gate: {error}"))?
            .recv_timeout(Duration::from_secs(8))
            .map_err(|error| format!("wait for owner release: {error}"))?;
        Ok(child)
    });
    let admission_check: ProcessAdmissionCheck = Arc::new(|_node_id, accept| accept());
    let root_path = root.path().to_path_buf();
    let state_path = state.path().to_path_buf();
    let worker = std::thread::spawn(move || {
        run_composition_with_process_gates(composition, admission_check, start_gate)
    });
    spawned_rx
        .recv_timeout(Duration::from_secs(8))
        .expect("owner process should reach the deterministic start gate");

    let (attempt_path, worktree_path) = fs::read_dir(&state_path)
        .expect("state directory")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .find_map(|path| {
            let value: serde_json::Value = serde_json::from_slice(&fs::read(&path).ok()?).ok()?;
            let worktree = value["isolatedWorktree"].as_str()?.to_owned();
            (!worktree.is_empty()).then_some((path, PathBuf::from(worktree)))
        })
        .expect("live owner attempt should record its isolated worktree");
    let registered = run(&root_path, &["worktree", "list", "--porcelain"])
        .lines()
        .any(|line| {
            line.strip_prefix("worktree ").is_some_and(|path| {
                fs::canonicalize(path).ok() == fs::canonicalize(&worktree_path).ok()
            })
        });
    assert!(registered, "live owner's worktree should be registered");

    let retry = input(
        root.path(),
        state.path(),
        binding(&base, vec![base.clone(), base.clone()]),
        vec![command("retry", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let error = run_composition(retry).expect_err("a live owner must block recovery");
    assert!(matches!(
        error,
        CompositionError::ActiveAttempt { owner_pid, .. } if owner_pid == std::process::id()
    ));
    assert!(worktree_path.is_dir(), "live owner's worktree was removed");
    assert!(attempt_path.is_file(), "live owner's attempt was replaced");

    release_tx
        .send(())
        .expect("release owner process after retry assertions");
    let finished = worker
        .join()
        .expect("owner thread")
        .expect("composition result");
    assert!(finished.passed, "owner attempt failed: {finished:?}");
    assert!(
        !worktree_path.exists(),
        "owner worktree should clean after completion"
    );
}

#[cfg(unix)]
#[test]
fn retry_reconciles_an_interrupted_owner_after_process_exit() {
    if std::env::var_os("COMPOSITION_CRASH_CHILD").is_some() {
        let root = PathBuf::from(std::env::var_os("COMPOSITION_CRASH_ROOT").expect("root"));
        let state = PathBuf::from(std::env::var_os("COMPOSITION_CRASH_STATE").expect("state"));
        let _owner_interruption_guard = cockpit_verification::OwnerInterruptionGuard::install()
            .expect("install owner interruption handler");
        let base = run(&root, &["rev-parse", "refs/heads/main"]);
        let state_arg = state.to_string_lossy().into_owned();
        let attempt = run_composition(input(
            &root,
            &state,
            binding(&base, vec![base.clone(), base.clone()]),
            vec![command(
                "terminate-owner",
                "sh",
                &[
                    "-c",
                    "while ! /usr/bin/grep -Eq '\"activeProcessGroupId\"[[:space:]]*:[[:space:]]*[0-9]+' \"$1\"/*.json; do /bin/sleep 0.01; done; /usr/bin/python3 -c 'import os,sys,time; error=None\ntry:\n held=open(os.path.join(os.getcwd(),\"README.md\"),\"rb\"); os.setsid(); os.chdir(\"/\")\nexcept Exception as exc:\n error=repr(exc); os.chdir(\"/\")\nopen(sys.argv[1]+\".error\",\"w\").write(error or \"none\"); open(sys.argv[1],\"w\").write(str(os.getpid())); time.sleep(60)' \"$1/escaped.pid\" >/dev/null 2>&1 & while [ ! -s \"$1/escaped.pid\" ]; do /bin/sleep 0.01; done; kill -INT \"$PPID\"",
                    "sh",
                    &state_arg,
                ],
            )],
            vec![CompositionPrecondition::satisfied("identity-bound")],
        ))
        .expect("owner-interrupted composition attempt");
        assert_eq!(attempt.owner_termination_signal, Some(2));
        assert!(!attempt.passed);
        assert!(attempt.cleanup.is_none());
        return;
    }

    #[cfg(target_os = "linux")]
    if std::env::var_os("COMPOSITION_CRASH_TEST_HELPER").is_none() {
        let mut original_subreaper_state = 0;
        assert_eq!(
            unsafe {
                libc::prctl(
                    libc::PR_GET_CHILD_SUBREAPER,
                    &mut original_subreaper_state as *mut libc::c_int,
                )
            },
            0,
            "read test harness child-subreaper state before isolated helper"
        );
        let mut helper = Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "retry_reconciles_an_interrupted_owner_after_process_exit",
                "--nocapture",
            ])
            .env("COMPOSITION_CRASH_TEST_HELPER", "1")
            .spawn()
            .expect("spawn isolated interrupted-owner test helper");
        let deadline = Instant::now() + Duration::from_secs(60);
        let status = loop {
            if let Some(status) = helper.try_wait().expect("check isolated test helper") {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = helper.kill();
                let _ = helper.wait();
                panic!("isolated interrupted-owner test helper did not terminate");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(
            status.success(),
            "isolated interrupted-owner test helper failed: {status}"
        );
        let mut final_subreaper_state = 0;
        assert_eq!(
            unsafe {
                libc::prctl(
                    libc::PR_GET_CHILD_SUBREAPER,
                    &mut final_subreaper_state as *mut libc::c_int,
                )
            },
            0,
            "read test harness child-subreaper state after isolated helper"
        );
        assert_eq!(
            final_subreaper_state, original_subreaper_state,
            "isolated helper must not change the test harness subreaper state"
        );
        return;
    }

    let root = repository();
    #[cfg(target_os = "linux")]
    let _subreaper = ChildSubreaperGuard::enable();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("crash-recovery-state");
    let mut child = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "retry_reconciles_an_interrupted_owner_after_process_exit",
        ])
        .env("COMPOSITION_CRASH_CHILD", "1")
        .env_remove("COMPOSITION_CRASH_TEST_HELPER")
        .env("COMPOSITION_CRASH_ROOT", root.path())
        .env("COMPOSITION_CRASH_STATE", state.path())
        .spawn()
        .expect("spawn killable composition owner");
    let owner_pid = child.id();

    let deadline = Instant::now() + Duration::from_secs(8);
    let status = loop {
        if let Some(status) = child.try_wait().expect("check owner process") {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "composition owner did not terminate"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(
        status.success(),
        "the owner should persist its SIGINT attempt before exiting"
    );

    let attempt_path = fs::read_dir(state.path())
        .expect("state directory")
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .expect("durable interrupted attempt");
    let mut interrupted: serde_json::Value =
        serde_json::from_slice(&fs::read(&attempt_path).expect("attempt bytes"))
            .expect("attempt JSON");
    assert_eq!(interrupted["ownerPid"], owner_pid);
    assert_eq!(interrupted["ownerTerminationSignal"], 2);
    assert_eq!(
        interrupted["executionRecords"][0]["nodeId"], "terminate-owner",
        "completed interrupted verifier identity must remain durable"
    );
    assert!(interrupted["activeExecutionNode"].is_null());
    assert!(interrupted["activeProcessGroupId"].is_null());
    let worktree = PathBuf::from(
        interrupted["isolatedWorktree"]
            .as_str()
            .expect("durable worktree path"),
    );
    assert!(!worktree.as_os_str().is_empty());
    assert!(
        worktree.is_dir(),
        "interrupted worktree should remain for recovery"
    );

    let observed_attempt = interrupted.clone();
    let interrupted_object = interrupted
        .as_object_mut()
        .expect("interrupted attempt object");
    interrupted_object.remove("processObservationSchemaVersion");
    interrupted_object.remove("activeExecutionNode");
    interrupted_object.remove("activeProcessGroupId");
    fs::write(
        &attempt_path,
        serde_json::to_vec_pretty(&interrupted).expect("serialize legacy attempt"),
    )
    .expect("simulate pre-observer interrupted attempt");
    let legacy_retry = run_composition(input(
        root.path(),
        state.path(),
        binding(&base, vec![base.clone(), base.clone()]),
        Vec::new(),
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ));
    assert!(
        matches!(
            legacy_retry,
            Err(CompositionError::UnknownAttemptOwner { .. }
                | CompositionError::ActiveVerifierDescendant { .. })
        ),
        "legacy interrupted attempts without process observations must fail closed"
    );
    assert!(
        worktree.is_dir(),
        "unknown legacy process state must preserve its worktree"
    );
    fs::write(
        &attempt_path,
        serde_json::to_vec_pretty(&observed_attempt).expect("restore observed attempt"),
    )
    .expect("restore process observations");

    let escaped_process_id = fs::read_to_string(state.path().join("escaped.pid"))
        .expect("detached verifier descendant pid")
        .trim()
        .parse::<u32>()
        .expect("detached verifier descendant process id");
    assert_eq!(
        fs::read_to_string(state.path().join("escaped.pid.error"))
            .expect("detached verifier setup result"),
        "none",
        "detached verifier must open a worktree file and leave its working directory"
    );
    struct DetachedProcessGuard {
        process_id: u32,
        reaped: bool,
    }
    impl Drop for DetachedProcessGuard {
        fn drop(&mut self) {
            if self.reaped {
                return;
            }
            unsafe {
                libc::kill(self.process_id as libc::pid_t, libc::SIGKILL);
            }
            #[cfg(target_os = "linux")]
            {
                let deadline = Instant::now() + Duration::from_secs(5);
                loop {
                    let mut status = 0;
                    let waited = unsafe {
                        libc::waitpid(self.process_id as libc::pid_t, &mut status, libc::WNOHANG)
                    };
                    if waited == self.process_id as libc::pid_t
                        || (waited < 0
                            && std::io::Error::last_os_error().raw_os_error() == Some(libc::ECHILD))
                    {
                        break;
                    }
                    if Instant::now() >= deadline {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        }
    }
    let mut escaped_process_guard = DetachedProcessGuard {
        process_id: escaped_process_id,
        reaped: false,
    };
    let blocked = run_composition(input(
        root.path(),
        state.path(),
        binding(&base, vec![base.clone(), base.clone()]),
        Vec::new(),
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ));
    assert!(
        matches!(
            blocked,
            Err(CompositionError::ActiveVerifierDescendant { process_id, .. })
                if process_id == escaped_process_id
        ),
        "a verifier that escaped its original process group must still protect the worktree: {blocked:?}"
    );
    assert!(
        worktree.is_dir(),
        "live detached verifier lost its worktree"
    );
    unsafe {
        libc::kill(escaped_process_id as libc::pid_t, libc::SIGKILL);
    }
    #[cfg(target_os = "linux")]
    let zombie_identity = {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let stat = fs::read_to_string(format!("/proc/{escaped_process_id}/stat"))
                .expect("detached verifier process stat");
            let close = stat.rfind(')').expect("proc stat command terminator");
            let fields = stat[close + 2..].split_whitespace().collect::<Vec<_>>();
            let state = fields[0].chars().next().expect("process state");
            let parent_pid = fields[1].parse::<u32>().expect("parent pid");
            let process_group_id = fields[2].parse::<u32>().expect("process group id");
            let session_id = fields[3].parse::<u32>().expect("session id");
            if state == 'Z' {
                break (state, parent_pid, process_group_id, session_id);
            }
            assert!(
                Instant::now() < deadline,
                "detached verifier did not become a zombie after SIGKILL"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    };
    #[cfg(target_os = "linux")]
    {
        assert_eq!(zombie_identity.1, std::process::id());
        assert_eq!(zombie_identity.2, escaped_process_id);
        assert_eq!(zombie_identity.3, escaped_process_id);
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        assert_eq!(
            unsafe {
                libc::waitid(
                    libc::P_PID,
                    escaped_process_id as libc::id_t,
                    &mut info,
                    libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
                )
            },
            0,
            "waitid WNOWAIT observes the detached verifier zombie without reaping it"
        );
        assert_eq!(
            unsafe { libc::kill(escaped_process_id as libc::pid_t, 0) },
            0,
            "kill(pid, 0) still sees the unreaped detached verifier zombie"
        );
        eprintln!(
            "observed detached verifier zombie: pid={} state={:?} ppid={} pgid={} sid={}; waitid(WNOWAIT)=0; kill(pid, 0)=0",
            escaped_process_id,
            zombie_identity.0,
            zombie_identity.1,
            zombie_identity.2,
            zombie_identity.3
        );
    }
    #[cfg(not(target_os = "linux"))]
    let escaped_process_details = "process state unavailable";
    #[cfg(target_os = "linux")]
    {
        let mut status = 0;
        let waited = unsafe { libc::waitpid(escaped_process_id as libc::pid_t, &mut status, 0) };
        assert_eq!(
            waited, escaped_process_id as libc::pid_t,
            "the isolated subreaper must explicitly reap its detached zombie"
        );
        escaped_process_guard.reaped = true;
        assert_eq!(
            unsafe { libc::kill(escaped_process_id as libc::pid_t, 0) },
            -1,
            "the detached verifier PID must be absent after waitpid reaps it"
        );
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }
    #[cfg(not(target_os = "linux"))]
    {
        let escaped_deadline = Instant::now() + Duration::from_secs(5);
        while unsafe { libc::kill(escaped_process_id as libc::pid_t, 0) } == 0 {
            assert!(
                Instant::now() < escaped_deadline,
                "detached verifier pid={escaped_process_id} {escaped_process_details} remained visible after SIGKILL"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        escaped_process_guard.reaped = true;
    }

    let retry = run_composition(input(
        root.path(),
        state.path(),
        binding(&base, vec![base.clone(), base.clone()]),
        vec![command("recovered", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("retry reconciles the dead owner");

    assert!(retry.passed, "recovered attempt failed: {retry:?}");
    assert_eq!(retry.processes_spawned, 1);
    interrupted = serde_json::from_slice(&fs::read(&attempt_path).expect("reconciled attempt"))
        .expect("reconciled attempt JSON");
    assert_eq!(
        interrupted["failure"],
        "interrupted_owner_terminated:signal=2"
    );
    assert_eq!(interrupted["ownerTerminationSignal"], 2);
    assert_eq!(interrupted["cleanup"]["removed"], true);
    assert!(
        !worktree.exists(),
        "abandoned worktree must be removed on retry"
    );
    assert!(
        !run(root.path(), &["worktree", "list", "--porcelain"])
            .lines()
            .any(|line| line.strip_prefix("worktree ").is_some_and(|path| {
                fs::canonicalize(path).ok() == fs::canonicalize(&worktree).ok()
            }))
    );
    assert_eq!(
        interrupted["ownerTerminationSignal"], 2,
        "the durable attempt must retain the owner's observed SIGINT"
    );
}

#[cfg(unix)]
#[test]
fn retry_preserves_worktree_while_orphan_verifier_process_group_is_alive() {
    if std::env::var_os("COMPOSITION_ORPHAN_CHILD").is_some() {
        let root = PathBuf::from(std::env::var_os("COMPOSITION_ORPHAN_ROOT").expect("root"));
        let state = PathBuf::from(std::env::var_os("COMPOSITION_ORPHAN_STATE").expect("state"));
        let pid_file =
            PathBuf::from(std::env::var_os("COMPOSITION_ORPHAN_PID_FILE").expect("pid file"));
        let base = run(&root, &["rev-parse", "refs/heads/main"]);
        let state_arg = state.to_string_lossy().into_owned();
        let pid_file_arg = pid_file.to_string_lossy().into_owned();
        let script = "printf '%s\\n' \"$$\" > \"$2\"; while ! /usr/bin/grep -Eq '\"activeProcessGroupId\"[[:space:]]*:[[:space:]]*[0-9]+' \"$1\"/*.json; do /bin/sleep 0.01; done; kill -9 \"$PPID\"; exec /bin/sleep 60";
        let _ = run_composition(input(
            &root,
            &state,
            binding(&base, vec![base.clone(), base.clone()]),
            vec![command(
                "orphan-verifier",
                "sh",
                &["-c", script, "sh", &state_arg, &pid_file_arg],
            )],
            vec![CompositionPrecondition::satisfied("identity-bound")],
        ));
        panic!("the verifier command should terminate its composition owner");
    }

    #[cfg(target_os = "linux")]
    if std::env::var_os("COMPOSITION_ORPHAN_TEST_HELPER").is_none() {
        let mut original_subreaper_state = 0;
        assert_eq!(
            unsafe {
                libc::prctl(
                    libc::PR_GET_CHILD_SUBREAPER,
                    &mut original_subreaper_state as *mut libc::c_int,
                )
            },
            0,
            "read test harness child-subreaper state before isolated helper"
        );
        let mut helper = Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "retry_preserves_worktree_while_orphan_verifier_process_group_is_alive",
            ])
            .env("COMPOSITION_ORPHAN_TEST_HELPER", "1")
            .env("AI_COCKPIT_COMPOSITION_RECONCILE_TRACE", "1")
            .spawn()
            .expect("spawn isolated subreaper test helper");
        let deadline = Instant::now() + Duration::from_secs(60);
        let status = loop {
            if let Some(status) = helper.try_wait().expect("check isolated test helper") {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = helper.kill();
                let _ = helper.wait();
                panic!("isolated subreaper test helper did not terminate");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(
            status.success(),
            "isolated subreaper test helper failed: {status}"
        );
        let mut final_subreaper_state = 0;
        assert_eq!(
            unsafe {
                libc::prctl(
                    libc::PR_GET_CHILD_SUBREAPER,
                    &mut final_subreaper_state as *mut libc::c_int,
                )
            },
            0,
            "read test harness child-subreaper state after isolated helper"
        );
        assert_eq!(
            final_subreaper_state, original_subreaper_state,
            "isolated helper must not change the test harness subreaper state"
        );
        return;
    }

    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    #[cfg(target_os = "linux")]
    let _subreaper = ChildSubreaperGuard::enable();
    let state = tempdir("orphan-verifier-state");
    let pid_file = state.path().join("orphan-verifier.pid");
    let mut child = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "retry_preserves_worktree_while_orphan_verifier_process_group_is_alive",
        ])
        .env("COMPOSITION_ORPHAN_CHILD", "1")
        .env_remove("COMPOSITION_ORPHAN_TEST_HELPER")
        .env("COMPOSITION_ORPHAN_ROOT", root.path())
        .env("COMPOSITION_ORPHAN_STATE", state.path())
        .env("COMPOSITION_ORPHAN_PID_FILE", &pid_file)
        .spawn()
        .expect("spawn killable composition owner");
    let owner_pid = child.id();
    let deadline = Instant::now() + Duration::from_secs(20);
    let status = loop {
        if let Some(status) = child.try_wait().expect("check owner process") {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "composition owner did not terminate"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(
        !status.success(),
        "verifier command should terminate its owner"
    );

    let attempt_path = fs::read_dir(state.path())
        .expect("state directory")
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .expect("durable interrupted attempt");
    let interrupted: serde_json::Value =
        serde_json::from_slice(&fs::read(&attempt_path).expect("attempt bytes"))
            .expect("attempt JSON");
    assert_eq!(interrupted["ownerPid"], owner_pid);
    let process_group_id = interrupted["activeProcessGroupId"]
        .as_u64()
        .expect("durable active verifier process group") as u32;
    assert_eq!(
        fs::read_to_string(&pid_file)
            .expect("verifier pid file")
            .trim(),
        process_group_id.to_string()
    );
    let worktree = PathBuf::from(
        interrupted["isolatedWorktree"]
            .as_str()
            .expect("durable worktree path"),
    );
    let canonical_worktree = fs::canonicalize(&worktree).expect("canonical interrupted worktree");
    let worktree_registration = format!("worktree {}", canonical_worktree.display());
    assert!(
        worktree.is_dir(),
        "interrupted worktree must remain recoverable"
    );
    assert!(
        run(root.path(), &["worktree", "list", "--porcelain"])
            .lines()
            .any(|line| line == worktree_registration),
        "live owner's original linked worktree must remain registered"
    );

    struct ProcessGroupGuard(Option<u32>);
    impl ProcessGroupGuard {
        fn disarm(&mut self) {
            self.0 = None;
        }
    }
    impl Drop for ProcessGroupGuard {
        fn drop(&mut self) {
            let Some(process_group_id) = self.0.take() else {
                return;
            };
            unsafe {
                libc::kill(-(process_group_id as libc::pid_t), libc::SIGKILL);
            }
            #[cfg(target_os = "linux")]
            {
                let deadline = Instant::now() + Duration::from_secs(5);
                loop {
                    let mut status = 0;
                    let waited = unsafe {
                        libc::waitpid(
                            -(process_group_id as libc::pid_t),
                            &mut status,
                            libc::WNOHANG,
                        )
                    };
                    if waited > 0 {
                        continue;
                    }
                    if waited < 0
                        && std::io::Error::last_os_error().raw_os_error() == Some(libc::ECHILD)
                    {
                        break;
                    }
                    if Instant::now() >= deadline {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        }
    }
    let mut process_group_guard = ProcessGroupGuard(Some(process_group_id));
    let retry_input = input(
        root.path(),
        state.path(),
        binding(&base, vec![base.clone(), base.clone()]),
        Vec::new(),
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let blocked = run_composition(retry_input.clone());
    #[cfg(target_os = "linux")]
    let proc_observation = linux_process_group_proc_observation(process_group_id);
    assert!(
        matches!(
            &blocked,
            Err(CompositionError::ActiveVerifierProcessGroup {
                process_group_id: active_group,
                ..
            }) if *active_group == process_group_id
        ),
        "expected ActiveVerifierProcessGroup for {process_group_id}; actual result: {blocked:?}; \
        same-time /proc observation: {}",
        {
            #[cfg(target_os = "linux")]
            {
                proc_observation.as_str()
            }
            #[cfg(not(target_os = "linux"))]
            {
                "unavailable on this platform"
            }
        }
    );
    assert!(
        worktree.is_dir(),
        "live verifier group worktree must be preserved"
    );
    assert!(
        run(root.path(), &["worktree", "list", "--porcelain"])
            .lines()
            .any(|line| line == worktree_registration),
        "live verifier group must preserve its worktree registration"
    );

    let mut unknown_attempt = interrupted.clone();
    unknown_attempt["activeProcessGroupIdentity"] = serde_json::Value::Null;
    fs::write(
        &attempt_path,
        serde_json::to_vec_pretty(&unknown_attempt).expect("serialize unknown attempt"),
    )
    .expect("persist missing-identity unknown fixture");
    let unknown = run_composition(retry_input.clone())
        .expect_err("missing leader identity must preserve the worktree as unknown");
    assert!(
        matches!(unknown, CompositionError::UnknownAttemptOwner { .. }),
        "missing leader identity must fail closed as unknown: {unknown:?}"
    );
    let still_unknown: serde_json::Value =
        serde_json::from_slice(&fs::read(&attempt_path).expect("unknown attempt remains durable"))
            .expect("unknown attempt JSON");
    assert!(still_unknown["cleanup"].is_null());
    assert!(
        worktree.is_dir(),
        "unknown verifier group must preserve worktree"
    );
    assert!(
        run(root.path(), &["worktree", "list", "--porcelain"])
            .lines()
            .any(|line| line == worktree_registration),
        "unknown verifier group must preserve worktree registration"
    );
    fs::write(
        &attempt_path,
        serde_json::to_vec_pretty(&interrupted).expect("restore complete attempt identity"),
    )
    .expect("restore complete process-group identity");

    unsafe {
        libc::kill(-(process_group_id as libc::pid_t), libc::SIGKILL);
    }
    #[cfg(target_os = "linux")]
    {
        let zombie_deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
            let wait_result = unsafe {
                libc::waitid(
                    libc::P_PID,
                    process_group_id as libc::id_t,
                    &mut info,
                    libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
                )
            };
            assert_eq!(wait_result, 0, "waitid WNOWAIT observes verifier exit");
            let stat = fs::read_to_string(format!("/proc/{process_group_id}/stat"))
                .expect("retained zombie process stat");
            let close = stat.rfind(')').expect("proc stat command terminator");
            if stat[close + 1..].split_whitespace().next() == Some("Z") {
                break;
            }
            assert!(
                Instant::now() < zombie_deadline,
                "verifier process group leader did not become a retained zombie"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        assert_eq!(
            unsafe {
                libc::waitid(
                    libc::P_PID,
                    process_group_id as libc::id_t,
                    &mut info,
                    libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
                )
            },
            0,
            "waitid WNOWAIT observes the retained zombie without reaping it"
        );
    }
    #[cfg(not(target_os = "linux"))]
    {
        let group_deadline = Instant::now() + Duration::from_secs(5);
        while unsafe { libc::kill(-(process_group_id as libc::pid_t), 0) } == 0 {
            assert!(
                Instant::now() < group_deadline,
                "verifier process group did not exit"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    let _ = run_composition(retry_input).expect("retry reconciles after verifier exit");
    let reconciled: serde_json::Value =
        serde_json::from_slice(&fs::read(&attempt_path).expect("reconciled attempt"))
            .expect("reconciled attempt JSON");
    assert_eq!(reconciled["cleanup"]["removed"], true);
    assert!(
        !worktree.exists(),
        "dead verifier worktree should be cleaned on retry"
    );
    assert!(
        !run(root.path(), &["worktree", "list", "--porcelain"])
            .lines()
            .any(|line| line == worktree_registration),
        "dead verifier group cleanup removes the worktree registration"
    );
    #[cfg(target_os = "linux")]
    {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let mut status = 0;
            let waited = unsafe {
                libc::waitpid(
                    -(process_group_id as libc::pid_t),
                    &mut status,
                    libc::WNOHANG,
                )
            };
            if waited > 0 {
                continue;
            }
            if waited == 0 {
                assert!(
                    Instant::now() < deadline,
                    "verifier process group children were not fully reaped"
                );
                std::thread::sleep(Duration::from_millis(5));
                continue;
            }
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ECHILD) {
                break;
            }
            if error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            panic!("failed to reap verifier process group children: {error}");
        }
    }
    process_group_guard.disarm();
}

#[cfg(target_os = "linux")]
struct ChildSubreaperGuard(libc::c_int);

#[cfg(target_os = "linux")]
impl ChildSubreaperGuard {
    fn enable() -> Self {
        let mut previous = 0;
        assert_eq!(
            unsafe {
                libc::prctl(
                    libc::PR_GET_CHILD_SUBREAPER,
                    &mut previous as *mut libc::c_int,
                )
            },
            0,
            "read prior child-subreaper state"
        );
        assert_eq!(
            unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1) },
            0,
            "make the fixture its descendants' subreaper"
        );
        Self(previous)
    }
}

#[cfg(target_os = "linux")]
impl Drop for ChildSubreaperGuard {
    fn drop(&mut self) {
        unsafe {
            libc::prctl(libc::PR_SET_CHILD_SUBREAPER, self.0);
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn retry_reconciles_zombie_only_verifier_process_group() {
    if std::env::var_os("COMPOSITION_ZOMBIE_GROUP_HELPER").is_some() {
        return;
    }

    fn proc_identity(pid: u32) -> (char, u64, u32, u32) {
        let stat = fs::read_to_string(format!("/proc/{pid}/stat")).expect("process stat");
        let close = stat.rfind(')').expect("stat command terminator");
        let fields = stat[close + 2..].split_whitespace().collect::<Vec<_>>();
        (
            fields[0].chars().next().expect("process state"),
            fields[19].parse().expect("process starttime"),
            fields[2].parse().expect("process group id"),
            fields[3].parse().expect("session id"),
        )
    }

    struct ReapChild(Child);
    impl Drop for ReapChild {
        fn drop(&mut self) {
            let _ = self.0.wait();
        }
    }

    let mut helper = Command::new(std::env::current_exe().expect("test executable"));
    helper
        .args([
            "--exact",
            "retry_reconciles_zombie_only_verifier_process_group",
        ])
        .env("COMPOSITION_ZOMBIE_GROUP_HELPER", "1");
    use std::os::unix::process::CommandExt;
    helper.process_group(0);
    let child = ReapChild(helper.spawn().expect("spawn process-group leader"));
    let leader_pid = child.0.id();
    let deadline = Instant::now() + Duration::from_secs(5);
    let leader_identity = loop {
        let (state, start_time_ticks, process_group_id, session_id) = proc_identity(leader_pid);
        if state == 'Z' {
            break (start_time_ticks, process_group_id, session_id);
        }
        assert!(
            Instant::now() < deadline,
            "helper process did not become a zombie"
        );
        std::thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(leader_identity.1, leader_pid);
    let mut zombie_info: libc::siginfo_t = unsafe { std::mem::zeroed() };
    assert_eq!(
        unsafe {
            libc::waitid(
                libc::P_PID,
                leader_pid as libc::id_t,
                &mut zombie_info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        },
        0,
        "waitid WNOWAIT observes the direct child zombie without reaping it"
    );
    assert_eq!(
        unsafe { libc::kill(-(leader_pid as libc::pid_t), 0) },
        0,
        "Linux reports a zombie-only process group as present"
    );

    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let mut exited_owner = Command::new("true")
        .spawn()
        .expect("spawn owner PID fixture");
    let owner_pid = exited_owner.id();
    assert!(
        exited_owner
            .wait()
            .expect("reap owner PID fixture")
            .success()
    );
    let owner_proc = PathBuf::from(format!("/proc/{owner_pid}"));
    assert!(
        !owner_proc.exists(),
        "owner PID fixture must be reaped before retry"
    );

    let state = tempdir("zombie-only-verifier-state");
    let worktree_parent = std::env::temp_dir().join(format!(
        "ai-cockpit-composition-{owner_pid}-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after UNIX epoch")
            .as_nanos(),
        NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&worktree_parent).expect("create owned worktree parent");
    let _worktree_parent = TempDir(worktree_parent.clone());
    let worktree = worktree_parent.join("composition");
    run(
        root.path(),
        &[
            "worktree",
            "add",
            "--detach",
            worktree.to_str().expect("UTF-8 worktree path"),
            &base,
        ],
    );
    let canonical_worktree = fs::canonicalize(&worktree).expect("canonical linked worktree");
    let worktree_registration = format!("worktree {}", canonical_worktree.display());
    assert!(
        run(root.path(), &["worktree", "list", "--porcelain"])
            .lines()
            .any(|line| line == worktree_registration),
        "interrupted attempt fixture must start with an owned linked worktree"
    );

    let attempt_binding = binding(&base, vec![base.clone(), base.clone()]);
    let attempt_id = "interrupted-zombie-only-verifier";
    let preconditions = vec![CompositionPrecondition::satisfied("identity-bound")];
    let interrupted = ::serde_json::json!({
        "schemaVersion": 2,
        "attemptId": attempt_id,
        "binding": attempt_binding,
        "identity": identity("zombie-only-fixture"),
        "preconditions": preconditions,
        "isolatedWorktree": worktree.to_string_lossy(),
        "textConflicts": [],
        "executionRecords": [],
        "processesSpawned": 1,
        "reuseDecision": {
            "kind": "execute",
            "reason": "interrupted fixture",
            "predecessorAttemptId": null
        },
        "passed": false,
        "failure": "in_progress",
        "ownerTerminationSignal": null,
        "cleanup": null,
        "recordedAtUnixNanos": 1,
        "ownerPid": owner_pid,
        "processObservationSchemaVersion": 1,
        "activeExecutionNode": "orphan-verifier",
        "activeProcessGroupId": leader_pid,
        "activeProcessGroupIdentity": {
            "leaderPid": leader_pid,
            "leaderStartTimeTicks": leader_identity.0,
            "processGroupId": leader_identity.1,
            "sessionId": leader_identity.2
        }
    });
    fs::write(
        state.path().join(format!("{attempt_id}.json")),
        serde_json::to_vec_pretty(&interrupted).expect("serialize interrupted attempt"),
    )
    .expect("write interrupted attempt");

    let retry = input(
        root.path(),
        state.path(),
        binding(&base, vec![base.clone(), base.clone()]),
        vec![command("retry", "sh", &["-c", "true"])],
        preconditions,
    );
    assert!(
        !owner_proc.exists(),
        "owner PID fixture remains absent immediately before reconciliation"
    );
    let recovered = run_composition_with_test_supervisor(retry).unwrap_or_else(|error| {
        panic!("zombie-only verifier group should be reconciled safely: {error}")
    });
    let target_tree = run(
        root.path(),
        &["rev-parse", "--verify", &format!("{base}^{{tree}}")],
    );
    let receipt = recovered
        .supervisor_receipt
        .as_ref()
        .expect("fresh retry needs a real supervisor receipt");

    let interrupted_path = attempt_record_path(state.path(), attempt_id);
    let reconciled: serde_json::Value = serde_json::from_slice(
        &fs::read(&interrupted_path).expect("reconciled interrupted attempt"),
    )
    .expect("reconciled attempt JSON");
    assert_eq!(reconciled["failure"], "interrupted_owner_terminated");
    assert_eq!(reconciled["schemaVersion"], 2);
    assert_eq!(reconciled["ownerPid"], owner_pid);
    assert_eq!(reconciled["cleanup"]["attempted"], true);
    assert_eq!(reconciled["cleanup"]["removed"], true);
    assert_eq!(
        reconciled["isolatedWorktree"],
        worktree.to_string_lossy().as_ref()
    );
    assert!(
        !worktree.exists(),
        "reconciliation removes the old worktree directory"
    );
    assert!(
        !run(root.path(), &["worktree", "list", "--porcelain"])
            .lines()
            .any(|line| line == worktree_registration),
        "reconciliation removes the old worktree registration"
    );
    assert_eq!(reconciled["activeProcessGroupId"], serde_json::Value::Null);
    assert_eq!(
        reconciled["activeProcessGroupIdentity"],
        serde_json::Value::Null
    );

    assert_eq!(
        receipt.snapshot_digest,
        Digest::sha256_bytes(target_tree.as_bytes()),
        "supervisor receipt must bind the observed target tree"
    );
    assert_eq!(
        recovered.execution_outcome,
        CompositionExecutionOutcome::Passed
    );
    assert!(recovered.execution_evidence_complete);
    assert_eq!(recovered.processes_spawned, 1);
    assert_eq!(recovered.execution_records.len(), 1);
    assert_eq!(recovered.execution_records[0].node_id, "retry");
    assert!(recovered.execution_records[0].spawned);
    assert!(!recovered.execution_records[0].reused);
    assert!(recovered.execution_records[0].passed);
    assert_eq!(recovered.execution_records[0].exit_code, Some(0));
    assert_eq!(recovered.binding.target_sha, base);
    assert!(recovered.text_conflicts.is_empty());
    assert!(recovered.preconditions.iter().all(|item| item.satisfied));
    assert_eq!(receipt.schema_version, 1);
    assert_eq!(
        receipt.backend,
        CompositionSupervisorBackend::LinuxSubreaper
    );
    assert_eq!(receipt.attempt_id, recovered.attempt_id);
    assert!(!receipt.run_nonce.is_empty());
    assert_eq!(receipt.generation, 1);
    assert_eq!(receipt.repository_id, recovered.binding.repository_id);
    assert_eq!(receipt.target_sha, recovered.binding.target_sha);
    assert_eq!(
        receipt.runtime_version,
        recovered.binding.verifier.runtime_version
    );
    assert_eq!(
        receipt.runtime_digest,
        recovered.binding.verifier.runtime_digest
    );
    assert_eq!(
        receipt.command_plan_digest,
        recovered.identity.command_digest
    );
    assert_eq!(receipt.owner, receipt.supervisor);
    assert!(receipt.owner.process_id > 0);
    assert!(receipt.owner.start_time_ticks.is_some());
    assert!(receipt.supervisor.process_group_id.is_some());
    assert!(receipt.supervisor.session_id.is_some());
    assert_eq!(recovered.owner_pid, Some(receipt.supervisor.process_id));
    assert!(receipt.descendants_reaped_to_echild);
    assert!(!recovered.owned_tree_termination_unknown);
    let durable: CompositionAttempt = serde_json::from_slice(
        &fs::read(attempt_record_path(state.path(), &recovered.attempt_id))
            .expect("durable fresh retry attempt"),
    )
    .expect("decode fresh retry attempt");
    assert_eq!(durable.supervisor_receipt, recovered.supervisor_receipt);

    match recovered.cleanup_disposition {
        CompositionCleanupDisposition::Cleaned => {
            assert!(recovered.passed, "retry failed: {recovered:?}");
            assert!(recovered.is_coherent_successful_terminal());
            let cleanup = recovered.cleanup.as_ref().expect("cleaned retry evidence");
            assert!(cleanup.attempted);
            assert!(cleanup.removed);
            assert!(cleanup.error.is_none());
        }
        CompositionCleanupDisposition::Deferred => {
            assert!(!recovered.passed);
            assert!(!recovered.is_coherent_successful_terminal());
            assert_eq!(
                recovered.failure.as_deref(),
                Some("composition_cleanup_deferred")
            );
            let cleanup = recovered.cleanup.as_ref().expect("deferred retry evidence");
            assert!(!cleanup.attempted);
            assert!(!cleanup.removed);
            let error = cleanup.error.as_deref().expect("external observer error");
            assert!(
                error.starts_with("verifier_process_state_unknown:cannot inspect process ")
                    && [" working directory", " file descriptors", " open files"]
                        .iter()
                        .any(|suffix| error.ends_with(suffix)),
                "unexpected observer error: {error}"
            );
            let retry_worktree = Path::new(&recovered.isolated_worktree);
            assert!(retry_worktree.is_dir(), "deferred retry tree must remain");
            let retry_registration = format!("worktree {}", retry_worktree.display());
            assert!(
                run(root.path(), &["worktree", "list", "--porcelain"])
                    .lines()
                    .any(|line| line == retry_registration),
                "deferred retry worktree registration must remain"
            );
        }
        other => panic!("unexpected zombie-only retry cleanup disposition: {other:?}"),
    }
}

#[test]
fn changed_command_only_reexecutes_the_affected_node() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let first_input = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![
            command("stable", "true", &[]),
            command("changed", "true", &[]),
        ],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let first = run_composition_with_pure_cache_test_supervisor(first_input.clone())
        .expect("first attempt");
    assert!(first.passed);

    let second_input = input(
        root.path(),
        state.path(),
        first_input.binding,
        vec![
            command("stable", "true", &[]),
            command("changed", "false", &[]),
        ],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let second =
        run_composition_with_pure_cache_test_supervisor(second_input).expect("second attempt");

    assert!(!second.passed);
    if cfg!(unix) {
        assert_eq!(second.processes_spawned, 1);
        assert!(!second.execution_records[0].spawned);
        assert!(second.execution_records[0].reused);
        assert!(second.execution_records[1].spawned);
    } else {
        assert_eq!(second.processes_spawned, 2);
        assert!(second.execution_records[0].spawned);
        assert!(!second.execution_records[0].reused);
        assert!(second.execution_records[1].spawned);
    }
    assert!(!second.execution_records[1].passed);
}

#[test]
fn changed_source_file_only_reexecutes_nodes_that_observe_that_file() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-qb", "provider"]);
    fs::write(root.path().join("api.txt"), "api-v1\n").expect("api input");
    fs::write(root.path().join("docs.txt"), "docs-stable\n").expect("docs input");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "provider-v1"]);
    let first_head = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-q", "main"]);

    let state = tempdir("state");
    let mut api = command("api", "true", &[]);
    api.input_paths = vec!["api.txt".into()];
    let mut docs = command("docs", "true", &[]);
    docs.input_paths = vec!["docs.txt".into()];
    let first_input = input(
        root.path(),
        state.path(),
        binding(&base, vec![first_head.clone(), base.clone()]),
        vec![api.clone(), docs.clone()],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let first =
        run_composition_with_pure_cache_test_supervisor(first_input).expect("first attempt");
    assert!(first.passed);

    run(root.path(), &["checkout", "-q", "provider"]);
    fs::write(root.path().join("api.txt"), "api-v2\n").expect("updated api input");
    run(root.path(), &["add", "api.txt"]);
    run(root.path(), &["commit", "-qm", "provider-api-v2"]);
    let second_head = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-q", "main"]);

    let second = run_composition_with_pure_cache_test_supervisor(input(
        root.path(),
        state.path(),
        binding(&base, vec![second_head, base.clone()]),
        vec![api, docs],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("second attempt");

    assert!(second.passed);
    assert_eq!(second.processes_spawned, if cfg!(unix) { 1 } else { 2 });
    let api_record = second
        .execution_records
        .iter()
        .find(|record| record.node_id == "api")
        .unwrap();
    let docs_record = second
        .execution_records
        .iter()
        .find(|record| record.node_id == "docs")
        .unwrap();
    assert!(api_record.spawned);
    assert!(!api_record.reused);
    assert_eq!(docs_record.spawned, !cfg!(unix));
    assert_eq!(docs_record.reused, cfg!(unix));
}

#[test]
fn changed_upstream_receipt_reexecutes_transitive_dependents_only() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-qb", "provider"]);
    fs::write(root.path().join("api.txt"), "api-v1\n").expect("api input");
    fs::write(root.path().join("docs.txt"), "docs-stable\n").expect("docs input");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "provider-v1"]);
    let first_head = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-q", "main"]);

    let state = tempdir("state");
    let mut source = command("source", "cat", &["api.txt"]);
    source.input_paths = vec!["api.txt".into()];
    let mut consumer = command("consumer", "true", &[]);
    consumer.depends_on = vec!["source".into()];
    let mut transitive = command("transitive", "true", &[]);
    transitive.depends_on = vec!["consumer".into()];
    let mut independent = command("independent", "true", &[]);
    independent.input_paths = vec!["docs.txt".into()];
    let commands = vec![
        source.clone(),
        consumer.clone(),
        transitive.clone(),
        independent.clone(),
    ];
    let first = run_composition_with_pure_cache_test_supervisor(input(
        root.path(),
        state.path(),
        binding(&base, vec![first_head.clone(), base.clone()]),
        commands.clone(),
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("first attempt");
    assert!(
        first.passed,
        "first dependency attempt: outcome={:?} complete={} failure={:?} cleanup={:?} owned_tree_unknown={} node_results={:?}",
        first.execution_outcome,
        first.execution_evidence_complete,
        first.failure,
        first.cleanup_disposition,
        first.owned_tree_termination_unknown,
        first
            .execution_records
            .iter()
            .map(|record| (
                &record.node_id,
                record.passed,
                record.exit_code,
                record.termination_signal
            ))
            .collect::<Vec<_>>()
    );

    run(root.path(), &["checkout", "-q", "provider"]);
    fs::write(root.path().join("api.txt"), "api-v2\n").expect("updated api input");
    run(root.path(), &["add", "api.txt"]);
    run(root.path(), &["commit", "-qm", "provider-api-v2"]);
    let second_head = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-q", "main"]);

    let second = run_composition_with_pure_cache_test_supervisor(input(
        root.path(),
        state.path(),
        binding(&base, vec![second_head, base.clone()]),
        commands,
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("second attempt");

    assert!(
        second.passed,
        "second dependency attempt: outcome={:?} complete={} failure={:?} cleanup={:?} owned_tree_unknown={} node_results={:?}",
        second.execution_outcome,
        second.execution_evidence_complete,
        second.failure,
        second.cleanup_disposition,
        second.owned_tree_termination_unknown,
        second
            .execution_records
            .iter()
            .map(|record| (
                &record.node_id,
                record.passed,
                record.exit_code,
                record.termination_signal
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(second.processes_spawned, if cfg!(unix) { 3 } else { 4 });
    for node_id in ["source", "consumer", "transitive"] {
        let record = second
            .execution_records
            .iter()
            .find(|record| record.node_id == node_id)
            .unwrap();
        assert!(record.spawned, "{node_id} must be re-executed");
        assert!(
            !record.reused,
            "{node_id} cannot reuse a stale dependency receipt"
        );
    }
    let independent = second
        .execution_records
        .iter()
        .find(|record| record.node_id == "independent")
        .unwrap();
    assert_eq!(independent.spawned, !cfg!(unix));
    assert_eq!(independent.reused, cfg!(unix));
}

#[test]
fn actual_command_environment_change_invalidates_reuse_even_when_json_identity_is_stale() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let mut first_command = command(
        "environment-check",
        "sh",
        &["-c", "test \"$COMPOSITION_FLAVOR\" = one"],
    );
    first_command
        .environment
        .insert("COMPOSITION_FLAVOR".into(), "one".into());
    let mut first_input = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![first_command],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    first_input.timeout_seconds = 30;
    let first = run_composition(first_input.clone()).expect("first attempt");
    assert!(first.passed, "first attempt: {first:?}");

    let mut second_input = first_input;
    second_input.commands[0]
        .environment
        .insert("COMPOSITION_FLAVOR".into(), "two".into());
    // Deliberately keep every caller-supplied identity digest unchanged.
    let second = run_composition(second_input).expect("second attempt");

    assert!(!second.passed);
    assert_eq!(second.processes_spawned, 1);
    assert!(second.execution_records[0].spawned);
}

#[test]
fn inherited_environment_does_not_enter_runtime_child_or_invalidate_reuse() {
    let child_mode = std::env::var_os("COMPOSITION_ENV_CHILD").is_some();
    if child_mode {
        let root = PathBuf::from(std::env::var_os("COMPOSITION_ENV_ROOT").expect("root path"));
        let state = PathBuf::from(std::env::var_os("COMPOSITION_ENV_STATE").expect("state path"));
        let base = run(&root, &["rev-parse", "refs/heads/main"]);
        let check = command("inherited-environment-check", "env", &[]);
        let attempt = run_composition_with_test_supervisor(input(
            &root,
            &state,
            binding(&base, vec![base.clone(), base.clone()]),
            vec![check],
            vec![CompositionPrecondition::satisfied("identity-bound")],
        ))
        .expect("child composition attempt");
        assert!(attempt.passed, "child composition failed: {attempt:?}");
        return;
    }

    let root = repository();
    let state = tempdir("inherited-environment-state");
    for flavor in ["one", "two"] {
        let child = Command::new(std::env::current_exe().expect("integration-test executable"))
            .args([
                "--exact",
                "inherited_environment_does_not_enter_runtime_child_or_invalidate_reuse",
            ])
            .env("COMPOSITION_ENV_CHILD", "1")
            .env("COMPOSITION_ENV_ROOT", root.path())
            .env("COMPOSITION_ENV_STATE", state.path())
            .env("COMPOSITION_EXTERNAL_FLAVOR", flavor)
            .output()
            .expect("spawn separate test process");
        assert!(
            child.status.success(),
            "child process for flavor {flavor} failed with {:?}; stdout: {}; stderr: {}",
            child.status,
            String::from_utf8_lossy(&child.stdout),
            String::from_utf8_lossy(&child.stderr)
        );
    }

    let mut attempts = fs::read_dir(state.path())
        .expect("composition state directory")
        .map(|entry| {
            let path = entry.expect("attempt entry").path();
            serde_json::from_slice::<serde_json::Value>(&fs::read(path).expect("attempt bytes"))
                .expect("attempt JSON")
        })
        .collect::<Vec<_>>();
    attempts.sort_by_key(|attempt| {
        attempt["recordedAtUnixNanos"]
            .as_u64()
            .expect("recorded timestamp")
    });
    assert_eq!(attempts.len(), 2, "both processes must persist an attempt");
    assert_eq!(attempts[0]["processesSpawned"], 1);
    assert_eq!(
        attempts[1]["processesSpawned"],
        if cfg!(unix) { 0 } else { 1 },
        "unobservable Windows read sets execute again instead of reusing"
    );
    assert_eq!(
        attempts[1]["executionRecords"][0]["reused"],
        cfg!(unix),
        "reuse is permitted only when the platform provides a bounded input-read proof"
    );
    assert!(
        !attempts[1]["executionRecords"][0]["stdout"]
            .as_str()
            .unwrap_or_default()
            .contains("COMPOSITION_EXTERNAL_FLAVOR")
    );
}

#[cfg(unix)]
#[test]
fn untrusted_absolute_executable_fails_before_spawn() {
    use std::os::unix::fs::PermissionsExt;

    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let executable = root.path().join("verify-tool");
    fs::write(&executable, "#!/bin/sh\nexit 0\n").expect("write initial verifier");
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))
        .expect("make initial verifier executable");
    let mut check = command("toolchain-check", executable.to_str().unwrap(), &[]);
    check.input_paths = vec!["README.md".into()];
    let first_input = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![check],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let attempt = run_composition(first_input).expect("attempt rejects untrusted executable");
    assert!(!attempt.passed);
    assert_eq!(attempt.processes_spawned, 0);
    assert!(attempt.execution_records.is_empty());
    assert_eq!(
        attempt.failure.as_deref(),
        Some("composition_executable_unbound")
    );
}

#[cfg(unix)]
#[test]
fn unbound_command_path_override_fails_before_spawn() {
    use std::os::unix::fs::PermissionsExt;

    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let path_override = tempdir("command-path");
    let executable = path_override.path().join("sh");
    fs::write(&executable, "#!/bin/sh\nexit 0\n").expect("write initial command executable");
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))
        .expect("make command executable");
    let mut check = command("path-toolchain-check", "sh", &[]);
    check.environment.insert(
        "PATH".into(),
        path_override.path().to_string_lossy().into_owned(),
    );
    let first_input = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![check],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let attempt = run_composition(first_input).expect("attempt rejects PATH overlay");
    assert!(!attempt.passed);
    assert_eq!(attempt.processes_spawned, 0);
    assert!(attempt.execution_records.is_empty());
    assert_eq!(
        attempt.failure.as_deref(),
        Some("unbound_runtime_environment_override")
    );
}

#[test]
fn unbounded_external_reads_execute_again_but_independent_node_reuses() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("unbounded-read-state");
    let external = tempdir("unbounded-read-source");
    let source = external.path().join("source.txt");
    fs::write(&source, "version-one\n").expect("write external source");
    let mut check = command("external-read", "cat", &[source.to_str().unwrap()]);
    // This is deliberately not the file read by `cat`; a caller-supplied path
    // list cannot make an unbounded command read-set complete.
    check.input_paths = vec!["README.md".into()];
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![check, command("independent", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let original_json = serde_json::to_vec(&composition).expect("serialize composition input");

    let first = run_composition_with_pure_cache_test_supervisor(composition.clone())
        .expect("first attempt");
    fs::write(&source, "version-two\n").expect("change external source without editing input");
    assert_eq!(
        serde_json::to_vec(&composition).expect("serialize unchanged composition input"),
        original_json
    );
    let second =
        run_composition_with_pure_cache_test_supervisor(composition).expect("second attempt");

    assert!(first.passed && second.passed);
    assert_eq!(first.processes_spawned, 2);
    assert_eq!(second.processes_spawned, if cfg!(unix) { 1 } else { 2 });
    assert!(!second.execution_records[0].reused);
    assert_eq!(second.execution_records[1].reused, cfg!(unix));
    assert!(second.execution_records[1].node_id == "independent");
}

#[cfg(unix)]
#[test]
fn relative_executable_uses_isolated_worktree_bytes_and_changes_identity() {
    use std::os::unix::fs::PermissionsExt;

    let root = repository();
    let state = tempdir("relative-executable-state");
    let executable = root.path().join("tools/check.sh");
    fs::create_dir_all(executable.parent().expect("tools parent")).expect("create tools directory");
    fs::write(&executable, "#!/bin/sh\nexit 0\n").expect("write initial executable");
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).expect("make executable");
    run(root.path(), &["add", "tools/check.sh"]);
    run(root.path(), &["commit", "-qm", "add relative executable"]);
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let check = command("relative-check", "./tools/check.sh", &[]);
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![check],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );

    let first = run_composition(composition.clone()).expect("first attempt");
    assert!(first.passed, "first attempt: {first:?}");
    assert_ne!(
        first.identity.toolchain_digest,
        CompositionIdentity::default().toolchain_digest,
        "the Runtime must hash the executable resolved from the isolated worktree"
    );

    fs::write(&executable, "#!/bin/sh\nexit 19\n").expect("replace committed executable bytes");
    run(root.path(), &["add", "tools/check.sh"]);
    run(
        root.path(),
        &["commit", "-qm", "change relative executable"],
    );
    let next_head = run(root.path(), &["rev-parse", "HEAD"]);
    let mut second_input = composition;
    second_input.binding.target_sha = next_head.clone();
    second_input.binding.participant_heads = vec![next_head.clone(), next_head];
    let second = run_composition(second_input).expect("second attempt");

    assert!(!second.passed);
    assert_eq!(second.processes_spawned, 1);
    assert_eq!(second.execution_records[0].exit_code, Some(19));
    assert_ne!(
        second.identity.toolchain_digest,
        first.identity.toolchain_digest
    );
}

#[test]
fn empty_required_checks_fail_closed_without_a_worktree_or_process() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let attempt = run_composition(input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        Vec::new(),
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("empty composition attempt");

    assert!(!attempt.passed);
    assert_eq!(attempt.processes_spawned, 0);
    assert_eq!(attempt.failure.as_deref(), Some("required_checks_empty"));
    assert!(
        attempt
            .cleanup
            .as_ref()
            .is_some_and(|cleanup| !cleanup.attempted)
    );
}

#[test]
fn timed_out_node_is_durable_and_reports_cleanup() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let mut composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("timeout", "sh", &["-c", "sleep 2"])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    composition.timeout_seconds = 1;
    let attempt = run_composition(composition).expect("timeout composition attempt");

    assert!(!attempt.passed);
    assert_eq!(attempt.processes_spawned, 1);
    assert!(attempt.execution_records[0].timed_out);
    assert!(
        attempt
            .cleanup
            .as_ref()
            .is_some_and(|cleanup| cleanup.attempted)
    );
    assert!(attempt_record_path(state.path(), &attempt.attempt_id).exists());
}

#[cfg(unix)]
#[test]
fn signal_terminated_node_is_durable_and_not_reusable() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("signal-state");
    // Parallel package-test workers may inherit SIGINT as ignored. Reset the
    // child disposition explicitly so this remains a real signal termination
    // regardless of how Cargo's test process was launched.
    let attempt = run_composition_with_test_supervisor(input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command(
            "signal",
            "python3",
            &[
                "-c",
                "import os,signal; signal.signal(signal.SIGINT, signal.SIG_DFL); os.kill(os.getpid(), signal.SIGINT)",
            ],
        )],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("signal-terminated composition attempt with owned supervisor proof");

    assert!(!attempt.passed);
    assert_eq!(
        attempt.execution_outcome,
        CompositionExecutionOutcome::Failed
    );
    assert!(attempt.execution_evidence_complete);
    assert_eq!(
        attempt.failure.as_deref(),
        Some("command_interrupted:signal=2")
    );
    assert_eq!(attempt.processes_spawned, 1);
    assert_eq!(attempt.execution_records[0].termination_signal, Some(2));
    assert!(!attempt.execution_records[0].reused);
    let receipt = attempt
        .supervisor_receipt
        .as_ref()
        .expect("real isolated supervisor receipt");
    assert_eq!(receipt.attempt_id, attempt.attempt_id);
    assert_eq!(attempt.owner_pid, Some(receipt.supervisor.process_id));
    assert_eq!(receipt.owner, receipt.supervisor);
    assert!(!attempt.owned_tree_termination_unknown);
    #[cfg(target_os = "linux")]
    {
        assert_eq!(
            receipt.backend,
            CompositionSupervisorBackend::LinuxSubreaper
        );
        assert!(receipt.descendants_reaped_to_echild);
    }
    match attempt.cleanup_disposition {
        CompositionCleanupDisposition::Cleaned => {
            let cleanup = attempt.cleanup.as_ref().expect("cleaned signal attempt");
            assert!(cleanup.attempted);
            assert!(cleanup.removed);
            assert!(cleanup.error.is_none());
            assert!(!Path::new(&attempt.isolated_worktree).exists());
        }
        #[cfg(target_os = "linux")]
        CompositionCleanupDisposition::Deferred => {
            let cleanup = attempt.cleanup.as_ref().expect("deferred signal attempt");
            assert!(!cleanup.attempted);
            assert!(!cleanup.removed);
            let error = cleanup.error.as_deref().expect("external observer error");
            assert!(
                error.starts_with("verifier_process_state_unknown:cannot inspect process ")
                    && [" working directory", " file descriptors", " open files"]
                        .iter()
                        .any(|suffix| error.ends_with(suffix)),
                "unexpected observer error: {error}"
            );
            let retained = Path::new(&attempt.isolated_worktree);
            assert!(retained.is_dir(), "deferred signal tree must remain");
            let registration = format!("worktree {}", retained.display());
            assert!(
                run(root.path(), &["worktree", "list", "--porcelain"])
                    .lines()
                    .any(|line| line == registration),
                "deferred signal worktree registration must remain"
            );
        }
        other => panic!("unexpected signal cleanup disposition: {other:?}"),
    }
    let durable: serde_json::Value = serde_json::from_slice(
        &fs::read(attempt_record_path(state.path(), &attempt.attempt_id))
            .expect("durable signal attempt"),
    )
    .expect("signal attempt JSON");
    assert_eq!(durable["schemaVersion"], 3);
    assert_eq!(durable["executionRecords"][0]["terminationSignal"], 2);
    assert_eq!(durable["passed"], false);
}

#[test]
fn composition_v2_attempt_reads_but_does_not_reuse_v1_history() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("legacy-composition-state");
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("check", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let first = run_composition(composition.clone()).expect("write current attempt");
    assert!(first.passed, "seed attempt failed: {first:?}");
    let first_path = attempt_record_path(state.path(), &first.attempt_id);
    let mut legacy: serde_json::Value =
        serde_json::from_slice(&fs::read(&first_path).expect("first attempt bytes"))
            .expect("first attempt JSON");
    legacy["schemaVersion"] = serde_json::json!(1);
    fs::write(
        &first_path,
        serde_json::to_vec_pretty(&legacy).expect("legacy attempt bytes"),
    )
    .expect("retain legacy attempt as v1");

    let second = run_composition(composition).expect("run without legacy reuse");

    assert!(second.passed);
    assert_eq!(second.processes_spawned, 1);
    assert!(!second.execution_records[0].reused);
    let preserved: serde_json::Value =
        serde_json::from_slice(&fs::read(&first_path).expect("preserved legacy bytes"))
            .expect("preserved legacy JSON");
    assert_eq!(preserved["schemaVersion"], 1);
}

#[cfg(unix)]
#[test]
fn composition_attempt_reads_legacy_logical_id_filename_and_preserves_it() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("legacy-attempt-filename");
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("legacy-name", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let first = run_composition_with_test_supervisor(composition.clone()).expect("first attempt");
    assert!(first.passed, "first attempt failed: {first:?}");
    let portable_path = attempt_record_path(state.path(), &first.attempt_id);
    let legacy_path = state.path().join(format!("{}.json", first.attempt_id));
    fs::rename(&portable_path, &legacy_path).expect("simulate historical Unix filename");

    let second = run_composition_with_test_supervisor(composition)
        .expect("attempt should read legacy filename");

    assert!(second.passed, "second attempt failed: {second:?}");
    assert_eq!(second.processes_spawned, 0);
    assert!(second.execution_records[0].reused);
    assert!(
        legacy_path.is_file(),
        "historical attempt bytes are retained"
    );
    assert!(attempt_record_path(state.path(), &second.attempt_id).is_file());
}

#[test]
fn composition_attempt_without_schema_version_is_not_reused() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("missing-composition-schema-state");
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("check", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let first = run_composition(composition.clone()).expect("write current attempt");
    let first_path = attempt_record_path(state.path(), &first.attempt_id);
    let mut historical: serde_json::Value =
        serde_json::from_slice(&fs::read(&first_path).expect("first attempt bytes"))
            .expect("first attempt JSON");
    historical
        .as_object_mut()
        .expect("attempt object")
        .remove("schemaVersion");
    fs::write(
        &first_path,
        serde_json::to_vec_pretty(&historical).expect("historical attempt bytes"),
    )
    .expect("retain pre-version history");

    let second = run_composition(composition).expect("run without historical reuse");

    assert!(second.passed);
    assert_eq!(second.processes_spawned, 1);
    assert!(!second.execution_records[0].reused);
    let preserved: serde_json::Value =
        serde_json::from_slice(&fs::read(&first_path).expect("preserved history bytes"))
            .expect("preserved history JSON");
    assert!(preserved.get("schemaVersion").is_none());
}

#[cfg(unix)]
#[test]
fn installed_rust_toolchain_change_with_stale_json_reexecutes_reusable_node() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let home = tempdir("rustup-home");
    let proxy_dir = home.path().join(".cargo/bin");
    fs::create_dir_all(&proxy_dir).expect("create proxy directory");
    let rustup = proxy_dir.join("rustup");
    fs::write(&rustup, "#!/bin/sh\nexit 0\n").expect("write rustup proxy");
    fs::set_permissions(&rustup, fs::Permissions::from_mode(0o755))
        .expect("make rustup proxy executable");
    symlink("rustup", proxy_dir.join("cargo")).expect("create cargo proxy");

    let toolchain = home.path().join(".rustup/toolchains/test-channel");
    let toolchain_bin = toolchain.join("bin");
    fs::create_dir_all(&toolchain_bin).expect("create fake toolchain");
    fs::write(toolchain_bin.join("cargo"), b"cargo-v1").expect("write cargo identity");
    fs::write(toolchain_bin.join("rustc"), b"rustc-v1").expect("write rustc identity");
    let rustlib = toolchain.join("lib/rustlib");
    fs::create_dir_all(&rustlib).expect("create rustlib metadata");
    fs::write(rustlib.join("components"), b"rustc\nrust-std\n")
        .expect("write installed components");
    fs::write(rustlib.join("manifest-test-channel"), b"toolchain-v1")
        .expect("write toolchain manifest");
    let rustup_home = home.path().join(".rustup");
    fs::create_dir_all(&rustup_home).expect("create rustup home");
    fs::write(
        rustup_home.join("settings.toml"),
        "default_toolchain = \"test-channel\"\n[overrides]\n",
    )
    .expect("write rustup settings");

    let root = repository();
    let state = tempdir("rustup-state");
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("toolchain-change", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let unchanged_input = serde_json::to_vec(&composition).expect("serialize stable input");
    let input_path = state.path().join("composition.input");
    fs::write(&input_path, unchanged_input).expect("write stable input JSON");

    let output = Command::new(std::env::current_exe().expect("current test executable"))
        .args(["--exact", "installed_toolchain_change_child", "--nocapture"])
        .env("HOME", home.path())
        .env_remove("CARGO_HOME")
        .env_remove("RUSTUP_HOME")
        .env("COCKPIT_TOOLCHAIN_TEST_INPUT", &input_path)
        .env("COCKPIT_TOOLCHAIN_TEST_INSTALL", &toolchain)
        .output()
        .expect("run isolated child test process");
    assert!(
        output.status.success(),
        "isolated toolchain test failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(unix)]
#[test]
fn installed_toolchain_change_child() {
    let Ok(input_path) = std::env::var("COCKPIT_TOOLCHAIN_TEST_INPUT") else {
        return;
    };
    let toolchain = PathBuf::from(
        std::env::var("COCKPIT_TOOLCHAIN_TEST_INSTALL").expect("toolchain install path"),
    );
    let composition: CompositionInput =
        serde_json::from_slice(&fs::read(input_path).expect("read unchanged composition JSON"))
            .expect("decode composition JSON");

    let first = run_composition(composition.clone()).expect("first toolchain attempt");
    assert!(first.passed, "first attempt: {first:?}");
    assert_eq!(first.processes_spawned, 1);
    fs::write(toolchain.join("bin/rustc"), b"rustc-v2")
        .expect("update installed rustc without changing composition JSON");

    let second = run_composition(composition).expect("second toolchain attempt");
    assert!(second.passed, "second attempt: {second:?}");
    assert_eq!(second.processes_spawned, 1, "changed toolchain must rerun");
    assert!(!second.execution_records[0].reused);
    assert_ne!(
        first.identity.toolchain_digest,
        second.identity.toolchain_digest
    );
}
