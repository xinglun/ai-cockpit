//! P2-C fault injection for lifecycle transitions.
//!
//! These tests exercise both same-process and cross-process callers.  The
//! latter matters because a process-local mutex cannot protect the repository
//! when two CLI invocations advance the same Work Item at once.

use cockpit_core::Digest;
use cockpit_protocol::ResourceFinalizationContext;
use cockpit_repository::{
    WorkItemStartOptions, archive_work_item, attach, checkpoint_work_item,
    close_work_item_with_decision, finish_work_item, plan_resource_finalization,
    preflight_work_item, record_verification, start_work_item_with_options,
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
    attach(directory.path()).expect("attach");
    directory
}

fn contract_path(root: &Path, id: &str) -> std::path::PathBuf {
    root.join(".ai/work-items/active")
        .join(format!("{id}.contract.json"))
}

/// Bring a Work Item to the point where `finish_work_item` is allowed.
fn prepare_finish_ready(root: &Path, work_item_id: &str) {
    start_work_item_with_options(
        root,
        work_item_id,
        "lifecycle concurrency fault injection",
        "prove lifecycle behavior under duplicate and interrupted calls",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["lifecycle writes are serialized and recoverable".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    plan_resource_finalization(
        root,
        work_item_id,
        &ResourceFinalizationContext {
            branch: format!("feature/{work_item_id}"),
            worktree: root.display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: format!("https://github.com/example/ai-cockpit/pull/{work_item_id}"),
        },
    )
    .expect("finalization plan");
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
