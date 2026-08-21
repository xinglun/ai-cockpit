use cockpit_core::Digest;
use cockpit_repository::{
    archive_work_item, attach, close_work_item_with_decision, finish_work_item,
    record_verification, start_work_item,
};
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn repository() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("cockpit-archive-integrity-{suffix}"));
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
