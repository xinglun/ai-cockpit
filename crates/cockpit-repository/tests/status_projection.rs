use cockpit_core::Digest;
use cockpit_protocol::{HumanDecision, RuntimeContext};
use cockpit_repository::{
    WorkItemStartOptions, archive_work_item, attach, checkpoint_work_item,
    close_work_item_with_structured_decision, finish_work_item, preflight_work_item,
    record_verification, start_work_item, start_work_item_with_options,
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
        runtime_version: "0.1.0".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"status-runtime"),
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

#[test]
fn status_projection_distinguishes_archived_from_valid_closed_decision() {
    let directory = repository();
    let work_item_id = "WI-STATUS-CLOSED";
    start_work_item(
        directory.path(),
        work_item_id,
        "status close projection",
        "show terminal close state",
        &["**".into()],
    )
    .expect("start");
    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    record_verification(
        directory.path(),
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"status-runtime"),
    )
    .expect("verification");
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");

    let archived =
        work_item_status_snapshot_with_runtime(directory.path(), work_item_id, &runtime())
            .expect("archived status");
    assert_eq!(archived.lifecycle_phase, "archived");
    assert_eq!(archived.completion_domains["closure"], "archived");
    assert!(archived.human_decisions.is_empty());

    close_work_item_with_structured_decision(
        directory.path(),
        work_item_id,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "user-authorized-work-item".into(),
            reason: "status projection has fresh evidence".into(),
            evidence_refs: vec![format!(".ai/evidence/{work_item_id}.verification.json")],
            policy_refs: vec!["status-projection".into()],
            decided_at: "2026-08-22T12:00:00Z".into(),
            resume_condition: Some("rerun verification if the base changes".into()),
        },
    )
    .expect("close");
    let closed = work_item_status_snapshot_with_runtime(directory.path(), work_item_id, &runtime())
        .expect("closed status");
    assert_eq!(closed.lifecycle_phase, "closed");
    assert_eq!(closed.completion_domains["closure"], "closed");
    assert_eq!(closed.human_decisions, vec!["close_decision_recorded"]);

    let decision_path = directory
        .path()
        .join(format!(".ai/decisions/{work_item_id}.close.json"));
    let decision_bytes = fs::read(&decision_path).expect("decision bytes");
    let archived_summary = fs::read(directory.path().join(format!(
        ".ai/work-items/archive/{work_item_id}.summary.json"
    )))
    .expect("archived summary");
    assert!(!decision_bytes.is_empty());
    assert!(
        archived_summary
            .windows(b"finish_ready".len())
            .any(|window| window == b"finish_ready")
    );
}

#[test]
fn invalid_close_decision_never_promotes_archived_status() {
    let directory = repository();
    let work_item_id = "WI-STATUS-INVALID-CLOSE";
    start_work_item(
        directory.path(),
        work_item_id,
        "status invalid close",
        "reject invalid close projection",
        &["**".into()],
    )
    .expect("start");
    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    record_verification(
        directory.path(),
        work_item_id,
        &serde_json::json!({"passed": true}),
        "0.1.0",
        &Digest::sha256_bytes(b"status-runtime"),
    )
    .expect("verification");
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");
    let path = directory
        .path()
        .join(format!(".ai/decisions/{work_item_id}.close.json"));
    fs::write(
        &path,
        serde_json::json!({
            "workItemId": work_item_id,
            "state": "closed",
            "decisionState": "confirmed",
            "humanDecision": "approved",
            "structuredDecision": {"decision": "approved"}
        })
        .to_string(),
    )
    .expect("invalid decision");
    let status = work_item_status_snapshot_with_runtime(directory.path(), work_item_id, &runtime())
        .expect("status");
    assert_eq!(status.lifecycle_phase, "archived");
    assert_eq!(status.completion_domains["closure"], "archived");
    assert!(status.human_decisions.is_empty());
    assert!(status.unknowns.contains(&"close_decision_invalid".into()));
}

#[test]
fn foreign_close_repository_identity_never_promotes_archived_status() {
    let directory = repository();
    let work_item_id = "WI-STATUS-FOREIGN-CLOSE";
    start_work_item(
        directory.path(),
        work_item_id,
        "status foreign close",
        "reject cross-repository close receipt",
        &["**".into()],
    )
    .expect("start");
    let contract = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    record_verification(
        directory.path(),
        work_item_id,
        &serde_json::json!({"passed": true}),
        "0.1.0",
        &Digest::sha256_bytes(b"status-runtime"),
    )
    .expect("verification");
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");
    close_work_item_with_structured_decision(
        directory.path(),
        work_item_id,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "status-projection".into(),
            reason: "valid close before tamper".into(),
            evidence_refs: vec![format!(".ai/evidence/{work_item_id}.verification.json")],
            policy_refs: vec!["status-projection".into()],
            decided_at: "2026-08-22T12:00:00Z".into(),
            resume_condition: Some("rerun verification".into()),
        },
    )
    .expect("close");
    let path = directory
        .path()
        .join(format!(".ai/decisions/{work_item_id}.close.json"));
    let mut decision: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("decision")).expect("decision JSON");
    decision["repositoryId"] = "sha256:foreign".into();
    fs::write(
        &path,
        serde_json::to_vec_pretty(&decision).expect("decision bytes"),
    )
    .expect("tamper decision");
    let status = work_item_status_snapshot_with_runtime(directory.path(), work_item_id, &runtime())
        .expect("status");
    assert_eq!(status.lifecycle_phase, "archived");
    assert_eq!(status.completion_domains["closure"], "archived");
    assert!(status.unknowns.contains(&"close_decision_invalid".into()));
}
