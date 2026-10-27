//! P2-C fault injection for lifecycle transitions.
//!
//! These tests exercise both same-process and cross-process callers.  The
//! latter matters because a process-local mutex cannot protect the repository
//! when two CLI invocations advance the same Work Item at once.

use cockpit_core::Digest;
use cockpit_protocol::{
    AssuranceLevel, RuntimeContext, TaskAuthorizationCaller, TaskAuthorizationPolicy,
};
use cockpit_repository::{
    WorkItemStartOptions, archive_work_item, attach, checkpoint_work_item,
    close_work_item_with_decision, finish_work_item, preflight_work_item,
    preflight_work_item_with_runtime_report, record_verification,
    record_work_item_governance_controls_with_runtime, repository_id, start_work_item_with_options,
};
use serde_json::Value;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};
use std::{env, fs, path::Path, process::Command, thread};

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=AI Cockpit test fixture",
                "-c",
                "user.email=ai-cockpit-test@example.invalid",
                "commit",
                "--allow-empty",
                "-qm",
                "test fixture baseline",
            ])
            .current_dir(directory.path())
            .status()
            .expect("git baseline commit")
            .success()
    );
    attach(directory.path()).expect("attach");
    directory
}

fn contract_path(root: &Path, id: &str) -> std::path::PathBuf {
    root.join(".ai/work-items/active")
        .join(format!("{id}.contract.json"))
}

fn batch_runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "1.0.1".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"lifecycle-cross-process-batch-start"),
    }
}

fn prepare_batch_start_member(
    root: &Path,
    work_item_id: &str,
) -> cockpit_protocol::TaskAuthorizationBatchReceipt {
    let policy = TaskAuthorizationPolicy {
        schema_version: 1,
        agent_actor: "agent:cross-process-batch-worker".into(),
        authority_source: "conversation:cross-process-batch-fixture-owner".into(),
        allowed_actions: vec!["run_verification".into()],
        closeout_actions: Vec::new(),
        expires_at: Some("2099-01-01T00:00:00Z".into()),
        public_target: None,
    };
    start_work_item_with_options(
        root,
        work_item_id,
        "exercise cross-process batch quota",
        "prove one member start quota is reserved atomically across processes",
        &["README.md".into()],
        &WorkItemStartOptions {
            task_authorization: Some(policy),
            authority: "authorized".into(),
            acceptance_criteria: vec!["one durable start reservation wins".into()],
            verification_commands: vec!["git diff --check".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start batch member Work Item");

    let contract_path = contract_path(root, work_item_id);
    preflight_work_item_with_runtime_report(root, &contract_path, &batch_runtime())
        .expect("preflight batch member Work Item");
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
        .expect("valid contract JSON");
    let summary: Value = serde_json::from_slice(
        &fs::read(root.join(format!(".ai/work-items/active/{work_item_id}.summary.json")))
            .expect("summary"),
    )
    .expect("valid summary JSON");
    record_work_item_governance_controls_with_runtime(
        root,
        work_item_id,
        &serde_json::json!({"decisionEvidence":{
            "schemaVersion":1,
            "decisionId":"contract-preflight-review",
            "decision":"confirm_review",
            "workItemId":work_item_id,
            "repositoryId":repository_id(root),
            "contractDigest":cockpit_protocol::digest_json(&contract).expect("contract digest"),
            "preflightDecisionDigest":summary["preflightDecisionDigest"],
            "repositorySnapshotDigest":summary["preflightRepositorySnapshotDigest"],
            "recordedAt":chrono::Utc::now().to_rfc3339(),
            "recordedBy":"agent:test-recorder",
            "reason":"synthetic owner confirmation for isolated concurrency fixture"
        }}),
        &batch_runtime(),
    )
    .expect("record exact member owner review");

    let request = cockpit_protocol::TaskAuthorizationBatchRequest {
        schema_version: 1,
        repository_id: repository_id(root).to_string(),
        members: vec![cockpit_protocol::TaskAuthorizationBatchMemberRequest {
            work_item_id: work_item_id.into(),
            actions: vec!["run_verification".into()],
            expires_at: "2098-01-01T00:00:00Z".into(),
            budget: cockpit_protocol::TaskAuthorizationBatchBudget {
                start_quota: 1,
                retry_budget_by_check: std::collections::BTreeMap::from([(
                    "git diff --check".into(),
                    2,
                )]),
            },
        }],
    };
    let manifest = cockpit_repository::prepare_task_authorization_batch_manifest(
        root,
        &request,
        &batch_runtime(),
    )
    .expect("prepare exact batch manifest");
    let review = cockpit_protocol::TaskAuthorizationBatchReview {
        schema_version: 1,
        repository_id: manifest.repository_id.clone(),
        manifest_digest: manifest.manifest_digest.clone(),
        decision: cockpit_protocol::TaskAuthorizationBatchDecision::ConfirmReview,
        reviewer_actor: "human:owner".into(),
        authority_source: "conversation:owner-confirmed-exact-manifest".into(),
        recorded_at: chrono::Utc::now().to_rfc3339(),
        reason: "confirm this exact isolated member and its one-start quota".into(),
    };
    cockpit_repository::record_task_authorization_batch_review(
        root,
        &manifest,
        &review,
        &batch_runtime(),
    )
    .expect("persist exact batch review")
}

/// Bring a Work Item to the point where `finish_work_item` is allowed.
fn prepare_finish_ready(root: &Path, work_item_id: &str) {
    start_work_item_with_options(
        root,
        work_item_id,
        "lifecycle concurrency fault injection",
        "prove lifecycle behavior under duplicate and interrupted calls",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["lifecycle writes are serialized and recoverable".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(contract_path(root, work_item_id)).expect("contract"))
            .expect("contract JSON");
    assert!(contract.get("resourceContext").is_none());
    preflight_work_item(root, &contract_path(root, work_item_id)).expect("preflight");
    checkpoint_work_item(root, work_item_id).expect("checkpoint");
    record_verification(
        root,
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.87",
        &Digest::sha256_bytes(b"lifecycle-concurrency-runtime"),
    )
    .expect("verification");
}

fn read_active_json(root: &Path, id: &str, suffix: &str) -> Value {
    let path = root
        .join(".ai/work-items/active")
        .join(format!("{id}.{suffix}.json"));
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    serde_json::from_slice(&bytes).unwrap_or_else(|error| panic!("parse {path:?}: {error}"))
}

fn assert_business_error<T, E: std::fmt::Display>(result: &Result<T, E>) {
    if let Err(error) = result {
        let message = error.to_string();
        assert!(
            !message.contains("No such file or directory") && !message.contains("os error 2"),
            "concurrent loser must fail with a business-level result, not a raw filesystem race: {message}"
        );
    }
}

#[test]
fn same_process_finish_calls_are_serialized_and_json_remains_valid() {
    let directory = repository();
    let work_item_id = "WI-SAME-PROCESS-FINISH";
    prepare_finish_ready(directory.path(), work_item_id);

    let barrier = Arc::new(Barrier::new(2));
    let workers = (0..2)
        .map(|_| {
            let root = directory.path().to_path_buf();
            let barrier = Arc::clone(&barrier);
            let id = work_item_id.to_string();
            thread::spawn(move || {
                barrier.wait();
                finish_work_item(&root, &id)
            })
        })
        .collect::<Vec<_>>();
    let results = workers
        .into_iter()
        .map(|worker| worker.join().expect("finish worker thread"))
        .collect::<Vec<_>>();

    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    for result in &results {
        assert_business_error(result);
    }
    assert_eq!(
        read_active_json(directory.path(), work_item_id, "summary")["state"],
        "finish_ready"
    );
    assert_eq!(
        read_active_json(directory.path(), work_item_id, "outcome")["state"],
        "finish_ready"
    );
}

#[test]
fn cross_process_finish_calls_have_one_authoritative_commit() {
    if env::var_os("COCKPIT_LIFECYCLE_CHILD_ROOT").is_some() {
        return;
    }

    let directory = repository();
    let work_item_id = "WI-CROSS-PROCESS-FINISH";
    prepare_finish_ready(directory.path(), work_item_id);
    let gate = directory.path().join(".ai/lifecycle-child-start");
    let mut children = Vec::new();
    for index in 0..2 {
        let result_path = directory
            .path()
            .join(format!(".ai/lifecycle-child-result-{index}.json"));
        let child = Command::new(env::current_exe().expect("current test executable"))
            .args(["--exact", "cross_process_finish_child", "--nocapture"])
            .env("COCKPIT_LIFECYCLE_CHILD_ROOT", directory.path())
            .env("COCKPIT_LIFECYCLE_CHILD_ID", work_item_id)
            .env("COCKPIT_LIFECYCLE_CHILD_GATE", &gate)
            .env("COCKPIT_LIFECYCLE_CHILD_RESULT", &result_path)
            .spawn()
            .expect("spawn lifecycle child");
        children.push((child, result_path));
    }
    fs::write(&gate, b"go").expect("release lifecycle children");
    for (mut child, result_path) in children {
        assert!(child.wait().expect("wait lifecycle child").success());
        assert!(result_path.is_file(), "child did not record a result");
    }

    // The result files are under .ai, so enumerate them from the directory
    // explicitly rather than relying on a recursive glob in the assertion.
    let results = fs::read_dir(directory.path().join(".ai"))
        .expect("read lifecycle result directory")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("lifecycle-child-result-")
        })
        .map(|entry| {
            let bytes = fs::read(entry.path()).expect("read child result");
            serde_json::from_slice::<Value>(&bytes).expect("parse child result")
        })
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 2);
    assert_eq!(
        results.iter().filter(|result| result["ok"] == true).count(),
        1
    );
    assert_eq!(
        read_active_json(directory.path(), work_item_id, "summary")["state"],
        "finish_ready"
    );
    assert_eq!(
        read_active_json(directory.path(), work_item_id, "outcome")["state"],
        "finish_ready"
    );
}

#[test]
fn cross_process_finish_child() {
    let Some(root) = env::var_os("COCKPIT_LIFECYCLE_CHILD_ROOT") else {
        return;
    };
    let id = env::var("COCKPIT_LIFECYCLE_CHILD_ID").expect("child Work Item id");
    let gate = env::var_os("COCKPIT_LIFECYCLE_CHILD_GATE").expect("child gate");
    let result_path = env::var_os("COCKPIT_LIFECYCLE_CHILD_RESULT").expect("child result");
    let deadline = Instant::now() + Duration::from_secs(30);
    while !Path::new(&gate).exists() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for child gate"
        );
        thread::sleep(Duration::from_millis(2));
    }
    let result = finish_work_item(Path::new(&root), &id);
    let value = serde_json::json!({
        "ok": result.is_ok(),
        "error": result.err().map(|error| error.to_string()),
    });
    fs::write(
        result_path,
        serde_json::to_vec(&value).expect("encode child result"),
    )
    .expect("write child result");
}

#[test]
fn concurrent_member_start_reserves_one_quota_atomically() {
    if env::var_os("COCKPIT_BATCH_START_CHILD_ROOT").is_some() {
        return;
    }

    const WORK_ITEM_ID: &str = "WI-CROSS-PROCESS-BATCH-START";
    let directory = repository();
    let root = directory.path();
    let receipt = prepare_batch_start_member(root, WORK_ITEM_ID);
    let member = receipt
        .manifest
        .members
        .first()
        .expect("one explicitly reviewed member")
        .clone();
    let gate = root.join(".ai/batch-start-child-gate");
    let mut children = Vec::with_capacity(2);
    let mut ready_paths = Vec::with_capacity(2);
    let mut result_paths = Vec::with_capacity(2);
    let mut spawn_error = None;

    for index in 0..2 {
        let ready_path = root.join(format!(".ai/batch-start-child-ready-{index}"));
        let result_path = root.join(format!(".ai/batch-start-child-result-{index}.json"));
        let child = Command::new(env::current_exe().expect("current test executable"))
            .args(["--exact", "concurrent_member_start_child", "--nocapture"])
            .env("COCKPIT_BATCH_START_CHILD_ROOT", root)
            .env("COCKPIT_BATCH_START_CHILD_WI", WORK_ITEM_ID)
            .env(
                "COCKPIT_BATCH_START_CHILD_MANIFEST",
                receipt.manifest.manifest_digest.to_string(),
            )
            .env(
                "COCKPIT_BATCH_START_CHILD_OPERATION",
                format!("run-{index}"),
            )
            .env("COCKPIT_BATCH_START_CHILD_ACTOR", &member.actor)
            .env(
                "COCKPIT_BATCH_START_CHILD_AUTHORITY",
                &member.authority_source,
            )
            .env("COCKPIT_BATCH_START_CHILD_GATE", &gate)
            .env("COCKPIT_BATCH_START_CHILD_READY", &ready_path)
            .env("COCKPIT_BATCH_START_CHILD_RESULT", &result_path)
            .spawn();
        match child {
            Ok(child) => {
                children.push(child);
                ready_paths.push(ready_path);
                result_paths.push(result_path);
            }
            Err(error) => {
                spawn_error = Some(error.to_string());
                break;
            }
        }
    }

    if children.len() != 2 {
        let _ = fs::write(&gate, b"release");
        for mut child in children {
            let _ = child.wait();
        }
        panic!("must spawn both quota contenders: {spawn_error:?}");
    }

    let ready_deadline = Instant::now() + Duration::from_secs(15);
    while !ready_paths.iter().all(|path| path.is_file()) && Instant::now() < ready_deadline {
        thread::sleep(Duration::from_millis(5));
    }
    let both_ready = ready_paths.iter().all(|path| path.is_file());
    let gate_result = fs::write(&gate, b"release");
    let child_statuses = children
        .into_iter()
        .map(|mut child| child.wait().expect("wait for quota contender process"))
        .collect::<Vec<_>>();

    assert!(
        both_ready,
        "both independent processes must reach the barrier"
    );
    gate_result.expect("release both independent contenders");
    assert!(
        child_statuses.iter().all(std::process::ExitStatus::success),
        "both child test processes must finish cleanly: {child_statuses:?}"
    );

    let results = result_paths
        .iter()
        .map(|path| {
            let bytes = fs::read(path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
            serde_json::from_slice::<Value>(&bytes)
                .unwrap_or_else(|error| panic!("parse {path:?}: {error}"))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        results.iter().filter(|result| result["ok"] == true).count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter(|result| result["ok"] == false)
            .count(),
        1
    );
    assert!(
        results
            .iter()
            .filter_map(|result| result["error"].as_str())
            .any(|error| error.contains("start_quota_exhausted"))
    );

    let ledger = cockpit_repository::inspect_task_authorization_batch_member(
        root,
        &receipt.manifest.manifest_digest,
        WORK_ITEM_ID,
        &batch_runtime(),
    )
    .expect("read durable member ledger");
    assert_eq!(ledger.starts_used, 1);
    assert_eq!(ledger.start_quota, 1);
    assert_eq!(ledger.events.len(), 1);
    assert_eq!(
        ledger.events[0].kind,
        cockpit_protocol::TaskAuthorizationBatchEventKind::StartReserved
    );
}

#[test]
fn concurrent_member_start_child() {
    let Some(root) = env::var_os("COCKPIT_BATCH_START_CHILD_ROOT") else {
        return;
    };
    let work_item_id = env::var("COCKPIT_BATCH_START_CHILD_WI").expect("Work Item id");
    let manifest_digest = env::var("COCKPIT_BATCH_START_CHILD_MANIFEST")
        .expect("manifest digest")
        .parse::<Digest>()
        .expect("valid manifest digest");
    let operation_id = env::var("COCKPIT_BATCH_START_CHILD_OPERATION").expect("operation id");
    let actor = env::var("COCKPIT_BATCH_START_CHILD_ACTOR").expect("member actor");
    let authority_source =
        env::var("COCKPIT_BATCH_START_CHILD_AUTHORITY").expect("member authority source");
    let gate = env::var_os("COCKPIT_BATCH_START_CHILD_GATE").expect("start gate");
    let ready = env::var_os("COCKPIT_BATCH_START_CHILD_READY").expect("ready marker");
    let result_path = env::var_os("COCKPIT_BATCH_START_CHILD_RESULT").expect("result path");
    fs::write(&ready, b"ready").expect("signal child ready");

    let deadline = Instant::now() + Duration::from_secs(30);
    while !Path::new(&gate).exists() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for parent to release quota barrier"
        );
        thread::sleep(Duration::from_millis(5));
    }

    let request = cockpit_protocol::TaskAuthorizationBatchStartRequest {
        schema_version: 1,
        manifest_digest,
        work_item_id,
        operation_id,
        action: "run_verification".into(),
        caller: TaskAuthorizationCaller {
            schema_version: 1,
            actor,
            authority_source,
            assurance: AssuranceLevel::SelfDeclared,
        },
    };
    let result = cockpit_repository::reserve_task_authorization_batch_start(
        Path::new(&root),
        &request,
        &batch_runtime(),
    );
    let value = match result {
        Ok(_) => serde_json::json!({"ok":true}),
        Err(error) => serde_json::json!({"ok":false,"error":error.to_string()}),
    };
    fs::write(
        result_path,
        serde_json::to_vec(&value).expect("encode contender result"),
    )
    .expect("write contender result");
}

#[test]
fn concurrent_archive_and_close_have_single_commit_each() {
    let directory = repository();
    let work_item_id = "WI-ARCHIVE-CLOSE-RACE";
    prepare_finish_ready(directory.path(), work_item_id);
    finish_work_item(directory.path(), work_item_id).expect("finish");

    let barrier = Arc::new(Barrier::new(2));
    let archive_workers = (0..2)
        .map(|_| {
            let root = directory.path().to_path_buf();
            let barrier = Arc::clone(&barrier);
            let id = work_item_id.to_string();
            thread::spawn(move || {
                barrier.wait();
                archive_work_item(&root, &id)
            })
        })
        .collect::<Vec<_>>();
    let archive_results = archive_workers
        .into_iter()
        .map(|worker| worker.join().expect("archive worker thread"))
        .collect::<Vec<_>>();
    assert_eq!(
        archive_results
            .iter()
            .filter(|result| result.is_ok())
            .count(),
        1
    );
    for result in &archive_results {
        assert_business_error(result);
    }
    let manifest = directory
        .path()
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.archive.json"));
    let manifest_value: Value =
        serde_json::from_slice(&fs::read(manifest).expect("archive manifest"))
            .expect("valid archive manifest");
    assert_eq!(manifest_value["state"], "archived");

    let barrier = Arc::new(Barrier::new(2));
    let close_workers = (0..2)
        .map(|_| {
            let root = directory.path().to_path_buf();
            let barrier = Arc::clone(&barrier);
            let id = work_item_id.to_string();
            thread::spawn(move || {
                barrier.wait();
                close_work_item_with_decision(&root, &id, "approved")
            })
        })
        .collect::<Vec<_>>();
    let close_results = close_workers
        .into_iter()
        .map(|worker| worker.join().expect("close worker thread"))
        .collect::<Vec<_>>();
    assert_eq!(
        close_results.iter().filter(|result| result.is_ok()).count(),
        1
    );
    for result in &close_results {
        assert_business_error(result);
    }
    let decision = directory
        .path()
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    let _: Value = serde_json::from_slice(&fs::read(decision).expect("close decision"))
        .expect("valid close decision");
}

#[test]
fn missing_or_corrupt_projections_fail_closed_before_commit() {
    let directory = repository();
    let missing_id = "WI-MISSING-OUTCOME";
    prepare_finish_ready(directory.path(), missing_id);
    finish_work_item(directory.path(), missing_id).expect("finish missing projection fixture");
    fs::remove_file(
        directory
            .path()
            .join(".ai/work-items/active")
            .join(format!("{missing_id}.outcome.json")),
    )
    .expect("remove active outcome");
    assert!(archive_work_item(directory.path(), missing_id).is_err());
    assert!(
        !directory
            .path()
            .join(".ai/work-items/archive")
            .join(format!("{missing_id}.archive.json"))
            .exists()
    );

    let corrupt_id = "WI-CORRUPT-ARCHIVE";
    prepare_finish_ready(directory.path(), corrupt_id);
    finish_work_item(directory.path(), corrupt_id).expect("finish corrupt projection fixture");
    archive_work_item(directory.path(), corrupt_id).expect("archive corrupt projection fixture");
    let manifest = directory
        .path()
        .join(".ai/work-items/archive")
        .join(format!("{corrupt_id}.archive.json"));
    fs::write(&manifest, b"{}").expect("corrupt archive manifest");
    assert!(close_work_item_with_decision(directory.path(), corrupt_id, "approved").is_err());
    assert!(
        !directory
            .path()
            .join(".ai/decisions")
            .join(format!("{corrupt_id}.close.json"))
            .exists()
    );
}
