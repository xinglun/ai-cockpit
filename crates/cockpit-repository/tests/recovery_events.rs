use cockpit_repository::{
    WorkItemStartOptions, attach, checkpoint_work_item, finish_work_item, preflight_work_item,
    record_verification, record_work_item_governance_controls, render_human_outcome,
    start_work_item_with_options,
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
    start_work_item_with_options(
        directory.path(),
        "WI-RECOVERY",
        "exercise recovery",
        "persist a blocked lifecycle outcome",
        &["crates/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["blocked outcome is recoverable".into()],
            required_evidence_classes: vec!["verification".into()],
            ..Default::default()
        },
    )
    .expect("start");
    let contract = directory
        .path()
        .join(".ai/work-items/active/WI-RECOVERY.contract.json");
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), "WI-RECOVERY").expect("checkpoint");
    record_work_item_governance_controls(
        directory.path(),
        "WI-RECOVERY",
        &serde_json::json!({
            "intentAlignment": {
                "state": "resolved",
                "evidence": ["crates/cockpit-repository/tests/recovery_events.rs"]
            }
        }),
    )
    .expect("intent alignment");
    directory
}

#[test]
fn failed_finish_persists_blocked_outcome_and_recovery_event_without_terminal_state() {
    let directory = repository();
    let error = finish_work_item(directory.path(), "WI-RECOVERY")
        .expect_err("missing verification must fail closed");
    assert!(error.to_string().contains("verification"));

    let outcome_path = directory
        .path()
        .join(".ai/work-items/active/WI-RECOVERY.outcome.json");
    let outcome: serde_json::Value =
        serde_json::from_slice(&fs::read(&outcome_path).unwrap()).unwrap();
    assert_eq!(outcome["state"], "blocked");
    assert_eq!(outcome["decisionState"], "red");
    assert_eq!(outcome["failedGate"], "finish.verification");
    assert!(
        outcome["recoveryCondition"]
            .as_str()
            .unwrap()
            .contains("verification")
    );
    assert_eq!(
        outcome["taskOutcomeReport"]["failedGate"],
        "finish.verification"
    );

    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-RECOVERY.summary.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(summary["state"], "checkpointed");
    let events = fs::read_to_string(
        directory
            .path()
            .join(".ai/work-items/active/WI-RECOVERY.events.jsonl"),
    )
    .unwrap();
    let event: serde_json::Value = serde_json::from_str(events.lines().next().unwrap()).unwrap();
    assert_eq!(event["eventType"], "blocked");
    assert_eq!(event["workItemId"], "WI-RECOVERY");
}

#[test]
fn valid_finish_appends_completion_after_blocked_event_without_rewriting_history() {
    let directory = repository();
    finish_work_item(directory.path(), "WI-RECOVERY").expect_err("first finish must block");
    let events_path = directory
        .path()
        .join(".ai/work-items/active/WI-RECOVERY.events.jsonl");
    let blocked_history = fs::read_to_string(&events_path).unwrap();

    record_verification(
        directory.path(),
        "WI-RECOVERY",
        &serde_json::json!({"passed": true, "workItemId": "WI-RECOVERY"}),
        "test-runtime",
        &cockpit_core::Digest::sha256_bytes(b"test-runtime"),
    )
    .expect("record verification");
    finish_work_item(directory.path(), "WI-RECOVERY").expect("finish after recovery");

    let events = fs::read_to_string(&events_path).unwrap();
    assert!(events.starts_with(&blocked_history));
    assert!(events.lines().any(|line| {
        serde_json::from_str::<serde_json::Value>(line)
            .ok()
            .and_then(|event| event["eventType"].as_str().map(str::to_owned))
            .as_deref()
            == Some("completed")
    }));
}

#[test]
fn malformed_recovery_event_is_rejected_before_archive_success() {
    let directory = repository();
    finish_work_item(directory.path(), "WI-RECOVERY").expect_err("first finish must block");
    let events_path = directory
        .path()
        .join(".ai/work-items/active/WI-RECOVERY.events.jsonl");
    fs::write(
        &events_path,
        fs::read_to_string(&events_path)
            .unwrap()
            .replace(
                "\"eventType\":\"blocked\"",
                "\"eventType\":\"made_up_success\"",
            )
            .as_bytes(),
    )
    .unwrap();
    assert!(
        cockpit_repository::validate_work_item_governance_controls(directory.path(), "WI-RECOVERY")
            .is_ok()
    );
    assert!(cockpit_repository::archive_work_item(directory.path(), "WI-RECOVERY").is_err());
}

#[test]
fn blocked_outcome_handoff_contains_recovery_facts_in_all_languages() {
    let directory = repository();
    finish_work_item(directory.path(), "WI-RECOVERY").expect_err("first finish must block");
    let outcome = cockpit_repository::outcome_v2(directory.path(), "WI-RECOVERY").expect("outcome");
    assert_eq!(outcome.failed_gate.as_deref(), Some("finish.verification"));
    for (language, marker) in [
        ("zh", "失败 gate"),
        ("ja", "失敗した gate"),
        ("en", "Failed gate"),
    ] {
        let handoff = render_human_outcome(directory.path(), &outcome, language);
        assert!(handoff.starts_with("Outcome: 🔴"), "{language}: {handoff}");
        assert!(handoff.contains(marker), "{language}: {handoff}");
        assert!(
            handoff.contains("finish.verification"),
            "{language}: {handoff}"
        );
    }
}

#[test]
fn foreign_blocked_outcome_is_not_projected_as_a_current_failure() {
    let directory = repository();
    finish_work_item(directory.path(), "WI-RECOVERY").expect_err("first finish must block");
    let outcome_path = directory
        .path()
        .join(".ai/work-items/active/WI-RECOVERY.outcome.json");
    let mut outcome: serde_json::Value =
        serde_json::from_slice(&fs::read(&outcome_path).unwrap()).unwrap();
    outcome["repositoryId"] = serde_json::json!(
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    );
    fs::write(&outcome_path, serde_json::to_vec_pretty(&outcome).unwrap()).unwrap();

    let projected = cockpit_repository::outcome_v2(directory.path(), "WI-RECOVERY").unwrap();
    assert_eq!(projected.state, cockpit_protocol::OutcomeState::NotReady);
    assert_eq!(
        projected.decision_state,
        Some(cockpit_core::DecisionState::Yellow)
    );
    assert_ne!(
        projected.failed_gate.as_deref(),
        Some("finish.verification")
    );
}

#[cfg(unix)]
#[test]
fn symlinked_blocked_outcome_is_not_trusted() {
    use std::os::unix::fs::symlink;

    let directory = repository();
    finish_work_item(directory.path(), "WI-RECOVERY").expect_err("first finish must block");
    let outcome_path = directory
        .path()
        .join(".ai/work-items/active/WI-RECOVERY.outcome.json");
    let target = directory
        .path()
        .join(".ai/work-items/active/WI-RECOVERY.blocked.json");
    fs::rename(&outcome_path, &target).unwrap();
    symlink(&target, &outcome_path).unwrap();

    let projected = cockpit_repository::outcome_v2(directory.path(), "WI-RECOVERY").unwrap();
    assert_ne!(projected.state, cockpit_protocol::OutcomeState::Verified);
    assert_ne!(
        projected.failed_gate.as_deref(),
        Some("finish.verification")
    );
}
