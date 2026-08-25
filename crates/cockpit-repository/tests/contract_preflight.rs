use cockpit_core::DecisionState;
use cockpit_git::GitRepository;
use cockpit_repository::{
    WorkItemStartOptions, attach, checkpoint_work_item, preflight_work_item, scaffold_work_item,
    snapshot_digest, start_work_item_with_options,
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

fn repository_snapshot_digest(path: &std::path::Path) -> cockpit_core::Digest {
    let snapshot = GitRepository::discover(path)
        .expect("discover")
        .snapshot()
        .expect("snapshot");
    snapshot_digest(&snapshot).expect("snapshot digest")
}

fn set_operation(path: &std::path::Path, work_item_id: &str, operation: &str) {
    let contract_path = path
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract")).expect("contract");
    contract["operation"] = serde_json::json!(operation);
    fs::write(
        contract_path,
        serde_json::to_vec_pretty(&contract).expect("encode contract"),
    )
    .expect("write operation");
}

fn write_capabilities(path: &std::path::Path, mappings: serde_json::Value) {
    let project = path.join(".ai/project");
    fs::create_dir_all(&project).expect("project directory");
    let value = serde_json::json!({
        "schemaVersion": 1,
        "repositoryId": cockpit_repository::repository_id(path),
        "repositorySnapshotDigest": repository_snapshot_digest(path),
        "capabilities": ["documentation", "ai_governance"],
        "nonCapabilities": ["physical_operation"],
        "criticalDomains": ["release"],
        "operationMappings": mappings
    });
    fs::write(
        project.join("capabilities.json"),
        serde_json::to_vec_pretty(&value).expect("encode capabilities"),
    )
    .expect("write capabilities");
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

#[test]
fn explicit_operation_uses_only_a_valid_repository_mapping() {
    let directory = repository();
    start_work_item_with_options(
        directory.path(),
        "WI-CAPABILITY-MAPPED",
        "documentation change",
        "update docs",
        &["docs/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["docs check".into()],
            ..Default::default()
        },
    )
    .expect("start");
    set_operation(
        directory.path(),
        "WI-CAPABILITY-MAPPED",
        "documentation.modify",
    );
    write_capabilities(
        directory.path(),
        serde_json::json!({"documentation.modify": ["documentation"]}),
    );
    let decision = preflight_work_item(
        directory.path(),
        &directory
            .path()
            .join(".ai/work-items/active/WI-CAPABILITY-MAPPED.contract.json"),
    )
    .expect("preflight");
    assert!(
        !decision
            .unknowns
            .iter()
            .any(|unknown| unknown.starts_with("project_capability_mapping_"))
    );
    assert_ne!(decision.state, DecisionState::Red);
}

#[test]
fn missing_or_insufficient_operation_mapping_requires_review() {
    for (operation, mappings, expected) in [
        (
            "release.publish",
            serde_json::json!({"documentation.modify": ["documentation"]}),
            "project_capability_mapping_missing",
        ),
        (
            "release.publish",
            serde_json::json!({"release.publish": ["unlisted"]}),
            "project_capability_mapping_insufficient",
        ),
        (
            "release.publish",
            serde_json::json!({"release.publish": ["physical_operation"]}),
            "project_capability_mapping_conflict",
        ),
    ] {
        let directory = repository();
        start_work_item_with_options(
            directory.path(),
            "WI-CAPABILITY-MAPPING",
            "release text",
            "release change",
            &["docs/**".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                acceptance_criteria: vec!["release check".into()],
                ..Default::default()
            },
        )
        .expect("start");
        set_operation(directory.path(), "WI-CAPABILITY-MAPPING", operation);
        write_capabilities(directory.path(), mappings);
        let decision = preflight_work_item(
            directory.path(),
            &directory
                .path()
                .join(".ai/work-items/active/WI-CAPABILITY-MAPPING.contract.json"),
        )
        .expect("preflight");
        assert!(decision.unknowns.iter().any(|unknown| unknown == expected));
        assert_ne!(decision.state, DecisionState::Green);
    }
}

#[test]
fn intent_cannot_satisfy_a_missing_mapping_and_legacy_contracts_remain_compatible() {
    let directory = repository();
    start_work_item_with_options(
        directory.path(),
        "WI-CAPABILITY-MISSING",
        "documentation.modify",
        "the prose must not authorize the operation",
        &["docs/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["docs check".into()],
            ..Default::default()
        },
    )
    .expect("start");
    set_operation(
        directory.path(),
        "WI-CAPABILITY-MISSING",
        "documentation.modify",
    );
    let decision = preflight_work_item(
        directory.path(),
        &directory
            .path()
            .join(".ai/work-items/active/WI-CAPABILITY-MISSING.contract.json"),
    )
    .expect("preflight");
    assert!(
        decision
            .unknowns
            .iter()
            .any(|unknown| unknown == "project_capabilities_missing")
    );
    assert!(
        decision
            .unknowns
            .iter()
            .any(|unknown| unknown == "project_capability_mapping_unknown")
    );
    assert_ne!(decision.state, DecisionState::Green);

    let legacy = repository();
    start_work_item_with_options(
        legacy.path(),
        "WI-LEGACY-NO-OPERATION",
        "intent prose",
        "legacy contract",
        &["docs/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["legacy check".into()],
            ..Default::default()
        },
    )
    .expect("legacy start");
    let legacy_decision = preflight_work_item(
        legacy.path(),
        &legacy
            .path()
            .join(".ai/work-items/active/WI-LEGACY-NO-OPERATION.contract.json"),
    )
    .expect("legacy preflight");
    assert!(
        !legacy_decision
            .unknowns
            .iter()
            .any(|unknown| unknown.starts_with("project_"))
    );
}
