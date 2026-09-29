use cockpit_repository::{
    WorkItemStartOptions, amend_work_item_contract, attach, checkpoint_work_item,
    preflight_work_item, start_work_item_with_options,
};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Command;

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("repository tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    attach(directory.path()).expect("attach repository");
    directory
}

fn read_json(path: impl AsRef<Path>) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read JSON file")).expect("valid JSON")
}

#[test]
fn typed_request_replaces_goal_without_rewriting_identity() {
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-TYPED-CONTRACT-AMENDMENT";
    start_work_item_with_options(
        root,
        work_item_id,
        "exercise a typed Contract amendment",
        "replace a human-owned plan field while preserving Runtime identity",
        &["crates/cockpit-repository/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["the replacement and identity are observed".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");

    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    preflight_work_item(root, &contract_path).expect("initial preflight");
    checkpoint_work_item(root, work_item_id).expect("checkpoint");
    let original = read_json(&contract_path);
    let expected_digest = cockpit_protocol::digest_json(&original).expect("Contract digest");
    let replacement = "Replace brittle legacy command handling with governed typed amendments";
    let reason = "the implementation plan now requires replacing, not only appending, goals";

    amend_work_item_contract(
        root,
        work_item_id,
        &json!({
            "schemaVersion": 1,
            "changeId": "replace-goal-1",
            "expectedContractDigest": expected_digest,
            "reason": reason,
            "changes": [{
                "path": "/goal",
                "operation": "replace",
                "value": replacement
            }]
        }),
        reason,
    )
    .expect("typed reasoned amendment");

    let amended = read_json(&contract_path);
    assert_eq!(amended["goal"], replacement);
    for field in [
        "repositoryId",
        "workItemId",
        "baseRevision",
        "projectProfileDigest",
        "repositorySnapshotDigest",
        "authority",
        "state",
    ] {
        assert_eq!(amended[field], original[field], "protected field {field}");
    }
}

#[test]
fn stale_contract_digest_rejects_without_writing() {
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-STALE-CONTRACT-AMENDMENT";
    start_work_item_with_options(
        root,
        work_item_id,
        "reject stale amendment input",
        "do not silently rebase an amendment over a changed Contract",
        &["crates/cockpit-repository/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["stale amendment state remains unchanged".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    preflight_work_item(root, &contract_path).expect("initial preflight");
    checkpoint_work_item(root, work_item_id).expect("checkpoint");
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let original_contract = fs::read(&contract_path).expect("Contract bytes");
    let original_summary = fs::read(&summary_path).expect("Summary bytes");
    let request = json!({
        "schemaVersion": 1,
        "changeId": "stale-change-1",
        "expectedContractDigest": cockpit_core::Digest::sha256_bytes(b"stale").to_string(),
        "reason": "this request was prepared against an older Contract",
        "changes": [{
            "path": "/goal",
            "operation": "replace",
            "value": "do not apply"
        }]
    });

    let error = amend_work_item_contract(
        root,
        work_item_id,
        &request,
        "this request was prepared against an older Contract",
    )
    .expect_err("stale digest must conflict");
    assert!(error.to_string().contains("contract_digest_conflict"));
    assert_eq!(
        fs::read(&contract_path).expect("Contract bytes"),
        original_contract
    );
    assert_eq!(
        fs::read(&summary_path).expect("Summary bytes"),
        original_summary
    );
}
