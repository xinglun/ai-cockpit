use cockpit_core::Digest;
use cockpit_protocol::RuntimeContext;
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item_with_runtime, attach, checkpoint_work_item, finish_work_item_with_runtime,
    preflight_work_item, preflight_work_item_with_runtime, record_recovery_decision,
    record_verification_with_runtime, repository_id, run_repository_verification,
    start_work_item_with_options, work_item_status_snapshot_with_runtime,
};
use serde_json::{Value, json};
use std::fs;
use std::process::Command;

fn commit_empty_baseline(path: &std::path::Path) {
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
            .current_dir(path)
            .status()
            .expect("git baseline commit")
            .success()
    );
}

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
    commit_empty_baseline(directory.path());
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

fn current_runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "0.2.31".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"runtime-0.2.31"),
    }
}

fn active_contract_path(root: &std::path::Path, id: &str) -> std::path::PathBuf {
    root.join(format!(".ai/work-items/active/{id}.contract.json"))
}

fn record_valid_supersede(directory: &tempfile::TempDir, runtime: &RuntimeContext) {
    let mut successor = receipt(directory, "create a bound successor");
    successor["runtimeVersion"] = json!(runtime.runtime_version);
    successor["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    record_recovery_decision(directory.path(), "WI-BLOCKED", &successor, runtime)
        .expect("successor recovery");
    start_work_item_with_options(
        directory.path(),
        "WI-SUCCESSOR",
        "continue on the successor",
        "preserve predecessor recovery bindings",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["successor remains bound".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("activate successor scaffold");

    let mut supersede = receipt(directory, "supersede the bound predecessor");
    supersede["decision"] = json!("supersede");
    supersede["decidedAt"] = json!("2026-08-23T00:01:00Z");
    supersede["runtimeVersion"] = json!(runtime.runtime_version);
    supersede["runtimeDigest"] = json!(runtime.runtime_digest.to_string());
    record_recovery_decision(directory.path(), "WI-BLOCKED", &supersede, runtime)
        .expect("supersede recovery");
}

#[test]
fn valid_supersede_is_admitted_before_archive() {
    let directory = repository();
    let runtime = current_runtime();
    record_valid_supersede(&directory, &runtime);

    let status = work_item_status_snapshot_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
        .expect("status after valid supersession");
    assert!(
        status
            .safe_actions
            .iter()
            .any(|action| action == "archive_when_reviewed"),
        "valid supersession must admit archive: {status:?}"
    );

    archive_work_item_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
        .expect("admitted superseded archive");
    let manifest: Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/archive/WI-BLOCKED.archive.json"),
        )
        .expect("archive manifest"),
    )
    .expect("archive manifest JSON");
    assert_eq!(manifest["state"], "superseded");
}

#[test]
fn archive_rechecks_successor_path_after_status_admission() {
    let directory = repository();
    let runtime = current_runtime();
    record_valid_supersede(&directory, &runtime);

    let status = work_item_status_snapshot_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
        .expect("initial status");
    assert!(
        status
            .safe_actions
            .contains(&"archive_when_reviewed".into())
    );

    let successor_path = directory
        .path()
        .join(".ai/work-items/active/WI-SUCCESSOR.contract.json");
    let mut successor: Value =
        serde_json::from_slice(&fs::read(&successor_path).expect("successor Contract"))
            .expect("successor Contract JSON");
    successor["recoveryDecisionPath"] = json!(".ai/decisions/foreign.recovery.json");
    fs::write(
        &successor_path,
        serde_json::to_vec_pretty(&successor).unwrap(),
    )
    .unwrap();

    let refreshed =
        work_item_status_snapshot_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
            .expect("status after successor path mutation");
    assert!(
        !refreshed
            .safe_actions
            .contains(&"archive_when_reviewed".into())
    );

    archive_work_item_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
        .expect_err("archive must recheck the current successor recovery path");
    assert!(
        !directory
            .path()
            .join(".ai/work-items/archive/WI-BLOCKED.archive.json")
            .exists(),
        "a stale status admission must not create a manifest"
    );
    assert!(
        directory
            .path()
            .join(".ai/work-items/active/WI-BLOCKED.contract.json")
            .is_file(),
        "a stale status admission must not move predecessor evidence"
    );
}

#[test]
fn archive_rejects_successor_deletion_after_status_admission() {
    let directory = repository();
    let runtime = current_runtime();
    record_valid_supersede(&directory, &runtime);
    let status = work_item_status_snapshot_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
        .expect("initial status");
    assert!(
        status
            .safe_actions
            .contains(&"archive_when_reviewed".into())
    );

    fs::remove_file(
        directory
            .path()
            .join(".ai/work-items/active/WI-SUCCESSOR.contract.json"),
    )
    .unwrap();

    let refreshed =
        work_item_status_snapshot_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
            .expect("status after successor deletion");
    assert!(
        !refreshed
            .safe_actions
            .contains(&"archive_when_reviewed".into())
    );
    archive_work_item_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
        .expect_err("archive must recheck successor presence");
    assert!(
        !directory
            .path()
            .join(".ai/work-items/archive/WI-BLOCKED.archive.json")
            .exists()
    );
}

#[test]
fn archive_rejects_rebinding_to_another_valid_recovery_path() {
    let directory = repository();
    let runtime = current_runtime();
    record_valid_supersede(&directory, &runtime);
    let status = work_item_status_snapshot_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
        .expect("initial status");
    assert!(
        status
            .safe_actions
            .contains(&"archive_when_reviewed".into())
    );

    let original_path = directory
        .path()
        .join(".ai/decisions/WI-BLOCKED.recovery.json");
    let original: Value = serde_json::from_slice(&fs::read(&original_path).unwrap()).unwrap();
    let digest = cockpit_protocol::digest_json(&original)
        .unwrap()
        .to_string();
    let substitute_name = format!(
        "WI-BLOCKED.recovery.{}.json",
        digest.trim_start_matches("sha256:")
    );
    fs::write(
        directory
            .path()
            .join(".ai/decisions")
            .join(&substitute_name),
        serde_json::to_vec_pretty(&original).unwrap(),
    )
    .unwrap();
    let successor_path = directory
        .path()
        .join(".ai/work-items/active/WI-SUCCESSOR.contract.json");
    let mut successor: Value = serde_json::from_slice(&fs::read(&successor_path).unwrap()).unwrap();
    successor["recoveryDecisionPath"] = json!(format!(".ai/decisions/{substitute_name}"));
    fs::write(
        &successor_path,
        serde_json::to_vec_pretty(&successor).unwrap(),
    )
    .unwrap();
    let summary_path = directory
        .path()
        .join(".ai/work-items/active/WI-SUCCESSOR.summary.json");
    let mut summary: Value = serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
    summary["recoveryDecisionPath"] = json!(format!(".ai/decisions/{substitute_name}"));
    fs::write(&summary_path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();

    let refreshed =
        work_item_status_snapshot_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
            .expect("status after simultaneous rebind");
    assert!(
        !refreshed
            .safe_actions
            .contains(&"archive_when_reviewed".into())
    );
    archive_work_item_with_runtime(directory.path(), "WI-BLOCKED", &runtime).expect_err(
        "a simultaneous Contract and Summary rebind must not replace the selected receipt",
    );
    assert!(
        !directory
            .path()
            .join(".ai/work-items/archive/WI-BLOCKED.archive.json")
            .exists()
    );
    assert!(
        directory
            .path()
            .join(".ai/work-items/active/WI-BLOCKED.contract.json")
            .is_file()
    );
}

#[test]
fn supersede_ignores_an_older_successor_candidate_with_different_predecessor_evidence() {
    let directory = repository();
    let runtime = current_runtime();
    record_valid_supersede(&directory, &runtime);

    let original: Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/decisions/WI-BLOCKED.recovery.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let mut stale = original;
    stale["predecessorSummaryDigest"] = json!(Digest::sha256_bytes(b"older summary"));
    let digest = cockpit_protocol::digest_json(&stale).unwrap().to_string();
    fs::write(
        directory.path().join(format!(
            ".ai/decisions/WI-BLOCKED.recovery.{}.json",
            digest.trim_start_matches("sha256:")
        )),
        serde_json::to_vec_pretty(&stale).unwrap(),
    )
    .unwrap();

    let status = work_item_status_snapshot_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
        .expect("stale historical candidate must not hide valid supersede");
    assert!(
        status
            .safe_actions
            .contains(&"archive_when_reviewed".into())
    );
    archive_work_item_with_runtime(directory.path(), "WI-BLOCKED", &runtime)
        .expect("historical successor evidence must not create a competing current path");
}

#[test]
fn ordinary_archive_admission_still_requires_finished_verification() {
    let directory = tempfile::tempdir().expect("temporary repository");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .unwrap()
            .success()
    );
    commit_empty_baseline(directory.path());
    attach(directory.path()).expect("attach repository");
    let id = "WI-ORDINARY-ARCHIVE";
    let runtime = current_runtime();
    start_work_item_with_options(
        directory.path(),
        id,
        "verify ordinary archive admission",
        "preserve the ordinary finish boundary",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["verification precedes archive".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start ordinary Work Item");
    let contract_path = active_contract_path(directory.path(), id);
    preflight_work_item_with_runtime(directory.path(), &contract_path, &runtime)
        .expect("preflight");
    checkpoint_work_item(directory.path(), id).expect("checkpoint");

    let before = work_item_status_snapshot_with_runtime(directory.path(), id, &runtime)
        .expect("checkpointed status");
    assert!(
        !before
            .safe_actions
            .contains(&"archive_when_reviewed".into())
    );
    archive_work_item_with_runtime(directory.path(), id, &runtime)
        .expect_err("unverified checkpoint may not archive");

    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "ordinary-archive-check".into(),
            program: "git".into(),
            args: vec!["--version".into()],
            scope: vec!["src/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("run verification");
    record_verification_with_runtime(
        directory.path(),
        id,
        &serde_json::to_value(&run.receipt).unwrap(),
        &runtime,
        &run.final_snapshot,
    )
    .expect("record verification");
    finish_work_item_with_runtime(directory.path(), id, &runtime)
        .expect("finish ordinary Work Item");

    let finished = work_item_status_snapshot_with_runtime(directory.path(), id, &runtime)
        .expect("finished status");
    assert!(
        finished
            .safe_actions
            .contains(&"archive_when_reviewed".into())
    );
    archive_work_item_with_runtime(directory.path(), id, &runtime)
        .expect("ordinary finished archive");
}
