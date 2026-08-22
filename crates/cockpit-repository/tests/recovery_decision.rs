use cockpit_core::Digest;
use cockpit_protocol::RuntimeContext;
use cockpit_repository::{
    WorkItemStartOptions, attach, checkpoint_work_item, outcome_v2, preflight_work_item,
    record_recovery_decision, repository_id, start_work_item_with_options,
};
use serde_json::json;
use std::fs;
use std::process::Command;

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
    start_work_item_with_options(
        directory.path(),
        "WI-BLOCKED",
        "recover a blocked item",
        "record an explicit recovery decision",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["predecessor remains immutable".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    let contract = directory
        .path()
        .join(".ai/work-items/active/WI-BLOCKED.contract.json");
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), "WI-BLOCKED").expect("checkpoint");
    fs::write(
        directory
            .path()
            .join(".ai/work-items/active/WI-BLOCKED.outcome.json"),
        br#"{"state":"blocked","workItemId":"WI-BLOCKED"}"#,
    )
    .unwrap();
    fs::write(
        directory
            .path()
            .join(".ai/work-items/active/WI-BLOCKED.events.jsonl"),
        b"blocked-event\n",
    )
    .unwrap();
    directory
}

fn receipt(directory: &tempfile::TempDir, reason: &str) -> serde_json::Value {
    let root = directory.path();
    let contract_path = root.join(".ai/work-items/active/WI-BLOCKED.contract.json");
    let summary_path = root.join(".ai/work-items/active/WI-BLOCKED.summary.json");
    let outcome_path = root.join(".ai/work-items/active/WI-BLOCKED.outcome.json");
    let events_path = root.join(".ai/work-items/active/WI-BLOCKED.events.jsonl");
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
    json!({
        "schemaVersion": 1,
        "decisionId": "work-item-recovery",
        "decision": "successor",
        "workItemId": "WI-BLOCKED",
        "repositoryId": repository_id(root),
        "predecessorWorkItemId": "WI-BLOCKED",
        "predecessorContractDigest": cockpit_protocol::digest_json(&contract).unwrap(),
        "predecessorSummaryDigest": cockpit_protocol::digest_json(&summary).unwrap(),
        "predecessorOutcomeDigest": cockpit_protocol::digest_json(&serde_json::from_slice::<serde_json::Value>(&fs::read(&outcome_path).unwrap()).unwrap()).unwrap(),
        "predecessorEventsDigest": Digest::sha256_bytes(&fs::read(events_path).unwrap()),
        "successorWorkItemId": "WI-SUCCESSOR",
        "runtimeVersion": "0.2.12",
        "runtimeDigest": Digest::sha256_bytes(b"runtime"),
        "actor": "human:owner",
        "authoritySource": "repository-local",
        "reason": reason,
        "evidenceRefs": [".ai/work-items/active/WI-BLOCKED.outcome.json"],
        "policyRefs": [],
        "decidedAt": "2026-08-23T00:00:00Z",
        "resumeCondition": "fresh verification evidence for the successor"
    })
}

#[test]
fn recovery_decision_binds_predecessor_and_projects_in_outcome() {
    let directory = repository();
    let runtime = RuntimeContext {
        runtime_version: "0.2.12".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"runtime"),
    };
    let first = receipt(&directory, "create an explicit successor");
    record_recovery_decision(directory.path(), "WI-BLOCKED", &first, &runtime)
        .expect("first recovery receipt");
    let successor_contract: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-SUCCESSOR.contract.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(successor_contract["predecessorWorkItemId"], "WI-BLOCKED");
    assert_eq!(
        successor_contract["predecessorContractDigest"],
        first["predecessorContractDigest"]
    );
    let successor_contract =
        serde_json::from_value::<cockpit_protocol::Contract>(successor_contract).unwrap();
    successor_contract
        .validate()
        .expect("valid successor Contract");
    let mut second = receipt(&directory, "record a retry after the successor decision");
    second["decidedAt"] = json!("2026-08-23T00:01:00Z");
    second["decision"] = json!("retry");
    second
        .as_object_mut()
        .unwrap()
        .remove("successorWorkItemId");
    record_recovery_decision(directory.path(), "WI-BLOCKED", &second, &runtime)
        .expect("second recovery receipt is append-only");

    let decisions = fs::read_dir(directory.path().join(".ai/decisions"))
        .unwrap()
        .flatten()
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("WI-BLOCKED.recovery")
        })
        .count();
    assert_eq!(decisions, 2);
    let outcome = outcome_v2(directory.path(), "WI-BLOCKED").expect("outcome");
    assert_eq!(
        outcome.recovery_decision.as_ref().unwrap().reason,
        "record a retry after the successor decision"
    );
}

#[test]
fn recovery_rejects_foreign_runtime_and_predecessor_digest() {
    let directory = repository();
    let runtime = RuntimeContext {
        runtime_version: "0.2.12".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"runtime"),
    };
    let mut foreign = receipt(&directory, "foreign");
    foreign["repositoryId"] = json!(Digest::sha256_bytes(b"foreign"));
    assert!(record_recovery_decision(directory.path(), "WI-BLOCKED", &foreign, &runtime).is_err());
    let mut stale = receipt(&directory, "stale");
    stale["predecessorSummaryDigest"] = json!(Digest::sha256_bytes(b"stale"));
    assert!(record_recovery_decision(directory.path(), "WI-BLOCKED", &stale, &runtime).is_err());
}
