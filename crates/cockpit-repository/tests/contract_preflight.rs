use cockpit_core::DecisionState;
use cockpit_repository::{
    WorkItemStartOptions, attach, checkpoint_work_item, preflight_work_item, scaffold_work_item,
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
    directory
}

#[test]
fn scaffold_preflight_is_not_ready_and_records_human_review_requirements() {
    let directory = repository();
    let receipt =
        scaffold_work_item(directory.path(), "WI-CONTRACT-SCAFFOLD", "code").expect("scaffold");
    let decision = preflight_work_item(
        directory.path(),
        &directory.path().join(&receipt.contract_path),
    )
    .expect("preflight scaffold");

    assert_eq!(decision.state, DecisionState::Yellow);
    assert_eq!(
        decision.review_state.as_deref(),
        Some("needs_human_confirmation")
    );
    let request = decision
        .human_decision_request
        .as_ref()
        .expect("structured human decision request");
    assert_eq!(request.status, "needs_human_confirmation");
    assert!(!request.question.is_empty());
    assert!(!request.resume_condition.is_empty());
    for unknown in [
        "contract_intent_missing",
        "contract_scope_missing",
        "contract_acceptance_missing",
        "human_authority_missing",
    ] {
        assert!(
            decision.unknowns.iter().any(|item| item == unknown),
            "missing {unknown}"
        );
    }
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-CONTRACT-SCAFFOLD.summary.json"),
        )
        .expect("summary"),
    )
    .expect("summary json");
    assert_eq!(summary["preflightState"], "yellow");
    assert!(summary["preflightDecisionDigest"].is_string());
    assert!(checkpoint_work_item(directory.path(), "WI-CONTRACT-SCAFFOLD").is_err());
}

#[test]
fn high_risk_scenario_coverage_stops_at_preflight_for_human_review() {
    let directory = repository();
    start_work_item_with_options(
        directory.path(),
        "WI-SCENARIO-PREFLIGHT",
        "review high risk scenario",
        "require explicit scenario evidence",
        &["crates/**".into()],
        &WorkItemStartOptions {
            risk: "high".into(),
            authority: "authorized".into(),
            out_of_scope: vec!["target/**".into()],
            acceptance_criteria: vec!["scenario gate".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    let decision = preflight_work_item(
        directory.path(),
        &directory
            .path()
            .join(".ai/work-items/active/WI-SCENARIO-PREFLIGHT.contract.json"),
    )
    .expect("preflight");
    assert_eq!(decision.state, DecisionState::Yellow);
    assert_eq!(
        decision.review_state.as_deref(),
        Some("needs_human_confirmation")
    );
    assert!(
        decision
            .unknowns
            .iter()
            .any(|unknown| unknown == "scenario_coverage_required_for_high_risk")
    );
    assert!(checkpoint_work_item(directory.path(), "WI-SCENARIO-PREFLIGHT").is_err());
}

#[test]
fn duplicate_or_unknown_contract_json_fails_before_governance_evaluation() {
    let directory = repository();
    let receipt =
        scaffold_work_item(directory.path(), "WI-CONTRACT-STRICT", "code").expect("scaffold");
    let path = directory.path().join(&receipt.contract_path);
    let original = fs::read_to_string(&path).expect("contract");

    let mut unknown: serde_json::Value = serde_json::from_str(&original).expect("json");
    unknown["untrustedInstruction"] = serde_json::json!("must not be accepted");
    fs::write(&path, serde_json::to_vec_pretty(&unknown).expect("encode")).expect("write");
    let error = preflight_work_item(directory.path(), &path).expect_err("unknown field fails");
    assert!(error.to_string().contains("unknown field"));

    fs::write(
        &path,
        format!(
            "{{\"protocolVersion\":1,\"protocolVersion\":1,{}}}",
            &original[1..]
        ),
    )
    .expect("duplicate write");
    let error = preflight_work_item(directory.path(), &path).expect_err("duplicate fails");
    assert!(error.to_string().contains("duplicate JSON object key"));
}
