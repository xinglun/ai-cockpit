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

fn repository() -> tempfile::TempDir {
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
    start_work_item_with_options(
        directory.path(),
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
    let root: &Path = directory.path();
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
    let root: &Path = directory.path();
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
