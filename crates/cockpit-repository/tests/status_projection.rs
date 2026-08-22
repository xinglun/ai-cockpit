use cockpit_core::Digest;
use cockpit_protocol::RuntimeContext;
use cockpit_repository::{
    WorkItemStartOptions, attach, start_work_item_with_options,
    work_item_status_snapshot_with_runtime,
};
use std::{fs, process::Command};

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

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"status"),
    }
}

#[test]
fn status_projection_is_read_only_and_contains_fact_counts() {
    let directory = repository();
    start_work_item_with_options(
        directory.path(),
        "WI-STATUS-A",
        "status projection",
        "read lifecycle facts",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect("start");
    let before = fs::read_dir(directory.path().join(".ai/work-items/active"))
        .expect("before")
        .count();
    let status =
        work_item_status_snapshot_with_runtime(directory.path(), "WI-STATUS-A", &runtime())
            .expect("status");
    let after = fs::read_dir(directory.path().join(".ai/work-items/active"))
        .expect("after")
        .count();
    assert_eq!(before, after, "status must not write repository state");
    assert_eq!(status.work_item_id, "WI-STATUS-A");
    assert_eq!(status.governance_state, "yellow");
    assert_eq!(status.verification, "not_ready");
    assert!(
        status
            .progress_facts
            .contains_key("acceptanceCriteriaDeclared")
    );
    assert!(
        status
            .unknowns
            .iter()
            .any(|item| item == "verification_evidence_missing")
    );
    assert!(
        status
            .governance_permissions
            .contains(&"read_status".into())
    );
}

#[test]
fn status_projection_isolated_between_repositories() {
    let left = repository();
    let right = repository();
    for (directory, id) in [(&left, "WI-LEFT"), (&right, "WI-RIGHT")] {
        start_work_item_with_options(
            directory.path(),
            id,
            "isolated status",
            "keep contexts separate",
            &["**".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                ..Default::default()
            },
        )
        .expect("start");
    }
    let left_status =
        work_item_status_snapshot_with_runtime(left.path(), "WI-LEFT", &runtime()).expect("left");
    let right_status = work_item_status_snapshot_with_runtime(right.path(), "WI-RIGHT", &runtime())
        .expect("right");
    assert_ne!(left_status.repository_id, right_status.repository_id);
    assert_eq!(left_status.work_item_id, "WI-LEFT");
    assert_eq!(right_status.work_item_id, "WI-RIGHT");
}
