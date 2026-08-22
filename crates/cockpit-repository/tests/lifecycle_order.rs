use cockpit_core::{DecisionState, Digest};
use cockpit_repository::{
    WorkItemStartOptions, attach, checkpoint_work_item, finish_work_item, preflight_work_item,
    record_verification, start_work_item_with_options,
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

fn start(path: &std::path::Path, id: &str, required: &[&str]) {
    start_work_item_with_options(
        path,
        id,
        "lifecycle ordering",
        "preserve serial governance",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            required_evidence_classes: required.iter().map(|value| (*value).into()).collect(),
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
}

fn contract(path: &std::path::Path, id: &str) -> std::path::PathBuf {
    path.join(".ai/work-items/active")
        .join(format!("{id}.contract.json"))
}

#[test]
fn skipped_preflight_and_checkpoint_fail_closed() {
    let directory = repository();
    start(directory.path(), "WI-ORDER-SKIP", &[]);

    let verify = record_verification(
        directory.path(),
        "WI-ORDER-SKIP",
        &serde_json::json!({"passed": true}),
        "0.2.7",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect_err("verification before checkpoint must fail closed");
    assert!(verify.to_string().contains("checkpoint"));
    let finish = finish_work_item(directory.path(), "WI-ORDER-SKIP")
        .expect_err("finish without preflight/checkpoint must fail closed");
    assert!(finish.to_string().contains("state") || finish.to_string().contains("checkpoint"));
    assert!(
        directory
            .path()
            .join(".ai/work-items/active/WI-ORDER-SKIP.summary.json")
            .is_file()
    );
}

#[test]
fn checkpoint_requires_preflight_and_duplicate_checkpoint_is_rejected() {
    let directory = repository();
    start(directory.path(), "WI-ORDER-DUPLICATE", &[]);
    let summary = directory
        .path()
        .join(".ai/work-items/active/WI-ORDER-DUPLICATE.summary.json");

    let before_preflight = checkpoint_work_item(directory.path(), "WI-ORDER-DUPLICATE")
        .expect_err("checkpoint without preflight must fail closed");
    assert!(before_preflight.to_string().contains("preflight"));

    let decision = preflight_work_item(
        directory.path(),
        &contract(directory.path(), "WI-ORDER-DUPLICATE"),
    )
    .expect("preflight");
    assert_eq!(decision.state, DecisionState::Green);
    checkpoint_work_item(directory.path(), "WI-ORDER-DUPLICATE").expect("checkpoint");
    let duplicate = checkpoint_work_item(directory.path(), "WI-ORDER-DUPLICATE")
        .expect_err("duplicate checkpoint must fail closed");
    assert!(duplicate.to_string().contains("duplicate"));
    let stored: serde_json::Value =
        serde_json::from_slice(&fs::read(summary).expect("summary")).expect("summary JSON");
    assert_eq!(stored["checkpointCount"], 1);
    assert_eq!(stored["state"], "checkpointed");
}

#[test]
fn verification_promotes_initial_yellow_preflight_and_allows_recovery() {
    let directory = repository();
    start(directory.path(), "WI-ORDER-RECOVER", &["verification"]);
    let decision = preflight_work_item(
        directory.path(),
        &contract(directory.path(), "WI-ORDER-RECOVER"),
    )
    .expect("preflight");
    assert_eq!(decision.state, DecisionState::Yellow);
    checkpoint_work_item(directory.path(), "WI-ORDER-RECOVER").expect("checkpoint");

    let missing = finish_work_item(directory.path(), "WI-ORDER-RECOVER")
        .expect_err("finish before verification must preserve recovery state");
    assert!(
        missing.to_string().contains("preflight") || missing.to_string().contains("verification")
    );

    record_verification(
        directory.path(),
        "WI-ORDER-RECOVER",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.7",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-ORDER-RECOVER.summary.json"),
        )
        .expect("summary"),
    )
    .expect("summary JSON");
    assert_eq!(summary["preflightState"], "green");
    finish_work_item(directory.path(), "WI-ORDER-RECOVER").expect("finish after recovery");
}
