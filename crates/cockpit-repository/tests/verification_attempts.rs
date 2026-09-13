use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::RuntimeContext;
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions, attach,
    load_reusable_verification_attempt, persist_verification_attempt, start_work_item_with_options,
};
use cockpit_verification::VerificationCommand;
use serde_json::json;
use std::{fs, path::Path, process::Command};

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
        policy: RepositoryVerificationPolicy::NeverReuse,
    }
}

fn attempt_receipt(
    root: &Path,
    request: &RepositoryVerificationRequest,
    passed: bool,
) -> serde_json::Value {
    let root = fs::canonicalize(root).expect("canonical root");
    let command_digest = VerificationCommand::new(
        &request.node_id,
        &request.program,
        request.args.clone(),
        cockpit_verification::VerificationReusePolicy::NeverReuse,
    )
    .with_current_dir(&root)
    .command_digest();
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
