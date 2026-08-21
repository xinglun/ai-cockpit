use cockpit_core::Digest;
use cockpit_protocol::HumanDecision;
use cockpit_repository::{
    archive_work_item, attach, close_work_item_with_decision,
    close_work_item_with_structured_decision, finish_work_item, record_verification,
    start_work_item,
};
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

fn repository() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "cockpit-archive-integrity-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&path)
        .status()
        .expect("git init");
    attach(&path).expect("attach");
    path
}

#[test]
fn close_rejects_tampered_archived_artifacts() {
    let path = repository();
    start_work_item(&path, "WI-INTEGRITY", "integrity", "verify", &["**".into()]).expect("start");
    record_verification(
        &path,
        "WI-INTEGRITY",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, "WI-INTEGRITY").expect("finish");
    archive_work_item(&path, "WI-INTEGRITY").expect("archive");
    fs::write(
        path.join(".ai/work-items/archive/WI-INTEGRITY.outcome.json"),
        br#"{"tampered":true}"#,
    )
    .expect("tamper");
    let error = close_work_item_with_decision(&path, "WI-INTEGRITY", "approved")
        .expect_err("tampered archive must be rejected");
    assert!(error.to_string().contains("digest does not match manifest"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn verification_receipt_cannot_cross_work_items() {
    let path = repository();
    start_work_item(&path, "WI-A", "first", "verify", &["**".into()]).expect("start A");
    start_work_item(&path, "WI-B", "second", "verify", &["**".into()]).expect("start B");
    let error = record_verification(
        &path,
        "WI-B",
        &serde_json::json!({"passed": true, "workItemId": "WI-A", "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect_err("cross-work-item evidence must be rejected");
    assert!(error.to_string().contains("another work item"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn close_persists_a_structured_human_decision_and_recovery_condition() {
    let path = repository();
    start_work_item(&path, "WI-DECISION", "decision", "verify", &["**".into()]).expect("start");
    record_verification(
        &path,
        "WI-DECISION",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, "WI-DECISION").expect("finish");
    archive_work_item(&path, "WI-DECISION").expect("archive");
    close_work_item_with_structured_decision(
        &path,
        "WI-DECISION",
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "team-policy".into(),
            reason: "bounded change and fresh evidence".into(),
            evidence_refs: vec![".ai/evidence/WI-DECISION.verification.json".into()],
            policy_refs: vec!["team-policy-v1".into()],
            decided_at: "2026-08-21T19:00:00Z".into(),
            resume_condition: Some("rerun verification if the base changes".into()),
        },
    )
    .expect("structured close");
    let decision: serde_json::Value = serde_json::from_slice(
        &fs::read(path.join(".ai/decisions/WI-DECISION.close.json")).expect("decision"),
    )
    .expect("decision JSON");
    assert_eq!(decision["structuredDecision"]["actor"], "human:owner");
    assert_eq!(
        decision["structuredDecision"]["resumeCondition"],
        "rerun verification if the base changes"
    );
    fs::remove_dir_all(path).expect("cleanup");
}
