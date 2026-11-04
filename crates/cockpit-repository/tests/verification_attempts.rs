use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::{RuntimeContext, digest_json};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions, attach,
    checkpoint_work_item, load_reusable_verification_attempt, outcome_v2_with_runtime,
    persist_verification_attempt, persist_verification_attempt_superseding,
    plan_repository_verification_action, preflight_work_item, record_verification_with_runtime,
    run_repository_verification, start_work_item_with_options,
    work_item_status_snapshot_with_runtime,
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn attached_repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(directory.path().join("src")).expect("src");
    fs::write(
        directory.path().join("Cargo.toml"),
        "[package]\nname = \"verification-attempt-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    fs::write(
        directory.path().join("src/lib.rs"),
        "pub fn value() -> u8 { 1 }\n",
    )
    .expect("source");
    run(directory.path(), &["init", "-q"]);
    run(directory.path(), &["add", "."]);
    run(
        directory.path(),
        &[
            "-c",
            "user.name=AI Cockpit Test",
            "-c",
            "user.email=ai-cockpit@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ],
    );
    attach(directory.path()).expect("attach");
    directory
}

fn start_attempt(root: &Path) {
    start_work_item_with_options(
        root,
        "WI-ATTEMPT",
        "Preserve verification attempts",
        "Test identity-bound attempt persistence",
        &["crates/cockpit-repository/src/lib.rs".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            required_evidence_classes: vec!["verification".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
}

fn repository() -> tempfile::TempDir {
    let directory = attached_repository();
    start_attempt(directory.path());
    directory
}

fn run(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .status()
            .expect("git")
            .success()
    );
}

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "0.2.91-test".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"verification-attempt-runtime"),
    }
}

#[cfg(unix)]
#[test]
fn interrupted_run_checkpoint_requires_exact_plan_and_node_partition() {
    let directory = repository();
    let root = directory.path();
    let snapshot = GitRepository::discover(root).unwrap().snapshot().unwrap();
    let command = checkpoint_command(root, "printf actual-output");
    let plan = plan_repository_verification_action(root, &command, &snapshot, 1).unwrap();
    for (pointer, mutation) in [
        ("/nodeId", json!("foreign")),
        ("/program", json!("foreign")),
        ("/args", json!(["changed"])),
        ("/timeoutSeconds", json!(1)),
        ("/identityBinding/repositoryId", json!("foreign")),
        (
            "/identityBinding/repositorySnapshotDigest",
            json!(Digest::sha256_bytes(b"other-source")),
        ),
        (
            "/identityBinding/runtimeDigest",
            json!(Digest::sha256_bytes(b"other-runtime")),
        ),
        ("/identityBinding/commandDigest", json!("invalid")),
    ] {
        let mut changed = plan.clone();
        *changed.pointer_mut(pointer).unwrap() = mutation;
        assert!(
            cockpit_repository::VerificationRunCheckpoint::prepare(
                root,
                std::slice::from_ref(&command),
                &[changed],
                &snapshot,
                &runtime()
            )
            .is_err(),
            "{pointer}"
        );
    }
    assert!(
        cockpit_repository::VerificationRunCheckpoint::prepare(
            root,
            &[command.clone(), command.clone()],
            &[plan.clone(), plan.clone()],
            &snapshot,
            &runtime()
        )
        .is_err()
    );
    assert!(
        cockpit_repository::VerificationRunCheckpoint::prepare(
            root,
            std::slice::from_ref(&command),
            &[],
            &snapshot,
            &runtime()
        )
        .is_err()
    );
    let checkpoint = cockpit_repository::VerificationRunCheckpoint::prepare(
        root,
        std::slice::from_ref(&command),
        &[plan],
        &snapshot,
        &runtime(),
    )
    .unwrap();
    for (unavailable, unstarted) in [
        (vec![], vec![]),
        (vec![], vec!["foreign".into()]),
        (
            vec![(command.node_id.clone(), "error".into())],
            vec![command.node_id.clone()],
        ),
    ] {
        assert!(
            checkpoint
                .record_interrupted(&[], &unavailable, &unstarted, "worker", "failed")
                .is_err()
        );
    }
    assert_eq!(
        checkpoint_values(root).len(),
        1,
        "rejected identities must not publish a terminal"
    );
}

#[cfg(unix)]
#[test]
fn interrupted_run_checkpoint_survives_git_loss_without_becoming_reusable() {
    let directory = repository();
    let root = directory.path();
    let snapshot = GitRepository::discover(root).unwrap().snapshot().unwrap();
    let command = checkpoint_command(root, "printf actual-output");
    let plan = plan_repository_verification_action(root, &command, &snapshot, 1).unwrap();
    let checkpoint = cockpit_repository::VerificationRunCheckpoint::prepare(
        root,
        std::slice::from_ref(&command),
        &[plan],
        &snapshot,
        &runtime(),
    )
    .unwrap();
    let run = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    )
    .unwrap();
    assert!(run.receipt.passed);
    fs::rename(root.join(".git"), root.join(".git-retained")).unwrap();
    let value = checkpoint
        .record_interrupted(
            &[&run.receipt],
            &[],
            &[],
            "aggregate_snapshot",
            "Git metadata unavailable after execution",
        )
        .expect("failure persistence must not consult Git again");
    assert_eq!(value["passed"], false);
    assert_eq!(value["postExecutionIdentityValidated"], false);
    assert_eq!(
        value["completedReceipts"][0]["executionRecords"][0]["stdout"],
        "61637475616c2d6f7574707574"
    );
    assert_eq!(
        value["completedReceipts"][0]["executionRecords"][0]["exitCode"],
        0
    );
    assert!(value.get("attemptId").is_none());
    drop(checkpoint);
    let stored = checkpoint_values(root)
        .into_iter()
        .find(|fact| fact["state"] == "interrupted")
        .unwrap();
    assert_eq!(
        stored, value,
        "a new read must recover the exact immutable fact"
    );
    let mut unsigned = stored.clone();
    unsigned.as_object_mut().unwrap().remove("checkpointId");
    assert_eq!(
        stored["checkpointId"],
        json!(digest_json(&unsigned).unwrap())
    );
    fs::rename(root.join(".git-retained"), root.join(".git")).unwrap();
    assert!(
        load_reusable_verification_attempt(root, "WI-ATTEMPT", &[command], &snapshot, &runtime())
            .unwrap()
            .is_none(),
        "an interruption is not a complete reusable attempt"
    );
}

#[cfg(unix)]
#[test]
fn interrupted_run_checkpoint_rejects_replaced_destination() {
    let directory = repository();
    let root = directory.path();
    let snapshot = GitRepository::discover(root).unwrap().snapshot().unwrap();
    let command = checkpoint_command(root, "printf not-started");
    let plan = plan_repository_verification_action(root, &command, &snapshot, 1).unwrap();
    let checkpoint = cockpit_repository::VerificationRunCheckpoint::prepare(
        root,
        std::slice::from_ref(&command),
        &[plan],
        &snapshot,
        &runtime(),
    )
    .unwrap();
    fs::rename(
        root.join(".ai/evidence"),
        root.join(".ai/evidence-retained"),
    )
    .unwrap();
    fs::create_dir(root.join(".ai/evidence")).unwrap();
    let error = checkpoint
        .record_interrupted(
            &[],
            &[],
            &[command.node_id],
            "dispatch",
            "stopped before dispatch",
        )
        .unwrap_err();
    assert!(
        error.to_string().contains("evidence directory changed"),
        "{error}"
    );
    assert_eq!(fs::read_dir(root.join(".ai/evidence")).unwrap().count(), 0);
    assert_eq!(
        fs::read_dir(root.join(".ai/evidence-retained"))
            .unwrap()
            .count(),
        1,
        "retain the prepared original fact"
    );
}

#[cfg(unix)]
#[test]
fn completed_node_survives_post_execution_repository_failure() {
    let directory = repository();
    let root = directory.path();
    let mut command = request(root);
    command.node_id = "durable-before-post-snapshot".into();
    command.program = "sh".into();
    command.args = vec![
        "-c".into(),
        "mv .git .git-after-execution && printf retained-node-output".into(),
    ];
    command.work_item_id = Some("WI-ATTEMPT".into());

    let result = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    );
    assert!(
        result.is_err(),
        "the missing Git repository must still fail"
    );
    assert!(
        root.join(".git-after-execution").is_dir(),
        "the real child must have run: {result:?}"
    );
    let attempts = fs::read_dir(root.join(".ai/evidence"))
        .expect("evidence directory")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("WI-ATTEMPT.verification-attempt.")
        })
        .map(|entry| {
            serde_json::from_slice::<serde_json::Value>(
                &fs::read(entry.path()).expect("attempt bytes"),
            )
            .expect("attempt JSON")
        })
        .collect::<Vec<_>>();
    assert!(
        attempts
            .iter()
            .any(|attempt| attempt["state"] == "execution_prepared"),
        "a durable declaration must precede the child"
    );
    let completed = attempts
        .iter()
        .find(|attempt| attempt["state"] == "node_execution_completed")
        .expect("actual child output must survive post-execution Git failure");
    let records = completed["executionRecords"].as_array().expect("records");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["nodeId"], command.node_id);
    assert_eq!(records[0]["spawned"], true);
    assert_eq!(records[0]["exitCode"], 0);
    assert_eq!(records[0]["timedOut"], false);
    assert_eq!(
        records[0]["stdout"],
        "72657461696e65642d6e6f64652d6f7574707574"
    );
    assert_eq!(
        completed["commands"][0]["commandDigest"],
        records[0]["commandDigest"]
    );
    assert_ne!(completed["state"], "execution_completed");
    assert!(
        !root
            .join(".ai/evidence/WI-ATTEMPT.verification.json")
            .exists()
    );
}

#[cfg(unix)]
fn checkpoint_command(root: &Path, script: &str) -> RepositoryVerificationRequest {
    let mut command = request(root);
    command.program = "sh".into();
    command.args = vec!["-c".into(), script.into()];
    command.work_item_id = Some("WI-ATTEMPT".into());
    command
}

#[cfg(unix)]
fn checkpoint_values(root: &Path) -> Vec<serde_json::Value> {
    fs::read_dir(root.join(".ai/evidence"))
        .expect("evidence directory")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("WI-ATTEMPT.verification-attempt.")
        })
        .map(|entry| {
            serde_json::from_slice(&fs::read(entry.path()).expect("checkpoint bytes"))
                .expect("checkpoint JSON")
        })
        .collect()
}

#[cfg(unix)]
#[test]
fn successful_node_checkpoint_is_not_completion_or_reuse_evidence() {
    let directory = repository();
    let root = directory.path();
    let command = checkpoint_command(root, "printf checkpoint-success");
    let result = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    )
    .expect("execute actual child");
    assert!(result.receipt.passed);
    let values = checkpoint_values(root);
    assert_eq!(values.len(), 2);
    for value in &values {
        assert_eq!(value["kind"], "verification_execution_checkpoint");
        assert_eq!(value["passed"], false);
        assert_eq!(value["postExecutionIdentityValidated"], false);
        let mut unsigned = value.clone();
        unsigned
            .as_object_mut()
            .expect("object")
            .remove("checkpointId");
        assert_eq!(
            value["checkpointId"],
            json!(digest_json(&unsigned).expect("digest"))
        );
        assert!(value.get("attemptId").is_none());
    }
    assert_eq!(values[0]["executionId"], values[1]["executionId"]);
    assert!(
        load_reusable_verification_attempt(
            root,
            "WI-ATTEMPT",
            &[command],
            &result.final_snapshot,
            &runtime(),
        )
        .expect("load attempts")
        .is_none()
    );
    assert!(
        !root
            .join(".ai/evidence/WI-ATTEMPT.verification.json")
            .exists()
    );
}

#[cfg(unix)]
#[test]
fn nonzero_child_checkpoint_preserves_actual_exit_and_stays_failed() {
    let directory = repository();
    let root = directory.path();
    let command = checkpoint_command(root, "printf checkpoint-failure; exit 7");
    let result = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    )
    .expect("executor returns the failed child result");
    assert!(!result.receipt.passed);
    let values = checkpoint_values(root);
    let captured = values
        .iter()
        .find(|value| value["state"] == "node_execution_completed")
        .expect("actual result");
    assert_eq!(captured["executionRecords"][0]["spawned"], true);
    assert_eq!(captured["executionRecords"][0]["exitCode"], 7);
    assert_eq!(captured["receipt"]["passed"], false);
    assert_eq!(captured["passed"], false);
    assert!(
        load_reusable_verification_attempt(
            root,
            "WI-ATTEMPT",
            &[command],
            &result.final_snapshot,
            &runtime(),
        )
        .expect("load attempts")
        .is_none()
    );
}

#[cfg(unix)]
#[test]
fn checkpoint_preparation_failure_prevents_child_execution() {
    let directory = repository();
    let root = directory.path();
    fs::rename(
        root.join(".ai/evidence"),
        root.join(".ai/evidence-retained"),
    )
    .expect("retain evidence");
    fs::write(root.join(".ai/evidence"), b"not a directory").expect("invalid destination");
    let command = checkpoint_command(root, "printf must-not-run > .ai/child-witness");
    let result = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    );
    assert!(result.is_err());
    assert!(
        !root.join(".ai/child-witness").exists(),
        "a checkpoint is required before spawn"
    );
    assert!(
        !root
            .join(".ai/evidence-retained/WI-ATTEMPT.verification.json")
            .exists()
    );
}

#[cfg(unix)]
#[test]
fn checkpoint_directory_replacement_after_child_never_reports_success() {
    let directory = repository();
    let root = directory.path();
    let command = checkpoint_command(
        root,
        "mv .ai/evidence .ai/evidence-retained && mkdir .ai/evidence && printf child-finished > .ai/child-witness",
    );
    let result = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    );
    let error = result.expect_err("changed evidence destination must reject");
    assert!(
        error
            .to_string()
            .contains("execution checkpoint evidence directory changed"),
        "{error}"
    );
    assert_eq!(
        fs::read(root.join(".ai/child-witness")).expect("child ran"),
        b"child-finished"
    );
    assert!(checkpoint_values(root).is_empty());
    assert!(
        !root
            .join(".ai/evidence/WI-ATTEMPT.verification.json")
            .exists()
    );
    assert_eq!(
        fs::read_dir(root.join(".ai/evidence-retained"))
            .expect("original evidence retained")
            .filter_map(Result::ok)
            .filter(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with("WI-ATTEMPT.verification-attempt."))
            .count(),
        1
    );
}

fn request(_root: &Path) -> RepositoryVerificationRequest {
    RepositoryVerificationRequest {
        node_id: "project-command-0".into(),
        program: "cargo".into(),
        args: vec!["test".into(), "--locked".into(), "--workspace".into()],
        scope: vec!["**".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: runtime().runtime_digest.to_string(),
        base_commit: None,
        workers: 1,
        work_item_id: None,
        timeout_seconds: None,
        policy: RepositoryVerificationPolicy::NeverReuse,
    }
}

fn attempt_receipt(
    root: &Path,
    request: &RepositoryVerificationRequest,
    passed: bool,
) -> serde_json::Value {
    let snapshot = GitRepository::discover(root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let plan = plan_repository_verification_action(root, request, &snapshot, 0)
        .expect("real command plan");
    let command_digest = plan["identityBinding"]["commandDigest"]
        .as_str()
        .expect("planned command digest");
    json!({
        "passed": passed,
        "executionRecords": [{
            "nodeId": request.node_id,
            "commandDigest": command_digest,
            "spawned": true,
            "passed": passed,
            "exitCode": if passed { json!(0) } else { json!(1) },
            "stdout": "6f6b",
            "stderr": "",
            "stdoutTruncated": false,
            "stderrTruncated": false,
            "timedOut": false,
            "timeoutSeconds": request
                .timeout_seconds
                .unwrap_or(cockpit_verification::DEFAULT_EXECUTION_SECONDS),
            "elapsedMs": 17
        }],
        "processesSpawned": 1,
        "results": [{
            "nodeId": request.node_id,
            "passed": passed,
            "reused": false,
            "protected": false,
            "action": "execute",
            "state": "fresh",
            "reason": if passed { "execution" } else { "failed" },
            "receiptId": null,
            "outputDigest": null,
            "outputTruncated": false,
            "timedOut": false,
            "satisfiedBy": "execution"
        }]
    })
}

fn rebind_attempt_timestamp(
    root: &Path,
    persisted: &serde_json::Value,
    created_at: &str,
) -> (String, PathBuf) {
    let original_path = root.join(persisted["path"].as_str().expect("attempt path"));
    let mut attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(&original_path).expect("attempt bytes"))
            .expect("attempt JSON");
    attempt["createdAt"] = json!(created_at);
    attempt
        .as_object_mut()
        .expect("attempt object")
        .remove("attemptId");
    let attempt_id = digest_json(&attempt).expect("attempt digest").to_string();
    attempt["attemptId"] = json!(&attempt_id);
    let suffix = attempt_id.strip_prefix("sha256:").unwrap_or(&attempt_id);
    let rebound_path = original_path
        .parent()
        .expect("evidence directory")
        .join(format!("WI-ATTEMPT.verification-attempt.{suffix}.json"));
    fs::write(
        &rebound_path,
        serde_json::to_vec_pretty(&attempt).expect("serialize rebound attempt"),
    )
    .expect("write rebound attempt");
    fs::remove_file(original_path).expect("remove fixture's prior attempt identity");
    (attempt_id, rebound_path)
}

#[test]
fn precondition_attempt_is_durable_without_spawning_a_process() {
    let directory = repository();
    let root = directory.path();
    let snapshot = GitRepository::discover(root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let request = request(root);
    let receipt = persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        &[request],
        &snapshot,
        &runtime(),
        "precondition_rejected",
        Some(("verification_preconditions", "preflight is stale")),
        None,
    )
    .expect("persist attempt");
    let path = root.join(receipt["path"].as_str().expect("path"));
    let stored: serde_json::Value =
        serde_json::from_slice(&fs::read(path).expect("attempt")).expect("json");
    assert_eq!(stored["state"], "precondition_rejected");
    assert_eq!(stored["processesSpawned"], 0);
    assert!(
        stored["executionRecords"]
            .as_array()
            .expect("records")
            .is_empty()
    );
    assert_eq!(stored["diagnostic"]["code"], "verification_preconditions");
}

#[test]
fn package_test_attempt_commands_record_the_effective_implicit_deadline() {
    let directory = repository();
    let root: &::std::path::Path = directory.path();
    let snapshot = GitRepository::discover(root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let mut request = request(root);
    request.args = vec![
        "test".into(),
        "--package".into(),
        "verification-attempt-fixture".into(),
    ];
    let persisted = persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        &[request.clone()],
        &snapshot,
        &runtime(),
        "precondition_rejected",
        Some((
            "verification_preconditions",
            "synthetic admission rejection",
        )),
        None,
    )
    .expect("persist attempt");
    let stored: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(persisted["path"].as_str().expect("path"))).expect("attempt"),
    )
    .expect("attempt JSON");
    assert_eq!(stored["commands"][0]["timeoutSeconds"], 600);
    assert_eq!(stored["processesSpawned"], 0);
}

#[test]
fn package_test_attempt_command_digest_matches_its_real_execution_record() {
    let directory = repository();
    let root: &::std::path::Path = directory.path();
    let mut request = request(root);
    request.args = vec![
        "test".into(),
        "--package".into(),
        "verification-attempt-fixture".into(),
    ];
    let run = run_repository_verification(root, &request).expect("execute package test");
    assert!(run.receipt.passed, "package execution receipt");
    let receipt = serde_json::to_value(&run.receipt).expect("receipt JSON");
    let persisted = persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        std::slice::from_ref(&request),
        &run.final_snapshot,
        &runtime(),
        "execution_completed",
        None,
        Some(&receipt),
    )
    .expect("persist execution attempt");
    let stored: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(persisted["path"].as_str().expect("path"))).expect("attempt"),
    )
    .expect("attempt JSON");
    assert_eq!(stored["commands"][0]["timeoutSeconds"], 600);
    assert_eq!(stored["executionRecords"][0]["timeoutSeconds"], 600);
    assert_eq!(
        stored["commands"][0]["commandDigest"], stored["executionRecords"][0]["commandDigest"],
        "attempt metadata must bind the actual prepared execution command"
    );
    assert!(
        load_reusable_verification_attempt(
            root,
            "WI-ATTEMPT",
            &[request],
            &run.final_snapshot,
            &runtime(),
        )
        .expect("read reusable attempt")
        .is_some(),
        "a passed, exact-bound package attempt should be reusable"
    );
}

#[test]
fn attempt_diagnostic_is_bounded_without_corrupting_utf8() {
    let directory = repository();
    let root = directory.path();
    let snapshot = GitRepository::discover(root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let request = request(root);
    let message = "诊断信息".repeat(2_000);
    let receipt = persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        &[request],
        &snapshot,
        &runtime(),
        "precondition_rejected",
        Some(("verification_preconditions", &message)),
        None,
    )
    .expect("persist bounded attempt");
    let stored: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(receipt["path"].as_str().expect("attempt path")))
            .expect("attempt bytes"),
    )
    .expect("attempt JSON");
    let diagnostic = stored["diagnostic"]["message"]
        .as_str()
        .expect("diagnostic message");
    assert!(
        diagnostic.len() <= 1_024,
        "diagnostic was {} bytes",
        diagnostic.len()
    );
    assert!(diagnostic.starts_with("诊断信息"));
    assert!(diagnostic.ends_with('…'));
}

#[test]
fn rejected_formal_receipt_is_projected_with_its_bound_attempt_in_outcome_and_status() {
    let directory = repository();
    let root = directory.path();
    let snapshot = GitRepository::discover(root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let request = request(root);
    let receipt = attempt_receipt(root, &request, true);
    let persisted = persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        std::slice::from_ref(&request),
        &snapshot,
        &runtime(),
        "formal_receipt_rejected",
        Some(("formal_receipt", "completion evidence was rejected")),
        Some(&receipt),
    )
    .expect("persist rejected formal receipt attempt");

    let outcome =
        outcome_v2_with_runtime(root, "WI-ATTEMPT", &runtime()).expect("Outcome projection");
    let status = work_item_status_snapshot_with_runtime(root, "WI-ATTEMPT", &runtime())
        .expect("status projection");
    let attempt_id = persisted["attemptId"].as_str().expect("attempt ID");
    let expected_unknown_prefix = format!(
        "verification_attempt_formal_receipt_rejected:{attempt_id}:formal_receipt:completion evidence was rejected:snapshot="
    );
    let attempt_path = persisted["path"].as_str().expect("attempt path");

    let projected_unknown = outcome
        .unknowns
        .iter()
        .find(|unknown| unknown.starts_with(&expected_unknown_prefix))
        .expect("bounded rejection diagnostic in Outcome");
    assert_eq!(format!("{:?}", outcome.state), "NotReady");
    assert_eq!(status.verification, "not_ready");
    assert!(projected_unknown.contains(attempt_path));
    assert!(status.unknowns.contains(projected_unknown));
    assert_eq!(outcome.unknowns, status.unknowns);
    assert!(
        outcome
            .evidence_refs
            .iter()
            .any(|reference| reference == attempt_path)
    );
    assert!(
        status
            .unknowns
            .iter()
            .any(|unknown| unknown.contains(attempt_path))
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(
            &fs::read(root.join(attempt_path)).expect("attempt evidence")
        )
        .expect("attempt JSON")["repositorySnapshotDigest"],
        serde_json::json!(cockpit_repository::snapshot_digest(&snapshot).expect("snapshot digest"))
    );
}

#[test]
fn persisted_rejection_supersedes_execution_with_equal_timestamps() {
    let directory = repository();
    let root = directory.path();
    let snapshot = GitRepository::discover(root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let request = request(root);
    let receipt = attempt_receipt(root, &request, true);
    let execution = persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        std::slice::from_ref(&request),
        &snapshot,
        &runtime(),
        "execution_completed",
        None,
        Some(&receipt),
    )
    .expect("persist successful execution");
    let (execution_id, execution_path) =
        rebind_attempt_timestamp(root, &execution, "2026-09-27T00:00:00Z");

    let diagnostic = "completion evidence was rejected";
    let rejection = persist_verification_attempt_superseding(
        root,
        "WI-ATTEMPT",
        std::slice::from_ref(&request),
        &snapshot,
        &runtime(),
        "formal_receipt_rejected",
        Some(("formal_receipt", diagnostic)),
        Some(&receipt),
        &execution_id,
    )
    .expect("persist receipt rejection superseding execution");
    let (rejection_id, rejection_path) =
        rebind_attempt_timestamp(root, &rejection, "2026-09-27T00:00:00Z");
    let execution_json: serde_json::Value =
        serde_json::from_slice(&fs::read(execution_path).expect("rebound execution attempt"))
            .expect("execution attempt JSON");
    let rejection_json: serde_json::Value =
        serde_json::from_slice(&fs::read(rejection_path).expect("rebound rejection attempt"))
            .expect("rejection attempt JSON");
    assert_eq!(execution_json["createdAt"], rejection_json["createdAt"]);
    assert_eq!(execution_json["state"], "execution_completed");
    assert_eq!(rejection_json["state"], "formal_receipt_rejected");
    assert_eq!(rejection_json["diagnostic"]["code"], "formal_receipt");
    assert_eq!(rejection_json["supersedesAttemptId"], execution_id);

    let outcome = outcome_v2_with_runtime(root, "WI-ATTEMPT", &runtime())
        .expect("Outcome projection after tied attempts");
    let expected = format!(
        "verification_attempt_formal_receipt_rejected:{rejection_id}:formal_receipt:{diagnostic}:snapshot="
    );
    assert!(
        outcome
            .unknowns
            .iter()
            .any(|unknown| unknown.starts_with(&expected))
    );
}

#[test]
fn successful_formal_verification_and_attempt_are_projected_consistently() {
    let directory = repository();
    let root = directory.path();
    let contract_path = root.join(".ai/work-items/active/WI-ATTEMPT.contract.json");
    preflight_work_item(root, &contract_path).expect("preflight");
    checkpoint_work_item(root, "WI-ATTEMPT").expect("checkpoint");
    let verification_request = RepositoryVerificationRequest {
        node_id: "attempt-success".into(),
        program: "true".into(),
        args: Vec::new(),
        scope: vec!["**".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: runtime().runtime_digest.to_string(),
        base_commit: None,
        workers: 1,
        work_item_id: None,
        timeout_seconds: None,
        policy: RepositoryVerificationPolicy::NeverReuse,
    };
    let run = run_repository_verification(root, &verification_request).expect("execute check");
    let receipt = serde_json::to_value(&run.receipt).expect("receipt JSON");
    let attempt = persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        std::slice::from_ref(&verification_request),
        &run.final_snapshot,
        &runtime(),
        "execution_completed",
        None,
        Some(&receipt),
    )
    .expect("persist completed execution");
    record_verification_with_runtime(
        root,
        "WI-ATTEMPT",
        &receipt,
        &runtime(),
        &run.final_snapshot,
    )
    .expect("record formal verification");

    let current_snapshot = GitRepository::discover(root)
        .expect("git")
        .snapshot()
        .expect("current snapshot");
    let stored_attempt: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(attempt["path"].as_str().expect("attempt path")))
            .expect("stored attempt"),
    )
    .expect("attempt JSON");
    assert_eq!(
        stored_attempt["repositorySnapshotDigest"],
        serde_json::json!(
            cockpit_repository::snapshot_digest(&current_snapshot).expect("current digest")
        ),
        "recording the formal receipt must not stale its execution attempt"
    );

    let outcome =
        outcome_v2_with_runtime(root, "WI-ATTEMPT", &runtime()).expect("Outcome projection");
    let status = work_item_status_snapshot_with_runtime(root, "WI-ATTEMPT", &runtime())
        .expect("status projection");
    let attempt_path = attempt["path"].as_str().expect("attempt path");
    assert_eq!(format!("{:?}", outcome.state), "Verified");
    assert_eq!(status.verification, "verified");
    assert_eq!(status.unknowns, outcome.unknowns);
    assert!(
        outcome
            .evidence_refs
            .iter()
            .any(|reference| reference == attempt_path),
        "attempt was not projected: state={}, snapshot={}, contract={}, runtime={}, records={}, commands={}, receipt={}",
        stored_attempt["state"],
        stored_attempt["repositorySnapshotDigest"],
        stored_attempt["contractDigest"],
        stored_attempt["runtimeDigest"],
        stored_attempt["executionRecords"],
        stored_attempt["commands"],
        stored_attempt["receipt"]
    );
    assert!(
        !outcome.unknowns.iter().any(|unknown| {
            unknown.starts_with("verification_attempt_formal_receipt_rejected:")
        })
    );
}

#[test]
fn stale_attempt_remains_durable_but_is_not_projected_as_current() {
    let directory = repository();
    let root = directory.path();
    let snapshot = GitRepository::discover(root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let request = request(root);
    let persisted = persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        std::slice::from_ref(&request),
        &snapshot,
        &runtime(),
        "formal_receipt_rejected",
        Some(("formal_receipt", "old snapshot rejection")),
        Some(&attempt_receipt(root, &request, true)),
    )
    .expect("persist attempt");
    let path = root.join(persisted["path"].as_str().expect("attempt path"));
    let original_bytes = fs::read(&path).expect("attempt bytes");
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 2 }\n").expect("change source");

    let outcome = outcome_v2_with_runtime(root, "WI-ATTEMPT", &runtime())
        .expect("Outcome projection after source change");
    assert!(
        !outcome.unknowns.iter().any(|unknown| {
            unknown.starts_with("verification_attempt_formal_receipt_rejected:")
        })
    );
    let persisted_path = persisted["path"].as_str().expect("attempt path");
    assert!(
        !outcome
            .evidence_refs
            .iter()
            .any(|reference| reference == persisted_path)
    );
    assert_eq!(
        fs::read(path).expect("attempt remains durable"),
        original_bytes
    );
}

#[test]
fn successful_execution_can_be_reused_after_formal_receipt_rejection() {
    let directory = repository();
    let root = directory.path();
    let snapshot = GitRepository::discover(root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let request = request(root);
    let receipt = attempt_receipt(root, &request, true);
    persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        std::slice::from_ref(&request),
        &snapshot,
        &runtime(),
        "formal_receipt_rejected",
        Some(("formal_receipt", "completion evidence was rejected")),
        Some(&receipt),
    )
    .expect("persist attempt");
    let loaded = load_reusable_verification_attempt(
        root,
        "WI-ATTEMPT",
        std::slice::from_ref(&request),
        &snapshot,
        &runtime(),
    )
    .expect("load attempt");
    assert!(loaded.is_some());
    assert_eq!(loaded.expect("attempt")["receipt"]["passed"], true);
}

#[test]
fn changed_source_or_runtime_invalidates_reuse_but_keeps_attempt_bytes() {
    let directory = repository();
    let root = directory.path();
    let snapshot = GitRepository::discover(root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let request = request(root);
    persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        std::slice::from_ref(&request),
        &snapshot,
        &runtime(),
        "formal_receipt_rejected",
        None,
        Some(&attempt_receipt(root, &request, true)),
    )
    .expect("persist attempt");
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 2 }\n").expect("source edit");
    let changed_snapshot = GitRepository::discover(root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    assert!(
        load_reusable_verification_attempt(
            root,
            "WI-ATTEMPT",
            &[request],
            &changed_snapshot,
            &runtime()
        )
        .expect("load changed source")
        .is_none()
    );
}

#[test]
fn timed_out_attempt_is_preserved_but_never_reused() {
    let directory = repository();
    let root = directory.path();
    let snapshot = GitRepository::discover(root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let request = request(root);
    let mut receipt = attempt_receipt(root, &request, false);
    receipt["executionRecords"][0]["timedOut"] = json!(true);
    receipt["executionRecords"][0]["exitCode"] = serde_json::Value::Null;
    let persisted = persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        std::slice::from_ref(&request),
        &snapshot,
        &runtime(),
        "execution_failed",
        Some(("verification_execution", "verification timed out")),
        Some(&receipt),
    )
    .expect("persist timeout");
    let stored: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(persisted["path"].as_str().expect("path"))).expect("attempt"),
    )
    .expect("json");
    assert_eq!(stored["executionRecords"][0]["timedOut"], true);
    assert_eq!(
        stored["executionRecords"][0]["exitCode"],
        serde_json::Value::Null
    );
    assert!(
        load_reusable_verification_attempt(
            root,
            "WI-ATTEMPT",
            std::slice::from_ref(&request),
            &snapshot,
            &runtime(),
        )
        .expect("load timeout")
        .is_none()
    );
}

#[test]
fn changed_timeout_invalidates_reuse_without_rewriting_the_attempt() {
    let directory = repository();
    let root = directory.path();
    let snapshot = GitRepository::discover(root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let request = request(root);
    persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        std::slice::from_ref(&request),
        &snapshot,
        &runtime(),
        "formal_receipt_rejected",
        None,
        Some(&attempt_receipt(root, &request, true)),
    )
    .expect("persist attempt");

    let mut changed = request.clone();
    changed.timeout_seconds = Some(1);
    assert!(
        load_reusable_verification_attempt(
            root,
            "WI-ATTEMPT",
            std::slice::from_ref(&changed),
            &snapshot,
            &runtime(),
        )
        .expect("load changed timeout")
        .is_none()
    );
}

#[cfg(unix)]
fn authorize_local_node_fixture_timeout(root: &Path) {
    // Install only this disposable repository's finite grant before native WI
    // creation/preflight/checkpoint; retain explicit timeout identity coverage.
    fs::write(
        root.join(".ai/policy.json"),
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": 1,
            "project": {
                "policyId": "local-node-timeout-fixture-v1",
                "layer": "project",
                "rules": [{
                    "operation": "modify_source",
                    "approvalMode": "single_authorized_human",
                    "requiredEvidence": [],
                    "verificationRequirement": {
                        "schemaVersion": 1,
                        "requiredTier": "T0",
                        "requiredAssurance": "self_declared",
                        "policyRefs": ["local-node-timeout-fixture-v1"],
                        "stageRefs": ["task"],
                        "gateRefs": [],
                        "maxTimeoutSeconds": 30,
                        "reason": "Bound explicit local-node fixture deadlines to 30 seconds"
                    }
                }]
            }
        }))
        .expect("encode fixture timeout policy"),
    )
    .expect("write fixture timeout policy");
}

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn local_node_reuse_requires_real_new_fact_parent_and_exact_current_identity() {
    let directory = attached_repository();
    let root = directory.path();
    let counter_dir = tempfile::tempdir().unwrap();
    let counter = counter_dir.path().join("counter");
    let quoted = format!("'{}'", counter.display().to_string().replace('\'', "'\\''"));
    let script = format!("printf x >> {quoted}; printf local-node-real-output");
    let mut command = checkpoint_command(root, &script);
    command.timeout_seconds = Some(30);
    command.policy = RepositoryVerificationPolicy::ProfileAuthorized;
    cockpit_repository::confirm_profile_update(root, &command.program, &command.args).unwrap();
    cockpit_repository::confirm_profile_update(
        root,
        "sh",
        &["-c".into(), "printf actual-gap-output".into()],
    )
    .unwrap();
    authorize_local_node_fixture_timeout(root);
    start_attempt(root);
    preflight_work_item(
        root,
        &root.join(".ai/work-items/active/WI-ATTEMPT.contract.json"),
    )
    .unwrap();
    checkpoint_work_item(root, "WI-ATTEMPT").unwrap();
    let run = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    )
    .expect("real node execution");
    assert_eq!(run.receipt.processes_spawned, 1);
    assert!(run.receipt.passed);
    assert_eq!(fs::read(&counter).unwrap(), b"x");
    let before_parent = cockpit_repository::plan_local_node_reuse(
        root,
        std::slice::from_ref(&command),
        &run.final_snapshot,
        &runtime(),
    )
    .unwrap();
    assert!(
        before_parent[0].candidate.is_none(),
        "standalone fact is not a successful parent attempt"
    );
    persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        std::slice::from_ref(&command),
        &run.final_snapshot,
        &runtime(),
        "execution_completed",
        None,
        Some(&serde_json::to_value(&run.receipt).unwrap()),
    )
    .unwrap();
    let plan = cockpit_repository::plan_local_node_reuse(
        root,
        std::slice::from_ref(&command),
        &run.final_snapshot,
        &runtime(),
    )
    .unwrap();
    let candidate = plan[0]
        .candidate
        .as_ref()
        .expect("new real complete input fact plus parent")
        .clone();
    let reused = cockpit_repository::reuse_local_node_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        &candidate,
    )
    .unwrap()
    .expect("exact current native node");
    assert_eq!(
        fs::read(&counter).unwrap(),
        b"x",
        "reuse must not physically run the command a second time"
    );
    assert_eq!(reused.receipt.processes_spawned, 0);
    assert_eq!(reused.receipt.nodes_reused, 1);
    assert!(reused.receipt.execution_records.is_empty());
    assert_eq!(
        reused.receipt.results[0].output_digest,
        run.receipt.results[0].output_digest
    );
    let missing = RepositoryVerificationRequest {
        node_id: "project-command-1".into(),
        args: vec!["-c".into(), "printf actual-gap-output".into()],
        ..command.clone()
    };
    let partial = cockpit_repository::plan_local_node_reuse(
        root,
        &[command.clone(), missing.clone()],
        &run.final_snapshot,
        &runtime(),
    )
    .unwrap();
    assert_eq!(partial.len(), 2);
    assert!(partial[0].candidate.is_some());
    assert!(partial[1].candidate.is_none());
    let real_gap = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &missing,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    )
    .unwrap();
    assert_eq!(real_gap.receipt.processes_spawned, 1);
    assert!(real_gap.receipt.passed);
    let restarted = cockpit_repository::plan_local_node_reuse(
        root,
        std::slice::from_ref(&command),
        &real_gap.final_snapshot,
        &runtime(),
    )
    .unwrap();
    assert_eq!(
        restarted[0].candidate.as_ref().unwrap().projection(),
        candidate.projection(),
        "reload from disk, not in-memory reuse state"
    );
    for changed in [
        RepositoryVerificationRequest {
            policy: RepositoryVerificationPolicy::NeverReuse,
            ..command.clone()
        },
        RepositoryVerificationRequest {
            timeout_seconds: Some(29),
            ..command.clone()
        },
        RepositoryVerificationRequest {
            args: vec!["-c".into(), "printf changed".into()],
            ..command.clone()
        },
        RepositoryVerificationRequest {
            node_id: "foreign-node".into(),
            ..command.clone()
        },
        RepositoryVerificationRequest {
            runner: "hosted".into(),
            ..command.clone()
        },
        RepositoryVerificationRequest {
            scope: vec!["different/**".into()],
            ..command.clone()
        },
        RepositoryVerificationRequest {
            runtime_digest: Digest::sha256_bytes(b"other-runtime").to_string(),
            ..command.clone()
        },
    ] {
        let p = cockpit_repository::plan_local_node_reuse(
            root,
            &[changed],
            &run.final_snapshot,
            &runtime(),
        )
        .unwrap();
        assert!(p[0].candidate.is_none());
    }
    let failed = checkpoint_command(root, "printf failed-terminal; exit 7");
    let failed_run = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &failed,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    )
    .unwrap();
    assert!(!failed_run.receipt.passed);
    persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        &[failed],
        &failed_run.final_snapshot,
        &runtime(),
        "execution_failed",
        None,
        Some(&serde_json::to_value(&failed_run.receipt).unwrap()),
    )
    .unwrap();
    assert!(
        cockpit_repository::require_local_node_candidates_current(
            root,
            &[command],
            &failed_run.final_snapshot,
            &runtime(),
            &[candidate]
        )
        .is_err(),
        "late failure cannot fall back to old success"
    );
}

// Added only to repository/tests/verification_attempts.rs by the proposal builder.
// Every success baseline below comes from a real native executor and parent writer.
// Mutations operate only on the test's own disposable files and must reject reuse.
#[cfg(unix)]
#[test]
fn local_node_fact_and_parent_mutations_never_authorize_reuse() {
    let directory = attached_repository();
    let root = directory.path();
    let mut command = checkpoint_command(root, "printf mutation-baseline");
    command.timeout_seconds = Some(30);
    command.policy = RepositoryVerificationPolicy::ProfileAuthorized;
    cockpit_repository::confirm_profile_update(root, &command.program, &command.args).unwrap();
    cockpit_repository::confirm_profile_update(
        root,
        "sh",
        &["-c".into(), "printf actual-gap-output".into()],
    )
    .unwrap();
    authorize_local_node_fixture_timeout(root);
    start_attempt(root);
    preflight_work_item(
        root,
        &root.join(".ai/work-items/active/WI-ATTEMPT.contract.json"),
    )
    .unwrap();
    checkpoint_work_item(root, "WI-ATTEMPT").unwrap();
    let run = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    )
    .unwrap();
    assert!(run.receipt.passed);
    assert_eq!(run.receipt.processes_spawned, 1);
    let saved = persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        std::slice::from_ref(&command),
        &run.final_snapshot,
        &runtime(),
        "execution_completed",
        None,
        Some(&serde_json::to_value(&run.receipt).unwrap()),
    )
    .unwrap();
    let parent_path = root.join(saved["path"].as_str().unwrap());
    let parent_original: serde_json::Value =
        serde_json::from_slice(&fs::read(&parent_path).unwrap()).unwrap();
    let fact_dir = root.join(".ai/evidence/WI-ATTEMPT.verification-admissions/local-node-facts");
    let facts = fs::read_dir(&fact_dir)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(facts.len(), 1);
    let fact_path = facts[0].path();
    let fact_original: serde_json::Value =
        serde_json::from_slice(&fs::read(&fact_path).unwrap()).unwrap();
    let exact = cockpit_repository::plan_local_node_reuse(
        root,
        std::slice::from_ref(&command),
        &run.final_snapshot,
        &runtime(),
    )
    .unwrap();
    assert!(
        exact[0].candidate.is_some(),
        "real exact baseline before adversarial mutations"
    );
    let mutations = vec![
        ("/schemaVersion", json!(2)),
        ("/identity/workItemId", json!("WI-FOREIGN")),
        ("/identity/repositoryId", json!("foreign")),
        (
            "/identity/contractDigest",
            json!(Digest::sha256_bytes(b"foreign-contract")),
        ),
        (
            "/identity/contractExecutionScopeDigest",
            json!(Digest::sha256_bytes(b"foreign-scope")),
        ),
        (
            "/identity/repositorySnapshotDigest",
            json!(Digest::sha256_bytes(b"foreign-source")),
        ),
        (
            "/identity/declaredInventoryDigest",
            json!(Digest::sha256_bytes(b"foreign-inventory")),
        ),
        (
            "/identity/runtimeDigest",
            json!(Digest::sha256_bytes(b"foreign-runtime")),
        ),
        ("/identity/runtimeVersion", json!("other")),
        ("/identity/requestRuntimeDigest", json!("other")),
        (
            "/identity/policyDigest",
            json!(Digest::sha256_bytes(b"other-policy")),
        ),
        ("/identity/nodeId", json!("other-node")),
        ("/identity/program", json!("other-program")),
        ("/identity/args", json!(["other-argument"])),
        ("/identity/scope", json!(["other/**"])),
        ("/identity/stage", json!("release")),
        ("/identity/runner", json!("hosted")),
        ("/identity/baseCommit", json!("other-base")),
        (
            "/identity/executableDigest",
            json!(Digest::sha256_bytes(b"other-executable")),
        ),
        ("/identity/executablePath", json!("/other/executable")),
        ("/identity/assurance", json!("provider_verified")),
        ("/identity/requestPolicy", json!("never_reuse")),
        ("/identity/effectivePolicy", json!("never_reuse")),
        ("/identity/profileAuthorization", serde_json::Value::Null),
        (
            "/identity/profileAuthorization/profileDigest",
            json!(Digest::sha256_bytes(b"other-profile")),
        ),
        (
            "/identity/profileAuthorization/profileFileDigest",
            json!(Digest::sha256_bytes(b"other-profile-bytes")),
        ),
        (
            "/identity/profileAuthorization/configFileDigest",
            json!(Digest::sha256_bytes(b"other-config")),
        ),
        ("/identity/producer", json!("artifact_self_report")),
        ("/identity/executionInput/schemaVersion", json!(2)),
        ("/identity/executionInput/platformOs", json!("other-os")),
        ("/identity/executionInput/platformArch", json!("other-arch")),
        ("/identity/executionInput/program", json!("other-program")),
        ("/identity/executionInput/args", json!(["other"])),
        (
            "/identity/executionInput/cwdBytes",
            json!([47, 111, 116, 104, 101, 114]),
        ),
        ("/identity/executionInput/resourceWeight", json!(2)),
        ("/identity/executionInput/clearEnvironment", json!(true)),
        ("/identity/executionInput/timeoutSeconds", json!(29)),
        ("/identity/executionInput/environmentComplete", json!(false)),
        (
            "/identity/executionInput/environmentDigest",
            json!(Digest::sha256_bytes(b"other-full-environment")),
        ),
        (
            "/identity/executionInput/environmentKeysDigest",
            json!(Digest::sha256_bytes(b"other-key-order")),
        ),
        (
            "/identity/executionInput/commandDigest",
            json!(Digest::sha256_bytes(b"other-command")),
        ),
        (
            "/identity/executionInput/dependencies",
            json!(["same-upstream-id"]),
        ),
        ("/postExecutionIdentityValidated", json!(false)),
        ("/parentAttemptRequired", json!(false)),
        ("/executionRecord/spawned", json!(false)),
        ("/executionRecord/passed", json!(false)),
        ("/executionRecord/exitCode", json!(7)),
        ("/executionRecord/timedOut", json!(true)),
        ("/executionRecord/stdoutTruncated", json!(true)),
        ("/executionRecord/stderrTruncated", json!(true)),
    ];
    fn write_owned(
        value: &mut serde_json::Value,
        key: &str,
        parent: &Path,
        prefix: &str,
    ) -> PathBuf {
        value.as_object_mut().unwrap().remove(key);
        let id = digest_json(value).unwrap().to_string();
        value[key] = json!(id);
        let path = parent.join(format!("{prefix}{}.json", id.trim_start_matches("sha256:")));
        fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
        path
    }
    for (pointer, new_value) in mutations {
        let mut fact = fact_original.clone();
        let mut parent = parent_original.clone();
        *fact
            .pointer_mut(pointer)
            .unwrap_or_else(|| panic!("missing native mutation field {pointer}")) =
            new_value.clone();
        if let Some(field) = pointer.strip_prefix("/executionRecord/") {
            parent["executionRecords"][0][field] = new_value.clone();
            parent["receipt"]["executionRecords"][0][field] = new_value;
        }
        fs::remove_file(&fact_path).unwrap();
        fs::remove_file(&parent_path).unwrap();
        let changed_fact = write_owned(&mut fact, "factId", &fact_dir, "");
        parent["nodeExecutionFacts"][0]["factId"] = fact["factId"].clone();
        let changed_parent = write_owned(
            &mut parent,
            "attemptId",
            parent_path.parent().unwrap(),
            "WI-ATTEMPT.verification-attempt.",
        );
        let plan = cockpit_repository::plan_local_node_reuse(
            root,
            std::slice::from_ref(&command),
            &run.final_snapshot,
            &runtime(),
        )
        .unwrap();
        assert!(
            plan[0].candidate.is_none(),
            "must not accept mutation {pointer}"
        );
        fs::remove_file(changed_fact).unwrap();
        fs::remove_file(changed_parent).unwrap();
        fs::write(
            &fact_path,
            serde_json::to_vec_pretty(&fact_original).unwrap(),
        )
        .unwrap();
        fs::write(
            &parent_path,
            serde_json::to_vec_pretty(&parent_original).unwrap(),
        )
        .unwrap();
    }
    // Raw corruption and nonregular input must fail closed, not fall back to a
    // previous complete-looking record. The real reader must remain nonblocking.
    let bytes = fs::read(&fact_path).unwrap();
    fs::write(&fact_path, b"{not-json").unwrap();
    assert!(
        cockpit_repository::plan_local_node_reuse(
            root,
            std::slice::from_ref(&command),
            &run.final_snapshot,
            &runtime()
        )
        .unwrap()[0]
            .candidate
            .is_none()
    );
    fs::remove_file(&fact_path).unwrap();
    use std::os::unix::ffi::OsStrExt;
    let fifo = std::ffi::CString::new(fact_path.as_os_str().as_bytes()).unwrap();
    // SAFETY: One FIFO in this private test-owned directory, no foreign PID or file.
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    assert!(
        cockpit_repository::plan_local_node_reuse(
            root,
            std::slice::from_ref(&command),
            &run.final_snapshot,
            &runtime()
        )
        .unwrap()[0]
            .candidate
            .is_none()
    );
    fs::remove_file(&fact_path).unwrap();
    fs::write(&fact_path, bytes).unwrap();
}

#[cfg(unix)]
#[cfg(not(target_os = "macos"))]
fn local_now_epoch_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .try_into()
        .unwrap()
}

#[cfg(unix)]
fn local_authorized_fixture(commands: &[RepositoryVerificationRequest], root: &Path) {
    for command in commands {
        cockpit_repository::confirm_profile_update(root, &command.program, &command.args).unwrap();
    }
    start_attempt(root);
    preflight_work_item(
        root,
        &root.join(".ai/work-items/active/WI-ATTEMPT.contract.json"),
    )
    .unwrap();
    checkpoint_work_item(root, "WI-ATTEMPT").unwrap();
}

#[cfg(unix)]
fn local_real_success(
    root: &Path,
    command: &RepositoryVerificationRequest,
) -> cockpit_repository::RepositoryVerificationRun {
    let result = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        command,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    )
    .unwrap();
    assert!(result.receipt.passed);
    assert_eq!(result.receipt.processes_spawned, 1);
    persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        std::slice::from_ref(command),
        &result.final_snapshot,
        &runtime(),
        "execution_completed",
        None,
        Some(&serde_json::to_value(&result.receipt).unwrap()),
    )
    .unwrap();
    assert!(
        cockpit_repository::plan_local_node_reuse(
            root,
            std::slice::from_ref(command),
            &result.final_snapshot,
            &runtime()
        )
        .unwrap()[0]
            .candidate
            .is_some()
    );
    result
}

#[cfg(unix)]
#[test]
fn optional_corrupt_or_fifo_facts_cannot_erase_native_terminal_attempts() {
    let directory = repository();
    let root = directory.path();
    let command = checkpoint_command(root, "printf real-terminal-output");
    preflight_work_item(
        root,
        &root.join(".ai/work-items/active/WI-ATTEMPT.contract.json"),
    )
    .unwrap();
    checkpoint_work_item(root, "WI-ATTEMPT").unwrap();
    let result = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    )
    .unwrap();
    assert!(result.receipt.passed);
    let receipt = serde_json::to_value(&result.receipt).unwrap();
    let facts = root.join(".ai/evidence/WI-ATTEMPT.verification-admissions/local-node-facts");
    fs::create_dir_all(&facts).unwrap();
    let bad = facts.join("broken.json");
    for fifo in [false, true] {
        if fifo {
            use std::os::unix::ffi::OsStrExt;
            let path = std::ffi::CString::new(bad.as_os_str().as_bytes()).unwrap();
            // SAFETY: Test-owned path; the production no-follow nonblocking reader must reject it.
            assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
        } else {
            fs::write(&bad, b"{invalid-json").unwrap();
        }
        for state in [
            "execution_failed",
            "precondition_rejected",
            "formal_receipt_rejected",
            "execution_completed",
        ] {
            let saved = persist_verification_attempt(
                root,
                "WI-ATTEMPT",
                std::slice::from_ref(&command),
                &result.final_snapshot,
                &runtime(),
                state,
                Some(("native_terminal", state)),
                Some(&receipt),
            )
            .expect("optional reuse cache must not block terminal append");
            let stored: serde_json::Value = serde_json::from_slice(
                &fs::read(root.join(saved["path"].as_str().unwrap())).unwrap(),
            )
            .unwrap();
            assert_eq!(stored["state"], state);
            assert_eq!(
                stored["receipt"]["executionRecords"],
                receipt["executionRecords"]
            );
            assert_eq!(
                stored["localNodeLinkState"],
                if state == "execution_completed" {
                    "unavailable"
                } else {
                    "not_applicable"
                }
            );
        }
        assert!(
            cockpit_repository::plan_local_node_reuse(
                root,
                std::slice::from_ref(&command),
                &result.final_snapshot,
                &runtime()
            )
            .unwrap()[0]
                .candidate
                .is_none()
        );
        fs::remove_file(&bad).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn prepared_real_execution_masks_old_success_before_any_terminal_exists() {
    use std::sync::{Arc, Mutex, mpsc};
    use std::time::Duration;
    let directory = attached_repository();
    let root = directory.path();
    let mut command = checkpoint_command(root, "printf bounded-pending-child");
    command.policy = RepositoryVerificationPolicy::ProfileAuthorized;
    local_authorized_fixture(std::slice::from_ref(&command), root);
    let old = local_real_success(root, &command);
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let release_rx = Arc::new(Mutex::new(release_rx));
    std::thread::scope(|scope| {
        let root_owned = root.to_owned();
        let mut forced = command.clone();
        forced.policy = RepositoryVerificationPolicy::NeverReuse;
        let worker = scope.spawn(move || {
            cockpit_repository::run_repository_verification_with_task_authorization(
                &root_owned,
                &forced,
                &runtime(),
                None,
                move |_, _, started| {
                    if started {
                        started_tx.send(()).map_err(|e| e.to_string())?;
                        release_rx
                            .lock()
                            .map_err(|e| e.to_string())?
                            .recv_timeout(Duration::from_secs(15))
                            .map_err(|e| e.to_string())?;
                    }
                    Ok(())
                },
            )
        });
        started_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("real executor start callback");
        let plan = cockpit_repository::plan_local_node_reuse(
            root,
            std::slice::from_ref(&command),
            &old.final_snapshot,
            &runtime(),
        )
        .unwrap();
        // Release before asserting so a failed assertion cannot strand the owned worker.
        release_tx.send(()).unwrap();
        let fresh = worker.join().unwrap().unwrap();
        assert_eq!(fresh.receipt.processes_spawned, 1);
        assert!(fresh.receipt.passed);
        assert!(
            plan[0].candidate.is_none(),
            "prepared actual execution without a parent terminal shadows old success"
        );
        assert!(
            checkpoint_values(root)
                .iter()
                .any(|value| value["state"] == "execution_prepared")
        );
    });
}

#[cfg(unix)]
#[test]
fn checkpoint_only_exit_or_post_identity_failure_masks_old_success_without_timestamp_order() {
    for exit in [0, 7] {
        let directory = attached_repository();
        let root = directory.path();
        let control = tempfile::tempdir().unwrap();
        let mode = control.path().join("fail-mode");
        let quoted = format!("'{}'", mode.display().to_string().replace('\'', "'\\''"));
        let script = format!(
            "if test -f {quoted}; then mv .git .git-retained; printf checkpoint-only-output; printf checkpoint-only-error >&2; exit {exit}; fi; printf exact-old-success"
        );
        let mut command = checkpoint_command(root, &script);
        command.policy = RepositoryVerificationPolicy::ProfileAuthorized;
        local_authorized_fixture(std::slice::from_ref(&command), root);
        let old = local_real_success(root, &command);
        fs::write(&mode, b"actual fault now").unwrap();
        let mut forced = command.clone();
        forced.policy = RepositoryVerificationPolicy::NeverReuse;
        let result = cockpit_repository::run_repository_verification_with_task_authorization(
            root,
            &forced,
            &runtime(),
            None,
            |_, _, _| Ok(()),
        );
        assert!(
            root.join(".git-retained").is_dir(),
            "the real command, not a fabricated checkpoint, must run"
        );
        // Restore only this fixture's own Git directory before querying the planner.
        fs::rename(root.join(".git-retained"), root.join(".git")).unwrap();
        assert!(
            result.is_err(),
            "post-execution snapshot must reject missing Git"
        );
        let values = checkpoint_values(root);
        let completed = values
            .iter()
            .find(|value| {
                value["state"] == "node_execution_completed"
                    && value["executionRecords"][0]["stdout"]
                        == "636865636b706f696e742d6f6e6c792d6f7574707574"
            })
            .expect("native raw child checkpoint");
        assert_eq!(completed["executionRecords"][0]["exitCode"], exit);
        let stderr = completed["executionRecords"][0]["stderr"]
            .as_str()
            .expect("recorded stderr hex");
        #[cfg(target_os = "macos")]
        {
            let stderr_bytes = (0..stderr.len())
                .step_by(2)
                .map(|offset| u8::from_str_radix(&stderr[offset..offset + 2], 16).unwrap())
                .collect::<Vec<_>>();
            let stderr_text = String::from_utf8(stderr_bytes).expect("Mac stderr text");
            assert!(stderr_text.starts_with("checkpoint-only-error"));
            assert!(stderr_text.contains("mac_original_group_signal:"));
            assert!(stderr_text.contains("direct_child_reap:"));
        }
        #[cfg(not(target_os = "macos"))]
        assert_eq!(stderr, "636865636b706f696e742d6f6c792d6572726f72");
        assert_eq!(
            values
                .iter()
                .filter(|v| v["kind"] == "verification_attempt")
                .count(),
            1,
            "no terminal was manufactured for the failed owner boundary"
        );
        assert!(
            cockpit_repository::plan_local_node_reuse(
                root,
                std::slice::from_ref(&command),
                &old.final_snapshot,
                &runtime()
            )
            .unwrap()[0]
                .candidate
                .is_none()
        );
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn prepared_owner_run_requires_causal_completion_and_interruption_is_never_exempt() {
    let directory = attached_repository();
    let root = directory.path();
    let mut command = checkpoint_command(root, "printf causal-owner");
    command.policy = RepositoryVerificationPolicy::ProfileAuthorized;
    local_authorized_fixture(std::slice::from_ref(&command), root);
    let old = local_real_success(root, &command);
    let decision = cockpit_repository::plan_local_node_reuse(
        root,
        std::slice::from_ref(&command),
        &old.final_snapshot,
        &runtime(),
    )
    .unwrap();
    let candidate = decision[0].candidate.as_ref().unwrap();
    let plan = plan_repository_verification_action(
        root,
        &command,
        &old.final_snapshot,
        local_now_epoch_seconds(),
    )
    .unwrap();
    let checkpoint = cockpit_repository::VerificationRunCheckpoint::prepare(
        root,
        std::slice::from_ref(&command),
        &[plan],
        &old.final_snapshot,
        &runtime(),
    )
    .unwrap();
    assert!(
        cockpit_repository::plan_local_node_reuse(
            root,
            std::slice::from_ref(&command),
            &old.final_snapshot,
            &runtime()
        )
        .unwrap()[0]
            .candidate
            .is_none()
    );
    let identity = checkpoint.local_run_identity();
    let reused = cockpit_repository::reuse_local_node_in_run_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        candidate,
        Some(&identity),
    )
    .unwrap()
    .expect("only this native prepared run can exempt itself");
    assert_eq!(reused.receipt.nodes_reused, 1);
    assert_eq!(reused.receipt.processes_spawned, 0);
    checkpoint
        .record_interrupted(
            &[&reused.receipt],
            &[],
            &[],
            "local_node_preconditions",
            "actual owner finalization refused",
        )
        .unwrap();
    assert!(
        cockpit_repository::require_local_node_candidates_current_in_run(
            root,
            std::slice::from_ref(&command),
            &old.final_snapshot,
            &runtime(),
            std::slice::from_ref(candidate),
            Some(&identity)
        )
        .is_err()
    );
    assert!(
        checkpoint_values(root)
            .iter()
            .any(|v| v["state"] == "interrupted"
                && v["diagnostic"]["phase"] == "local_node_preconditions")
    );
}

#[cfg(unix)]
#[test]
fn source_never_reuse_and_profile_denied_execution_cannot_become_local_facts() {
    for denied in [false, true] {
        let directory = attached_repository();
        let root = directory.path();
        let mut command = checkpoint_command(root, "printf policy-source");
        if denied {
            command.policy = RepositoryVerificationPolicy::ProfileAuthorized;
        } else {
            cockpit_repository::confirm_profile_update(root, &command.program, &command.args)
                .unwrap();
        }
        start_attempt(root);
        preflight_work_item(
            root,
            &root.join(".ai/work-items/active/WI-ATTEMPT.contract.json"),
        )
        .unwrap();
        checkpoint_work_item(root, "WI-ATTEMPT").unwrap();
        let result = cockpit_repository::run_repository_verification_with_task_authorization(
            root,
            &command,
            &runtime(),
            None,
            |_, _, _| Ok(()),
        )
        .unwrap();
        assert!(result.receipt.passed);
        assert_eq!(result.receipt.processes_spawned, 1);
        persist_verification_attempt(
            root,
            "WI-ATTEMPT",
            std::slice::from_ref(&command),
            &result.final_snapshot,
            &runtime(),
            "execution_completed",
            None,
            Some(&serde_json::to_value(&result.receipt).unwrap()),
        )
        .unwrap();
        let facts = root.join(".ai/evidence/WI-ATTEMPT.verification-admissions/local-node-facts");
        assert!(
            !facts.exists() || fs::read_dir(&facts).unwrap().next().is_none(),
            "actual source effective NeverReuse must not publish a reusable proof"
        );
        command.policy = RepositoryVerificationPolicy::ProfileAuthorized;
        if denied {
            cockpit_repository::confirm_profile_update(root, &command.program, &command.args)
                .unwrap();
        }
        let snapshot = GitRepository::discover(root).unwrap().snapshot().unwrap();
        assert!(
            cockpit_repository::plan_local_node_reuse(root, &[command], &snapshot, &runtime())
                .unwrap()[0]
                .candidate
                .is_none()
        );
    }
}

#[cfg(unix)]
#[test]
fn current_never_protected_denied_and_changed_profile_reject_saved_local_fact() {
    let directory = attached_repository();
    let root = directory.path();
    let mut command = checkpoint_command(root, "printf profile-exact");
    command.policy = RepositoryVerificationPolicy::ProfileAuthorized;
    local_authorized_fixture(std::slice::from_ref(&command), root);
    let old = local_real_success(root, &command);
    for policy in [
        RepositoryVerificationPolicy::NeverReuse,
        RepositoryVerificationPolicy::Protected(cockpit_verification::ProtectedGateClass::Security),
    ] {
        let changed = RepositoryVerificationRequest {
            policy,
            ..command.clone()
        };
        assert!(
            cockpit_repository::plan_local_node_reuse(
                root,
                &[changed],
                &old.final_snapshot,
                &runtime()
            )
            .unwrap()[0]
                .candidate
                .is_none()
        );
    }
    let project = root.join(".ai/project.json");
    let bytes = fs::read(&project).unwrap();
    let mut profile: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    profile["state"] = json!("uncalibrated");
    fs::write(&project, serde_json::to_vec_pretty(&profile).unwrap()).unwrap();
    let snapshot = GitRepository::discover(root).unwrap().snapshot().unwrap();
    assert!(
        cockpit_repository::plan_local_node_reuse(
            root,
            std::slice::from_ref(&command),
            &snapshot,
            &runtime()
        )
        .unwrap()[0]
            .candidate
            .is_none(),
        "current denied profile cannot use a previous authorized local fact"
    );
    fs::write(&project, bytes).unwrap();
    cockpit_repository::confirm_profile_update(
        root,
        "sh",
        &["-c".into(), "printf unrelated-profile-addition".into()],
    )
    .unwrap();
    let snapshot = GitRepository::discover(root).unwrap().snapshot().unwrap();
    assert!(
        cockpit_repository::plan_local_node_reuse(root, &[command], &snapshot, &runtime()).unwrap()
            [0]
        .candidate
        .is_none(),
        "profile context/file digest is part of identity even with same node argv"
    );
}

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn mixed_parent_reloads_its_new_physical_node_without_reusing_unknown_siblings() {
    let directory = attached_repository();
    let root = directory.path();
    let control = tempfile::tempdir().unwrap();
    let mode = control.path().join("later-sibling-failure");
    let quoted = format!("'{}'", mode.display().to_string().replace('\'', "'\\''"));
    let mut first = checkpoint_command(
        root,
        &format!("if test -f {quoted}; then exit 7; fi; printf mixed-first"),
    );
    first.policy = RepositoryVerificationPolicy::ProfileAuthorized;
    let second = RepositoryVerificationRequest {
        node_id: "project-command-1".into(),
        args: vec!["-c".into(), "printf mixed-second".into()],
        ..first.clone()
    };
    local_authorized_fixture(&[first.clone(), second.clone()], root);
    let baseline = local_real_success(root, &first);
    let planned = cockpit_repository::plan_local_node_reuse(
        root,
        std::slice::from_ref(&first),
        &baseline.final_snapshot,
        &runtime(),
    )
    .unwrap();
    let mut reused = cockpit_repository::reuse_local_node_with_task_authorization(
        root,
        &first,
        &runtime(),
        None,
        planned[0].candidate.as_ref().unwrap(),
    )
    .unwrap()
    .unwrap();
    let actual = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &second,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    )
    .unwrap();
    assert!(actual.receipt.passed);
    assert_eq!(actual.receipt.processes_spawned, 1);
    reused
        .receipt
        .results
        .extend(actual.receipt.results.clone());
    reused
        .receipt
        .execution_records
        .extend(actual.receipt.execution_records.clone());
    reused.receipt.nodes_planned += actual.receipt.nodes_planned;
    reused.receipt.nodes_executed += actual.receipt.nodes_executed;
    reused.receipt.processes_spawned += actual.receipt.processes_spawned;
    reused
        .receipt
        .receipt_candidates
        .extend(actual.receipt.receipt_candidates.clone());
    let parent = persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        &[first.clone(), second.clone()],
        &actual.final_snapshot,
        &runtime(),
        "execution_completed",
        None,
        Some(&serde_json::to_value(&reused.receipt).unwrap()),
    )
    .unwrap();
    drop(reused);
    drop(actual);
    let snapshot = GitRepository::discover(root).unwrap().snapshot().unwrap();
    let plan = cockpit_repository::plan_local_node_reuse(
        root,
        std::slice::from_ref(&second),
        &snapshot,
        &runtime(),
    )
    .unwrap();
    let candidate = plan[0]
        .candidate
        .as_ref()
        .expect("new physical B from mixed parent must survive disk reload");
    assert_eq!(
        candidate.projection()["parentAttemptId"],
        parent["attemptId"]
    );
    let result = cockpit_repository::reuse_local_node_with_task_authorization(
        root,
        &second,
        &runtime(),
        None,
        candidate,
    )
    .unwrap()
    .unwrap();
    assert!(result.receipt.passed);
    assert_eq!(result.receipt.nodes_reused, 1);
    assert_eq!(result.receipt.processes_spawned, 0);
    let parent_path = root.join(parent["path"].as_str().unwrap());
    let parent_bytes = fs::read(&parent_path).unwrap();
    let original_parent: serde_json::Value = serde_json::from_slice(&parent_bytes).unwrap();
    let b_fact = root
        .join(".ai/evidence/WI-ATTEMPT.verification-admissions/local-node-facts")
        .join(format!(
            "{}.json",
            candidate.projection()["nodeFactId"]
                .as_str()
                .unwrap()
                .trim_start_matches("sha256:")
        ));
    let b_fact_bytes = fs::read(&b_fact).unwrap();
    let mut invalid = original_parent.clone();
    invalid["receipt"]["results"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["nodeId"] == first.node_id)
        .unwrap()["receiptId"] =
        json!(Digest::sha256_bytes(b"valid-format-but-no-historical-fact"));
    invalid.as_object_mut().unwrap().remove("attemptId");
    let invalid_id = digest_json(&invalid).unwrap();
    invalid["attemptId"] = json!(invalid_id);
    let invalid_path = parent_path.parent().unwrap().join(format!(
        "WI-ATTEMPT.verification-attempt.{}.json",
        invalid_id.to_string().trim_start_matches("sha256:")
    ));
    fs::remove_file(&parent_path).unwrap();
    fs::write(&invalid_path, serde_json::to_vec_pretty(&invalid).unwrap()).unwrap();
    assert!(
        cockpit_repository::plan_local_node_reuse(
            root,
            std::slice::from_ref(&second),
            &snapshot,
            &runtime()
        )
        .unwrap()[0]
            .candidate
            .is_none(),
        "a well-formed receiptId without its original proof cannot justify a reused sibling"
    );
    assert_eq!(fs::read(&b_fact).unwrap(), b_fact_bytes);
    assert_eq!(
        invalid["executionRecords"],
        original_parent["executionRecords"]
    );
    fs::remove_file(invalid_path).unwrap();
    fs::write(&parent_path, &parent_bytes).unwrap();
    let a_binding = planned[0].candidate.as_ref().unwrap().projection();
    let a_parent = root.join(".ai/evidence").join(format!(
        "WI-ATTEMPT.verification-attempt.{}.json",
        a_binding["parentAttemptId"]
            .as_str()
            .unwrap()
            .trim_start_matches("sha256:")
    ));
    let a_parent_bytes = fs::read(&a_parent).unwrap();
    fs::remove_file(&a_parent).unwrap();
    assert!(
        cockpit_repository::plan_local_node_reuse(
            root,
            std::slice::from_ref(&second),
            &snapshot,
            &runtime()
        )
        .unwrap()[0]
            .candidate
            .is_none(),
        "an orphan historical fact is not a successful original parent chain"
    );
    assert_eq!(fs::read(&b_fact).unwrap(), b_fact_bytes);
    assert_eq!(
        fs::read(&parent_path).unwrap(),
        parent_bytes,
        "B's real record is retained despite its mixed parent being unavailable for reuse"
    );
    fs::write(a_parent, a_parent_bytes).unwrap();
    fs::write(&mode, b"unrelated current sibling can fail").unwrap();
    let failed_request = RepositoryVerificationRequest {
        policy: RepositoryVerificationPolicy::NeverReuse,
        ..first
    };
    let failed = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &failed_request,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    )
    .unwrap();
    assert!(!failed.receipt.passed);
    assert_eq!(failed.receipt.execution_records[0].exit_code, Some(7));
    persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        &[failed_request],
        &failed.final_snapshot,
        &runtime(),
        "execution_failed",
        None,
        Some(&serde_json::to_value(&failed.receipt).unwrap()),
    )
    .unwrap();
    assert!(
        cockpit_repository::plan_local_node_reuse(
            root,
            std::slice::from_ref(&second),
            &failed.final_snapshot,
            &runtime()
        )
        .unwrap()[0]
            .candidate
            .is_some(),
        "historical sibling provenance does not require an independent sibling to remain currently successful forever"
    );
}

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn run_terminal_link_is_required_and_checkpoint_timestamps_cannot_replace_it() {
    let directory = attached_repository();
    let root = directory.path();
    let mut command = checkpoint_command(root, "printf run-terminal-causal-link");
    command.policy = RepositoryVerificationPolicy::ProfileAuthorized;
    local_authorized_fixture(std::slice::from_ref(&command), root);
    let initial = GitRepository::discover(root).unwrap().snapshot().unwrap();
    let planned =
        plan_repository_verification_action(root, &command, &initial, local_now_epoch_seconds())
            .unwrap();
    let checkpoint = cockpit_repository::VerificationRunCheckpoint::prepare(
        root,
        std::slice::from_ref(&command),
        &[planned],
        &initial,
        &runtime(),
    )
    .unwrap();
    let actual = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    )
    .unwrap();
    assert!(actual.receipt.passed);
    assert_eq!(actual.receipt.processes_spawned, 1);
    let parent = checkpoint
        .persist_local_run_attempt(
            std::slice::from_ref(&command),
            &actual.final_snapshot,
            &runtime(),
            cockpit_repository::LocalRunAttemptWrite {
                state: "execution_completed",
                diagnostic: None,
                receipt: Some(&serde_json::to_value(&actual.receipt).unwrap()),
                supersedes: None,
            },
        )
        .unwrap();
    assert!(
        cockpit_repository::plan_local_node_reuse(
            root,
            std::slice::from_ref(&command),
            &actual.final_snapshot,
            &runtime()
        )
        .unwrap()[0]
            .candidate
            .is_none(),
        "execution parent is not owner finalization"
    );
    let completed = checkpoint.record_local_run_completion(&parent).unwrap();
    assert_eq!(completed["parentAttemptId"], parent["attemptId"]);
    // Rehash only test-owned checkpoint timestamps; runId, executionId, parent
    // and physical-record associations remain exactly the native observations.
    for entry in fs::read_dir(root.join(".ai/evidence"))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
    {
        if !entry
            .file_name()
            .to_string_lossy()
            .starts_with("WI-ATTEMPT.verification-attempt.")
        {
            continue;
        }
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(entry.path()).unwrap()).unwrap();
        if value["kind"] == "verification_attempt" {
            continue;
        }
        value["createdAt"] = json!(if value["state"] == "run_prepared" {
            "2999-01-01T00:00:00Z"
        } else {
            "1970-01-01T00:00:00Z"
        });
        value.as_object_mut().unwrap().remove("checkpointId");
        let id = digest_json(&value).unwrap();
        value["checkpointId"] = json!(id);
        fs::remove_file(entry.path()).unwrap();
        fs::write(
            root.join(".ai/evidence").join(format!(
                "WI-ATTEMPT.verification-attempt.{}.json",
                id.to_string().trim_start_matches("sha256:")
            )),
            serde_json::to_vec_pretty(&value).unwrap(),
        )
        .unwrap();
    }
    let plan = cockpit_repository::plan_local_node_reuse(
        root,
        std::slice::from_ref(&command),
        &actual.final_snapshot,
        &runtime(),
    )
    .unwrap();
    assert!(
        plan[0].candidate.is_some(),
        "IDs and terminal association decide reuse, not wall-clock order"
    );
    let result = cockpit_repository::reuse_local_node_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        plan[0].candidate.as_ref().unwrap(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(result.receipt.processes_spawned, 0);
}

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn missing_native_execution_checkpoint_pair_rejects_fact_parent_and_old_profile_fallback() {
    for missing in ["execution_prepared", "node_execution_completed", "both"] {
        let directory = attached_repository();
        let root = directory.path();
        let mut command = checkpoint_command(root, "printf checkpoint-pair-required");
        command.policy = RepositoryVerificationPolicy::ProfileAuthorized;
        local_authorized_fixture(std::slice::from_ref(&command), root);
        let actual = local_real_success(root, &command);
        let legacy = plan_repository_verification_action(
            root,
            &command,
            &actual.final_snapshot,
            local_now_epoch_seconds(),
        )
        .unwrap();
        assert_eq!(
            legacy["action"], "reuse",
            "the old profile receipt must really exist"
        );
        let plan = cockpit_repository::plan_local_node_reuse(
            root,
            std::slice::from_ref(&command),
            &actual.final_snapshot,
            &runtime(),
        )
        .unwrap();
        let binding = plan[0].candidate.as_ref().unwrap().projection();
        let fact = root
            .join(".ai/evidence/WI-ATTEMPT.verification-admissions/local-node-facts")
            .join(format!(
                "{}.json",
                binding["nodeFactId"]
                    .as_str()
                    .unwrap()
                    .trim_start_matches("sha256:")
            ));
        let fact_bytes = fs::read(&fact).unwrap();
        let parent = root.join(".ai/evidence").join(format!(
            "WI-ATTEMPT.verification-attempt.{}.json",
            binding["parentAttemptId"]
                .as_str()
                .unwrap()
                .trim_start_matches("sha256:")
        ));
        let parent_bytes = fs::read(&parent).unwrap();
        let mut removed = 0;
        for entry in fs::read_dir(root.join(".ai/evidence"))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
        {
            if !entry
                .file_name()
                .to_string_lossy()
                .starts_with("WI-ATTEMPT.verification-attempt.")
            {
                continue;
            }
            let value: serde_json::Value =
                serde_json::from_slice(&fs::read(entry.path()).unwrap()).unwrap();
            if value["kind"] == "verification_execution_checkpoint"
                && value["executionId"] == binding["executionId"]
                && (missing == "both" || value["state"] == missing)
            {
                fs::remove_file(entry.path()).unwrap();
                removed += 1;
            }
        }
        assert_eq!(removed, if missing == "both" { 2 } else { 1 });
        let refused = cockpit_repository::plan_local_node_reuse(
            root,
            std::slice::from_ref(&command),
            &actual.final_snapshot,
            &runtime(),
        )
        .unwrap();
        assert!(refused[0].candidate.is_none());
        assert!(refused[0].fresh_required);
        assert_eq!(fs::read(&fact).unwrap(), fact_bytes);
        assert_eq!(
            fs::read(&parent).unwrap(),
            parent_bytes,
            "missing proof must not erase the real passed parent"
        );
        let fresh = cockpit_repository::run_repository_verification_with_task_authorization(
            root,
            &command,
            &runtime(),
            None,
            |_, _, _| Ok(()),
        )
        .unwrap();
        assert!(fresh.receipt.passed);
        assert_eq!(
            fresh.receipt.processes_spawned, 1,
            "the other profile cache must not bypass mandatory proof refusal"
        );
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn causal_failure_blocks_real_legacy_profile_cache_in_direct_and_invalidated_consumer_routes() {
    for route in [
        "task_context",
        "consumer_invalidated",
        "legacy_without_runtime",
    ] {
        let directory = attached_repository();
        let root = directory.path();
        let control = tempfile::tempdir().unwrap();
        let mode = control.path().join("fail-once");
        let counter = control.path().join("count");
        let quote = |p: &Path| format!("'{}'", p.display().to_string().replace('\'', "'\\''"));
        let script = format!(
            "printf x >> {}; if test -f {}; then rm {}; printf real-failure; exit 7; fi; printf cache-fallback-success",
            quote(&counter),
            quote(&mode),
            quote(&mode)
        );
        let mut command = checkpoint_command(root, &script);
        command.policy = RepositoryVerificationPolicy::ProfileAuthorized;
        local_authorized_fixture(std::slice::from_ref(&command), root);
        let baseline = local_real_success(root, &command);
        let legacy = plan_repository_verification_action(
            root,
            &command,
            &baseline.final_snapshot,
            local_now_epoch_seconds(),
        )
        .unwrap();
        assert_eq!(legacy["action"], "reuse");
        assert!(legacy["receiptId"].as_str().is_some());
        let planned = cockpit_repository::plan_local_node_reuse(
            root,
            std::slice::from_ref(&command),
            &baseline.final_snapshot,
            &runtime(),
        )
        .unwrap();
        let candidate = planned[0].candidate.as_ref().unwrap();
        fs::write(&mode, b"fail only the new actual invocation").unwrap();
        let forced = RepositoryVerificationRequest {
            policy: RepositoryVerificationPolicy::NeverReuse,
            ..command.clone()
        };
        let failed = cockpit_repository::run_repository_verification_with_task_authorization(
            root,
            &forced,
            &runtime(),
            None,
            |_, _, _| Ok(()),
        )
        .unwrap();
        assert!(!failed.receipt.passed);
        assert_eq!(failed.receipt.processes_spawned, 1);
        assert_eq!(failed.receipt.execution_records[0].exit_code, Some(7));
        // Deliberately no parent terminal: the new native checkpoint alone is
        // sufficient, with the old successful profile cache still untouched.
        let current = cockpit_repository::plan_local_node_reuse(
            root,
            std::slice::from_ref(&command),
            &failed.final_snapshot,
            &runtime(),
        )
        .unwrap();
        assert!(current[0].candidate.is_none());
        assert!(current[0].fresh_required);
        assert_eq!(
            plan_repository_verification_action(
                root,
                &command,
                &failed.final_snapshot,
                local_now_epoch_seconds()
            )
            .unwrap()["action"],
            "reuse",
            "the legacy candidate remains on disk; it is not deleted to make this pass"
        );
        let result = if route == "consumer_invalidated" {
            assert!(
                cockpit_repository::reuse_local_node_with_task_authorization(
                    root,
                    &command,
                    &runtime(),
                    None,
                    candidate
                )
                .unwrap()
                .is_none()
            );
            cockpit_repository::run_repository_verification_with_local_node_guard(
                root,
                &command,
                &runtime(),
                None,
                |_, _, _| Ok(()),
                None,
                cockpit_repository::LocalNodeExecutionContext {
                    own_run: None,
                    force_fresh: true,
                },
            )
            .unwrap()
        } else if route == "legacy_without_runtime" {
            cockpit_repository::run_repository_verification(root, &command).unwrap()
        } else {
            cockpit_repository::run_repository_verification_with_task_authorization(
                root,
                &command,
                &runtime(),
                None,
                |_, _, _| Ok(()),
            )
            .unwrap()
        };
        assert!(result.receipt.passed);
        assert_eq!(result.receipt.processes_spawned, 1);
        assert_eq!(result.receipt.nodes_reused, 0);
        assert_eq!(
            fs::read(counter).unwrap(),
            b"xxx",
            "fallback must actually execute instead of reusing the older success"
        );
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn all_reuse_parent_omitted_empty_records_preserves_repeat_zero_spawn_reuse() {
    let directory = attached_repository();
    let root = directory.path();
    let mut command = checkpoint_command(root, "printf all-reuse-parent");
    command.policy = RepositoryVerificationPolicy::ProfileAuthorized;
    local_authorized_fixture(std::slice::from_ref(&command), root);
    let original = local_real_success(root, &command);
    let plan = cockpit_repository::plan_local_node_reuse(
        root,
        std::slice::from_ref(&command),
        &original.final_snapshot,
        &runtime(),
    )
    .unwrap();
    let candidate = plan[0].candidate.as_ref().unwrap();
    let node = plan_repository_verification_action(
        root,
        &command,
        &original.final_snapshot,
        local_now_epoch_seconds(),
    )
    .unwrap();
    let owner = cockpit_repository::VerificationRunCheckpoint::prepare(
        root,
        std::slice::from_ref(&command),
        &[node],
        &original.final_snapshot,
        &runtime(),
    )
    .unwrap();
    let identity = owner.local_run_identity();
    let reused = cockpit_repository::reuse_local_node_in_run_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        candidate,
        Some(&identity),
    )
    .unwrap()
    .unwrap();
    assert_eq!(reused.receipt.processes_spawned, 0);
    assert_eq!(reused.receipt.nodes_reused, 1);
    let wire = serde_json::to_value(&reused.receipt).unwrap();
    assert!(
        wire.get("executionRecords").is_none(),
        "exercise actual skip_serializing_if wire behavior"
    );
    let parent = owner
        .persist_local_run_attempt(
            std::slice::from_ref(&command),
            &reused.final_snapshot,
            &runtime(),
            cockpit_repository::LocalRunAttemptWrite {
                state: "execution_completed",
                diagnostic: None,
                receipt: Some(&wire),
                supersedes: None,
            },
        )
        .unwrap();
    let stored: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join(parent["path"].as_str().unwrap())).unwrap())
            .unwrap();
    assert_eq!(stored["executionRecords"], json!([]));
    assert!(stored["receipt"].get("executionRecords").is_none());
    owner.record_local_run_completion(&parent).unwrap();
    let next = cockpit_repository::plan_local_node_reuse(
        root,
        std::slice::from_ref(&command),
        &reused.final_snapshot,
        &runtime(),
    )
    .unwrap();
    let again = cockpit_repository::reuse_local_node_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        next[0]
            .candidate
            .as_ref()
            .expect("normalized all-reuse parent must not poison its original physical proof"),
    )
    .unwrap()
    .unwrap();
    assert_eq!(again.receipt.processes_spawned, 0);
    assert_eq!(again.receipt.nodes_reused, 1);
}

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn an_old_equal_profile_parent_cannot_claim_a_later_unfinished_cached_execution_id() {
    let directory = attached_repository();
    let root = directory.path();
    let mut command = checkpoint_command(root, "printf same-cached-result");
    command.policy = RepositoryVerificationPolicy::ProfileAuthorized;
    local_authorized_fixture(std::slice::from_ref(&command), root);
    let original = local_real_success(root, &command);
    let old_cached = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    )
    .unwrap();
    assert!(old_cached.receipt.passed);
    assert_eq!(old_cached.receipt.processes_spawned, 0);
    assert_eq!(old_cached.receipt.nodes_reused, 1);
    let old_parent = persist_verification_attempt(
        root,
        "WI-ATTEMPT",
        std::slice::from_ref(&command),
        &old_cached.final_snapshot,
        &runtime(),
        "execution_completed",
        None,
        Some(&serde_json::to_value(&old_cached.receipt).unwrap()),
    )
    .unwrap();
    let old_parent_path = root.join(old_parent["path"].as_str().unwrap());
    let old_parent_bytes = fs::read(&old_parent_path).unwrap();
    let old_parent_value: serde_json::Value = serde_json::from_slice(&old_parent_bytes).unwrap();
    assert!(
        old_parent_value.get("runId").is_none(),
        "exercise the existing direct API parent without an explicit owner ID"
    );
    let old_cached_observations = checkpoint_values(root)
        .into_iter()
        .filter(|value| {
            value["kind"] == "verification_execution_checkpoint"
                && value["state"] == "node_execution_completed"
                && value["receipt"]["nodesReused"] == 1
        })
        .collect::<Vec<_>>();
    assert_eq!(old_cached_observations.len(), 1);
    let old_id = old_cached_observations[0]["executionId"].clone();
    // Temporarily retain only this fixture's two old cached checkpoint files
    // outside its repository. This lets the native profile executor produce a
    // second real zero-spawn observation even under the conservative fix. The
    // old successful parent stays present; all files are restored before any
    // assertion about the new execution. No fact/receipt is fabricated.
    let retained = tempfile::tempdir().unwrap();
    let mut moved = Vec::new();
    for entry in fs::read_dir(root.join(".ai/evidence"))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
    {
        if !entry
            .file_name()
            .to_string_lossy()
            .starts_with("WI-ATTEMPT.verification-attempt.")
        {
            continue;
        }
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(entry.path()).unwrap()).unwrap();
        if value["kind"] == "verification_execution_checkpoint" && value["executionId"] == old_id {
            let source = entry.path();
            let target = retained.path().join(entry.file_name());
            fs::rename(&source, &target).unwrap();
            moved.push((source, target));
        }
    }
    let before_later = cockpit_repository::plan_local_node_reuse(
        root,
        std::slice::from_ref(&command),
        &original.final_snapshot,
        &runtime(),
    );
    let later = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &command,
        &runtime(),
        None,
        |_, _, _| Ok(()),
    );
    // Query while the old checkpoint pair is still retained outside the view:
    // only the newly published unowned ID can cause this refusal. Restore the
    // old files before asserting, including if either native call returned Err.
    let after_later = cockpit_repository::plan_local_node_reuse(
        root,
        std::slice::from_ref(&command),
        &original.final_snapshot,
        &runtime(),
    );
    let moved_count = moved.len();
    for (source, target) in moved {
        assert!(!source.exists());
        fs::rename(target, source).unwrap();
    }
    assert_eq!(moved_count, 2);
    assert!(
        before_later.unwrap()[0].candidate.is_some(),
        "the isolated before-view must have a genuinely usable original physical fact"
    );
    let later = later.unwrap();
    assert!(later.receipt.passed);
    assert_eq!(later.receipt.processes_spawned, 0);
    assert_eq!(later.receipt.nodes_reused, 1);
    assert_eq!(
        later.receipt.results, old_cached.receipt.results,
        "the results really are equal; equality still cannot identify an owner"
    );
    let after_later = after_later.unwrap();
    assert!(
        after_later[0].candidate.is_none(),
        "the older pure-profile parent cannot resolve the later execution ID"
    );
    assert!(after_later[0].fresh_required);
    assert_eq!(fs::read(&old_parent_path).unwrap(), old_parent_bytes);
    let values = checkpoint_values(root);
    assert_eq!(
        values
            .iter()
            .filter(|value| value["kind"] == "verification_attempt")
            .count(),
        2,
        "no parent terminal is recorded for the later cached call"
    );
    let cached_ids = values
        .iter()
        .filter(|value| {
            value["kind"] == "verification_execution_checkpoint"
                && value["state"] == "node_execution_completed"
                && value["receipt"]["nodesReused"] == 1
        })
        .map(|value| value["executionId"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        cached_ids.len(),
        2,
        "two genuine native cached executions have different IDs"
    );
}
