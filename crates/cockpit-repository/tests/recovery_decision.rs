use cockpit_core::Digest;
use cockpit_protocol::{HumanDecision, RuntimeContext};
use cockpit_repository::{
    WorkItemStartOptions, archive_work_item, attach, checkpoint_work_item,
    close_work_item_with_structured_decision, outcome_v2, preflight_work_item,
    record_recovery_decision, render_human_outcome, repository_id, start_work_item_with_options,
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
        br#"{"schemaVersion":1,"eventId":"blocked-1","repositoryId":"REPOSITORY_ID","workItemId":"WI-BLOCKED","eventType":"blocked","timestamp":"2026-08-23T00:00:00Z","detail":"blocked for recovery"}
"#,
    )
    .unwrap();
    let events_path = directory
        .path()
        .join(".ai/work-items/active/WI-BLOCKED.events.jsonl");
    let mut events = fs::read_to_string(&events_path).unwrap();
    events = events.replace(
        "REPOSITORY_ID",
        &repository_id(directory.path()).to_string(),
    );
    fs::write(events_path, events).unwrap();
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
    let mut first = receipt(&directory, "create an explicit successor");
    first["runtimeVersion"] = json!(runtime.runtime_version);
    first["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
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

#[test]
fn superseded_predecessor_preserves_bytes_and_closes_without_current_verification() {
    let directory = repository();
    let runtime = RuntimeContext {
        runtime_version: "0.2.13".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"runtime-0.2.13"),
    };
    let mut first = receipt(&directory, "create an explicit successor");
    first["runtimeVersion"] = json!(runtime.runtime_version);
    first["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    record_recovery_decision(directory.path(), "WI-BLOCKED", &first, &runtime)
        .expect("successor receipt");
    let successor_contract = directory
        .path()
        .join(".ai/work-items/active/WI-SUCCESSOR.contract.json");
    let mut successor =
        serde_json::from_slice::<serde_json::Value>(&fs::read(&successor_contract).unwrap())
            .unwrap();
    successor["predecessorWorkItemId"] = json!("WI-BLOCKED");
    successor["predecessorContractDigest"] = first["predecessorContractDigest"].clone();
    fs::write(
        &successor_contract,
        serde_json::to_vec_pretty(&successor).unwrap(),
    )
    .unwrap();
    start_work_item_with_options(
        directory.path(),
        "WI-SUCCESSOR",
        "continue on the successor",
        "continue the bounded successor work",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["successor remains independently governed".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("activate recovery successor scaffold");

    let predecessor_contract = fs::read(
        directory
            .path()
            .join(".ai/work-items/active/WI-BLOCKED.contract.json"),
    )
    .unwrap();
    let predecessor_summary = fs::read(
        directory
            .path()
            .join(".ai/work-items/active/WI-BLOCKED.summary.json"),
    )
    .unwrap();
    let predecessor_outcome = fs::read(
        directory
            .path()
            .join(".ai/work-items/active/WI-BLOCKED.outcome.json"),
    )
    .unwrap();
    let mut supersede = receipt(&directory, "supersede the historical predecessor");
    supersede["decision"] = json!("supersede");
    supersede["decidedAt"] = json!("2026-08-23T00:02:00Z");
    supersede["runtimeVersion"] = json!(runtime.runtime_version);
    supersede["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    record_recovery_decision(directory.path(), "WI-BLOCKED", &supersede, &runtime)
        .expect("supersession receipt");

    archive_work_item(directory.path(), "WI-BLOCKED").expect("superseded archive");
    assert_eq!(
        fs::read(
            directory
                .path()
                .join(".ai/work-items/archive/WI-BLOCKED.contract.json")
        )
        .unwrap(),
        predecessor_contract
    );
    assert_eq!(
        fs::read(
            directory
                .path()
                .join(".ai/work-items/archive/WI-BLOCKED.summary.json")
        )
        .unwrap(),
        predecessor_summary
    );
    assert_eq!(
        fs::read(
            directory
                .path()
                .join(".ai/work-items/archive/WI-BLOCKED.outcome.json")
        )
        .unwrap(),
        predecessor_outcome
    );

    close_work_item_with_structured_decision(
        directory.path(),
        "WI-BLOCKED",
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "repository-owner".into(),
            reason: "historical predecessor superseded without rewriting evidence".into(),
            evidence_refs: vec![".ai/decisions/WI-BLOCKED.recovery.json".into()],
            policy_refs: vec!["docs/reference/agent-workflow.md".into()],
            decided_at: "2026-08-23T00:03:00Z".into(),
            resume_condition: Some("continue on the successor Work Item".into()),
        },
    )
    .expect("close superseded predecessor");

    let outcome = outcome_v2(directory.path(), "WI-BLOCKED").expect("historical outcome");
    assert_eq!(outcome.historical_status.as_deref(), Some("superseded"));
    assert_eq!(
        outcome.decision_state,
        Some(cockpit_core::DecisionState::Yellow)
    );
    let handoff = render_human_outcome(directory.path(), &outcome, "zh");
    assert!(handoff.starts_with("Outcome: 🟡"), "{handoff}");
}

#[test]
fn supersede_requires_a_matching_existing_successor() {
    let directory = repository();
    let runtime = RuntimeContext {
        runtime_version: "0.2.13".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"runtime-0.2.13"),
    };
    let mut missing = receipt(&directory, "supersede without successor");
    missing["decision"] = json!("supersede");
    missing["runtimeVersion"] = json!(runtime.runtime_version);
    missing["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    assert!(record_recovery_decision(directory.path(), "WI-BLOCKED", &missing, &runtime).is_err());

    let mut successor = receipt(&directory, "create successor");
    successor["runtimeVersion"] = json!(runtime.runtime_version);
    successor["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    record_recovery_decision(directory.path(), "WI-BLOCKED", &successor, &runtime)
        .expect("successor receipt");
    let successor_path = directory
        .path()
        .join(".ai/work-items/active/WI-SUCCESSOR.contract.json");
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&successor_path).unwrap()).unwrap();
    contract["predecessorContractDigest"] = json!(Digest::sha256_bytes(b"foreign"));
    fs::write(
        &successor_path,
        serde_json::to_vec_pretty(&contract).unwrap(),
    )
    .unwrap();
    let mut mismatch = receipt(&directory, "supersede mismatch");
    mismatch["decision"] = json!("supersede");
    mismatch["runtimeVersion"] = json!(runtime.runtime_version);
    mismatch["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    assert!(record_recovery_decision(directory.path(), "WI-BLOCKED", &mismatch, &runtime).is_err());
}
