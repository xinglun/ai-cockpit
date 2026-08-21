use cockpit_evidence::{DiffIdentity, EvidenceContext, ReusableReceipt};
use cockpit_verification::{
    ExecutionError, MAX_CAPTURE_BYTES_PER_STREAM, PlannedAction, PlannedSatisfaction,
    ProtectedGateClass, VerificationCommand, VerificationReusePolicy, execute_bounded,
    execute_bounded_at, execute_bounded_with_resource_budget, execute_verification_plan_bounded,
    plan_verification_commands,
};
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicUsize, Ordering},
};
use std::time::{SystemTime, UNIX_EPOCH};

const NOW: i64 = 1_800_000_000;

fn digest(fill: char) -> String {
    format!("sha256:{}", fill.to_string().repeat(64))
}

fn context(command_digest: String) -> EvidenceContext {
    EvidenceContext {
        content_digest: digest('a'),
        diff: DiffIdentity {
            base_commit: "1111111111111111111111111111111111111111".into(),
            head_commit: "2222222222222222222222222222222222222222".into(),
            changed_paths_digest: digest('b'),
        },
        environment_digest: digest('c'),
        command_digest,
        scope_digest: digest('e'),
        governance_digest: digest('f'),
        toolchain_digest: digest('1'),
        policy_digest: digest('2'),
        profile_digest: digest('6'),
        stage: "task".into(),
        runner: "local".into(),
    }
}

fn receipt(node_id: &str, context: &EvidenceContext) -> ReusableReceipt {
    ReusableReceipt::new(
        node_id,
        true,
        context.clone(),
        &digest('3'),
        NOW - 60,
        NOW + 60,
    )
    .expect("valid receipt")
}

fn reusable_command(id: &str, program: &str, args: Vec<String>) -> VerificationCommand {
    VerificationCommand::new(id, program, args, VerificationReusePolicy::Reusable)
}

fn always_command(id: &str, program: &str, args: Vec<String>) -> VerificationCommand {
    VerificationCommand::new(id, program, args, VerificationReusePolicy::NeverReuse)
}

#[test]
fn pinned_execution_paths_do_not_change_the_logical_command_identity() {
    let first = VerificationCommand::new_pinned(
        "tests",
        "/private/staging/one",
        vec!["--internal".into()],
        "cargo",
        vec!["test".into()],
        VerificationReusePolicy::Reusable,
    );
    let second = VerificationCommand::new_pinned(
        "tests",
        "/private/staging/two",
        vec!["--different-internal".into()],
        "cargo",
        vec!["test".into()],
        VerificationReusePolicy::Reusable,
    );
    let changed_arguments = VerificationCommand::new_pinned(
        "tests",
        "/private/staging/two",
        vec!["--different-internal".into()],
        "cargo",
        vec!["clippy".into()],
        VerificationReusePolicy::Reusable,
    );

    assert_eq!(first.command_digest(), second.command_digest());
    assert_ne!(first.command_digest(), changed_arguments.command_digest());
}

#[test]
fn bounded_execution_reports_plan_and_process_telemetry() {
    let receipt = execute_bounded(
        vec![
            always_command("one", "true", vec![]),
            always_command("two", "true", vec![]),
        ],
        2,
    )
    .expect("execute");
    assert_eq!(receipt.nodes_planned, 2);
    assert_eq!(receipt.nodes_executed, 2);
    assert_eq!(receipt.processes_spawned, 2);
    assert!(receipt.passed);
}

#[test]
fn resource_budget_rejects_zero_and_overweight_commands_fail_closed() {
    let zero = always_command("zero", "true", vec![]).with_resource_weight(0);
    assert!(matches!(
        execute_bounded_with_resource_budget(vec![zero], 1, 1),
        Err(ExecutionError::CommandExceedsResourceBudget(id)) if id == "zero"
    ));

    let overweight = always_command("heavy", "true", vec![]).with_resource_weight(2);
    assert!(matches!(
        execute_bounded_with_resource_budget(vec![overweight], 2, 1),
        Err(ExecutionError::CommandExceedsResourceBudget(id)) if id == "heavy"
    ));
}

#[test]
fn resource_budget_allows_weighted_commands_within_bound() {
    let first = always_command("first", "true", vec![]).with_resource_weight(2);
    let second = always_command("second", "true", vec![]).with_resource_weight(1);
    let result = execute_bounded_with_resource_budget(vec![first, second], 2, 2)
        .expect("weighted execution");
    assert!(result.passed);
    assert_eq!(result.processes_spawned, 2);
}

#[test]
fn single_flight_coalesces_same_repository_work_item_and_runtime_identity() {
    let coordinator = Arc::new(cockpit_verification::SingleFlightCoordinator::default());
    let key = cockpit_verification::SingleFlightKey {
        repository_id: digest('r'),
        work_item_id: "WI-SINGLE".into(),
        command_digest: digest('c'),
        runtime_digest: digest('v'),
    };
    let barrier = Arc::new(Barrier::new(3));
    let calls = Arc::new(AtomicUsize::new(0));
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..2 {
            let coordinator = Arc::clone(&coordinator);
            let key = key.clone();
            let barrier = Arc::clone(&barrier);
            let calls = Arc::clone(&calls);
            handles.push(scope.spawn(move || {
                barrier.wait();
                coordinator.execute(key, || {
                    calls.fetch_add(1, Ordering::SeqCst);
                    std::thread::sleep(std::time::Duration::from_millis(30));
                    Ok(
                        execute_bounded(vec![always_command("check", "true", vec![])], 1)
                            .expect("receipt"),
                    )
                })
            }));
        }
        barrier.wait();
        let first = handles.remove(0).join().expect("first").expect("result");
        let second = handles.remove(0).join().expect("second").expect("result");
        assert!(Arc::ptr_eq(&first, &second));
    });
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(coordinator.active_count().expect("count"), 0);
}

#[test]
fn single_flight_rejects_unbound_repository_identity() {
    let coordinator = cockpit_verification::SingleFlightCoordinator::default();
    let key = cockpit_verification::SingleFlightKey {
        repository_id: String::new(),
        work_item_id: "WI-SINGLE".into(),
        command_digest: digest('c'),
        runtime_digest: digest('v'),
    };
    let error = coordinator
        .execute(key, || Ok(execute_bounded(vec![], 1).expect("receipt")))
        .expect_err("unbound key must fail closed");
    assert_eq!(error, "single_flight_key_invalid");
}

#[test]
fn fresh_typed_receipt_skips_the_actual_failing_process() {
    let command = reusable_command("tests", "false", vec![]);
    let current = context(command.command_digest());
    let candidate = receipt("tests", &current);
    let command = command.with_reuse_candidate(Some(candidate), current);

    let plan = plan_verification_commands(vec![command], NOW).expect("plan");
    assert_eq!(plan.commands()[0].action, PlannedAction::Reuse);
    assert_eq!(
        plan.commands()[0].satisfied_by,
        PlannedSatisfaction::ReusedReceipt
    );
    assert!(plan.commands()[0].receipt_id.is_some());
    let expected_receipt_id = plan.commands()[0].receipt_id.clone();

    let receipt = execute_verification_plan_bounded(plan, 1).expect("execute");
    assert_eq!(receipt.nodes_reused, 1);
    assert_eq!(receipt.nodes_executed, 0);
    assert_eq!(receipt.processes_spawned, 0);
    assert_eq!(receipt.results.len(), 1);
    assert_eq!(receipt.results[0].action, PlannedAction::Reuse);
    assert_eq!(receipt.results[0].receipt_id, expected_receipt_id);
    assert!(
        receipt.passed,
        "the failing command must not have been called"
    );
}

#[test]
fn stale_and_missing_receipts_execute_and_report_rerun_state() {
    let stale_command = reusable_command("stale", "true", vec![]);
    let bound = context(stale_command.command_digest());
    let candidate = receipt("stale", &bound);
    let mut current = bound;
    current.environment_digest = digest('9');
    let stale_command = stale_command.with_reuse_candidate(Some(candidate), current);

    let missing_command = reusable_command("missing", "true", vec![]);
    let missing_context = context(missing_command.command_digest());
    let missing_command = missing_command.with_reuse_candidate(None, missing_context);

    let receipt = execute_bounded_at(vec![stale_command, missing_command], 2, NOW)
        .expect("execute fail closed");
    assert_eq!(receipt.nodes_executed, 2);
    assert_eq!(receipt.nodes_reused, 0);
    assert_eq!(receipt.rerun_stale, 1);
    assert_eq!(receipt.rerun_unknown, 1);
    assert!(receipt.passed);
}

#[test]
fn protected_fresh_candidate_executes_and_cannot_be_reported_skipped() {
    let command = VerificationCommand::new(
        "scope",
        "false",
        vec![],
        VerificationReusePolicy::Protected(ProtectedGateClass::Scope),
    );
    let current = context(command.command_digest());
    let candidate = receipt("scope", &current);
    let command = command.with_reuse_candidate(Some(candidate), current);

    let receipt = execute_bounded_at(vec![command], 1, NOW).expect("execute protected");
    assert_eq!(receipt.nodes_reused, 0);
    assert_eq!(receipt.nodes_executed, 1);
    assert_eq!(receipt.protected_nodes_executed, 1);
    assert_eq!(receipt.protected_nodes_skipped, 0);
    assert_eq!(receipt.rerun_unknown, 1);
    assert!(!receipt.passed, "the protected failing command must run");
}

#[test]
fn never_reuse_policy_cannot_be_overridden_by_a_fresh_candidate() {
    let command = always_command("required", "false", vec![]);
    let current = context(command.command_digest());
    let candidate = receipt("required", &current);
    let command = command.with_reuse_candidate(Some(candidate), current);

    let receipt = execute_bounded_at(vec![command], 1, NOW).expect("execute never-reuse node");

    assert_eq!(receipt.nodes_reused, 0);
    assert_eq!(receipt.processes_spawned, 1);
    assert!(!receipt.passed, "typed NeverReuse policy must win");
}

#[test]
fn dependency_rerun_forces_an_otherwise_fresh_downstream_command_to_execute() {
    let upstream = reusable_command("upstream", "true", vec![]);
    let upstream_bound = context(upstream.command_digest());
    let upstream_receipt = receipt("upstream", &upstream_bound);
    let mut upstream_current = upstream_bound;
    upstream_current.content_digest = digest('9');
    let upstream = upstream.with_reuse_candidate(Some(upstream_receipt), upstream_current);

    let downstream =
        reusable_command("downstream", "false", vec![]).with_dependencies(vec!["upstream".into()]);
    let downstream_current = context(downstream.command_digest());
    let downstream_receipt = receipt("downstream", &downstream_current);
    let downstream = downstream.with_reuse_candidate(Some(downstream_receipt), downstream_current);

    let receipt =
        execute_bounded_at(vec![downstream, upstream], 2, NOW).expect("execute dependency closure");
    assert_eq!(receipt.nodes_executed, 2);
    assert_eq!(receipt.nodes_reused, 0);
    assert_eq!(receipt.rerun_stale, 2);
    assert!(!receipt.passed, "downstream false must have executed");
}

#[test]
fn planner_derives_identity_from_the_final_actual_command() {
    let command = reusable_command("tests", "true", vec![]);
    let current = context(command.command_digest());
    let candidate = receipt("tests", &current);
    let mut command = command.with_reuse_candidate(Some(candidate), current);
    command.program = "false".into();

    let receipt = execute_bounded_at(vec![command], 1, NOW).expect("execute changed command");
    assert_eq!(receipt.rerun_stale, 1);
    assert_eq!(receipt.processes_spawned, 1);
    assert!(
        !receipt.passed,
        "changed actual command must not reuse evidence"
    );
}

#[test]
fn planner_includes_actual_arguments_in_command_identity() {
    let command = reusable_command("tests", "true", vec![]);
    let current = context(command.command_digest());
    let candidate = receipt("tests", &current);
    let mut command = command.with_reuse_candidate(Some(candidate), current);
    command.args.push("changed-argument".into());

    let receipt = execute_bounded_at(vec![command], 1, NOW).expect("execute changed arguments");

    assert_eq!(receipt.rerun_stale, 1);
    assert_eq!(receipt.processes_spawned, 1);
    assert!(receipt.passed, "true accepts the changed argument");
}

#[test]
fn failed_spawn_is_not_reported_as_a_spawned_or_executed_protected_process() {
    let command = VerificationCommand::new(
        "scope",
        "ai-cockpit-command-that-does-not-exist-wi32",
        vec![],
        VerificationReusePolicy::Protected(ProtectedGateClass::Scope),
    );

    let receipt = execute_bounded_at(vec![command], 1, NOW).expect("record spawn failure");

    assert_eq!(receipt.nodes_executed, 1);
    assert_eq!(receipt.processes_spawned, 0);
    assert_eq!(receipt.process_spawn_failures, 1);
    assert_eq!(receipt.protected_nodes_executed, 0);
    assert!(!receipt.results[0].passed);
    assert!(!receipt.passed);
}

#[test]
fn bounded_executor_starts_a_dependent_only_after_its_dependency_finishes() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let marker = std::env::temp_dir().join(format!(
        "cockpit-verification-dependency-{}-{suffix}",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&marker);
    let marker_arg = marker.to_string_lossy().into_owned();
    let upstream = always_command(
        "upstream",
        "python3",
        vec![
            "-c".into(),
            "import pathlib,sys,time; time.sleep(0.2); pathlib.Path(sys.argv[1]).write_text('ready')"
                .into(),
            marker_arg.clone(),
        ],
    );
    let downstream = always_command(
        "downstream",
        "python3",
        vec![
            "-c".into(),
            "import pathlib,sys; raise SystemExit(not pathlib.Path(sys.argv[1]).is_file())".into(),
            marker_arg,
        ],
    )
    .with_dependencies(vec!["upstream".into()]);

    let receipt = execute_bounded_at(vec![downstream, upstream], 2, NOW)
        .expect("execute dependency-aware schedule");

    assert!(
        receipt.passed,
        "dependent process must wait for its dependency to finish"
    );
    std::fs::remove_file(marker).expect("cleanup marker");
}

#[test]
fn bounded_executor_still_runs_independent_nodes_in_parallel() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let first_marker = std::env::temp_dir().join(format!(
        "cockpit-verification-parallel-first-{}-{suffix}",
        std::process::id()
    ));
    let second_marker = std::env::temp_dir().join(format!(
        "cockpit-verification-parallel-second-{}-{suffix}",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&first_marker);
    let _ = std::fs::remove_file(&second_marker);
    let script = r#"
import pathlib, sys, time
mine, other = map(pathlib.Path, sys.argv[1:])
mine.write_text("ready")
deadline = time.monotonic() + 2
while not other.is_file() and time.monotonic() < deadline:
    time.sleep(0.01)
raise SystemExit(not other.is_file())
"#;
    let first = always_command(
        "first",
        "python3",
        vec![
            "-c".into(),
            script.into(),
            first_marker.to_string_lossy().into_owned(),
            second_marker.to_string_lossy().into_owned(),
        ],
    );
    let second = always_command(
        "second",
        "python3",
        vec![
            "-c".into(),
            script.into(),
            second_marker.to_string_lossy().into_owned(),
            first_marker.to_string_lossy().into_owned(),
        ],
    );

    let receipt = execute_bounded_at(vec![first, second], 2, NOW)
        .expect("execute independent nodes in parallel");

    assert!(receipt.passed, "independent nodes must overlap");
    assert_eq!(receipt.processes_spawned, 2);
    std::fs::remove_file(first_marker).expect("cleanup first marker");
    std::fs::remove_file(second_marker).expect("cleanup second marker");
}

#[test]
fn successful_reusable_execution_returns_an_output_bound_receipt_candidate() {
    let command = reusable_command(
        "tests",
        "python3",
        vec!["-c".into(), "print('bounded evidence')".into()],
    );
    let current = context(command.command_digest());
    let command = command.with_reuse_candidate(None, current.clone());

    let result = execute_bounded_at(vec![command], 1, NOW).expect("execute");

    assert!(result.passed);
    assert_eq!(result.receipt_candidates.len(), 1);
    let candidate = &result.receipt_candidates[0];
    assert!(candidate.validate().is_ok());
    assert_eq!(candidate.node_id, "tests");
    assert_eq!(candidate.context, current);
    assert_eq!(
        result.results[0].receipt_id,
        Some(candidate.receipt_id.clone())
    );
    assert_eq!(
        result.results[0].output_digest,
        Some(candidate.output_digest.clone())
    );
}

#[test]
fn failed_spawn_nonzero_exit_and_truncated_output_create_no_passed_candidate() {
    let cases = [
        reusable_command("spawn", "command-that-does-not-exist-wi33", vec![]),
        reusable_command("exit", "false", vec![]),
        reusable_command(
            "truncated",
            "python3",
            vec![
                "-c".into(),
                format!("print('x' * {})", MAX_CAPTURE_BYTES_PER_STREAM + 1),
            ],
        ),
    ];
    let commands = cases
        .into_iter()
        .map(|command| {
            let current = context(command.command_digest());
            command.with_reuse_candidate(None, current)
        })
        .collect();

    let result = execute_bounded_at(commands, 3, NOW).expect("execute failures");

    assert!(result.receipt_candidates.is_empty());
    assert!(
        result
            .results
            .iter()
            .all(|result| result.receipt_id.is_none())
    );
    assert!(
        result
            .results
            .iter()
            .find(|result| result.node_id == "truncated")
            .expect("truncated result")
            .output_truncated
    );
}

#[test]
fn output_identity_changes_when_successful_command_output_changes() {
    let run = |text: &str| {
        let command = reusable_command(
            "tests",
            "python3",
            vec!["-c".into(), format!("print({text:?})")],
        );
        let current = context(command.command_digest());
        execute_bounded_at(vec![command.with_reuse_candidate(None, current)], 1, NOW)
            .expect("execute")
            .results
            .remove(0)
            .output_digest
            .expect("output digest")
    };

    assert_ne!(run("first"), run("second"));
}

#[cfg(unix)]
#[test]
fn descendant_inheriting_capture_pipes_cannot_hold_the_executor_open() {
    let command = always_command(
        "descendant",
        "python3",
        vec![
            "-c".into(),
            "import subprocess,sys; subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)'])"
                .into(),
        ],
    );
    let started = std::time::Instant::now();

    let result = execute_bounded_at(vec![command], 1, NOW).expect("execute descendant");

    assert!(result.passed);
    assert!(
        started.elapsed() < std::time::Duration::from_secs(5),
        "descendant must not retain the capture pipes"
    );
}

#[cfg(unix)]
#[test]
fn detached_descendant_pipe_is_cancelled_and_fails_closed() {
    let command = always_command(
        "detached-descendant",
        "python3",
        vec![
            "-c".into(),
            "import os,subprocess,sys; subprocess.Popen([sys.executable,'-c','import time; time.sleep(3)'], preexec_fn=os.setsid)"
                .into(),
        ],
    );
    let started = std::time::Instant::now();

    let result = execute_bounded_at(vec![command], 1, NOW).expect("execute detached descendant");

    assert!(!result.passed, "unclosed capture identity must fail closed");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(5),
        "detached descendant must not retain a capture worker"
    );
}

#[cfg(windows)]
#[test]
fn descendant_cannot_escape_the_windows_job_before_assignment() {
    let marker = std::env::temp_dir().join(format!(
        "cockpit-windows-descendant-{}-{}.txt",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    // Use PowerShell's process API instead of nested `cmd /c start` quoting.
    // The latter can make the direct child wait for its grandchild on hosted
    // Windows runners, turning this boundary test into a five-minute timeout
    // before it can assert the job-kill behavior.
    let marker_for_powershell = marker.display().to_string().replace('\'', "''");
    let delayed_write = format!(
        "Start-Sleep -Seconds 1; Set-Content -LiteralPath '{marker_for_powershell}' -Value escaped"
    );
    let delayed_write_for_argument = delayed_write.replace('\'', "''");
    let spawn_descendant = format!(
        "Start-Process -FilePath powershell.exe -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-Command','{delayed_write_for_argument}')"
    );
    let command = always_command(
        "windows-descendant",
        "powershell.exe",
        vec![
            "-NoLogo".into(),
            "-NoProfile".into(),
            "-NonInteractive".into(),
            "-Command".into(),
            spawn_descendant,
        ],
    );

    let result = execute_bounded_at(vec![command], 1, NOW).expect("execute descendant");
    std::thread::sleep(std::time::Duration::from_secs(3));

    assert!(result.passed);
    assert!(!marker.exists(), "descendant escaped the kill-on-close job");
    let _ = std::fs::remove_file(marker);
}
