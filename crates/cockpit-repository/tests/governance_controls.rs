use cockpit_core::Digest;
use cockpit_protocol::{Contract, RuntimeContext};
use cockpit_repository::{
    FINAL_DIMENSIONS, validate_acceptance_evidence_values, validate_final_dimensions_value,
    validate_intent_alignment_values, validate_scenario_coverage_values,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

fn contract(risk: &str, intent: &str, acceptance: Vec<&str>) -> Contract {
    Contract {
        protocol_version: 1,
        contract_version: None,
        repository_id: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .parse()
            .unwrap(),
        work_item_id: "WI-122".into(),
        mode: Some("implementation".into()),
        title: None,
        state: Some("implementation_active".into()),
        created_at: None,
        intent: intent.into(),
        goal: "test".into(),
        scope: vec!["crates/**".into()],
        out_of_scope: vec![],
        risk: risk.into(),
        authority: "authorized".into(),
        acceptance_criteria: acceptance.into_iter().map(str::to_owned).collect(),
        required_evidence_classes: vec!["verification".into()],
        sources: vec![],
        verification: vec![],
        base_revision: "abc".into(),
        project_profile_digest: Digest::sha256_bytes(b"profile"),
        repository_snapshot_digest: Digest::sha256_bytes(b"snapshot"),
        operation: None,
        governance_policy: None,
        problem_statement: None,
        risk_assessment: None,
        agent_capability: None,
        execution_decision: None,
        destructive_change_policy: None,
        rollback_note: None,
        rollback_plan: None,
        unknowns: Vec::new(),
        not_codable: None,
        scenario_coverage: None,
        concurrency_boundary: None,
        checkpoint_policy: None,
        human_decision_points: None,
        documentation_impact: None,
        performance_impact: None,
        residual_risk_expectation: None,
        governance_profile: None,
        requested_operation: None,
        implementation_surface: None,
        restricted_write_approval: None,
        adoption_bootstrap_paths: Vec::new(),
    }
}

#[test]
fn high_risk_scenario_coverage_fails_closed_until_verified() {
    let contract = json!({"risk": "high"});
    let (state, unknowns, findings) = validate_scenario_coverage_values(&contract, &json!({}));
    assert_eq!(state, "blocked");
    assert!(
        unknowns
            .iter()
            .any(|item| item == "scenario_coverage_required_for_high_risk")
    );
    assert!(
        findings
            .iter()
            .any(|item| item.code == "scenario_coverage_required_for_high_risk")
    );

    let contract = json!({
        "risk": "high",
        "scenarioCoverage": [{"scenario":"tamper","required":true,"status":"verified","evidence":["tests/tamper.rs"]}]
    });
    let summary = json!({
        "scenarioCoverage": [{"scenario":"tamper","required":true,"status":"verified","evidence":["tests/tamper.rs"]}]
    });
    let (state, unknowns, findings) = validate_scenario_coverage_values(&contract, &summary);
    assert_eq!(state, "verified");
    assert!(unknowns.is_empty());
    assert!(findings.is_empty());

    let (_, _, findings) = validate_scenario_coverage_values(
        &json!({
            "risk": "high",
            "scenarioCoverage": [{"scenario":"tamper","required":true,"status":"verified","evidence":["tests/tamper.rs"]}]
        }),
        &json!({
            "scenarioCoverage": [{"scenario":"other","required":true,"status":"verified","evidence":["tests/other.rs"]}]
        }),
    );
    assert!(
        findings
            .iter()
            .any(|item| item.code == "scenario_contract_summary_mismatch")
    );
}

#[test]
fn acceptance_ids_preserve_legacy_and_validate_numbered_evidence() {
    let legacy = contract("normal", "", vec!["works"]);
    let (state, _, findings) = validate_acceptance_evidence_values(&legacy, &json!({}));
    assert_eq!(state, "not_applicable");
    assert!(findings.is_empty());

    let numbered = contract("normal", "intent", vec!["A1: works", "A2: recovers"]);
    let (state, unknowns, _) = validate_acceptance_evidence_values(&numbered, &json!({}));
    assert_eq!(state, "unknown");
    assert!(
        unknowns
            .iter()
            .any(|item| item == "acceptance_evidence_missing")
    );

    let summary = json!({"acceptanceEvidence": [
        {"acceptanceId":"A1","evidence":[{"type":"test","path":"tests/a.rs","locator":"case_a","verification":"passed"}]},
        {"acceptanceId":"A2","evidence":[{"type":"test","path":"tests/b.rs","locator":"case_b","verification":"passed"}]}
    ]});
    let (state, unknowns, findings) = validate_acceptance_evidence_values(&numbered, &summary);
    assert_eq!(state, "verified");
    assert!(unknowns.is_empty());
    assert!(findings.is_empty());
}

#[test]
fn intent_alignment_never_invents_resolution() {
    let contract = contract("normal", "human intent", vec![]);
    let (state, unknowns, _) = validate_intent_alignment_values(&contract, &json!({}));
    assert_eq!(state, "unknown");
    assert!(
        unknowns
            .iter()
            .any(|item| item == "intent_alignment_missing")
    );

    let (state, _, _) = validate_intent_alignment_values(
        &contract,
        &json!({"intentAlignment":{"state":"resolved","evidence":["decision.md"]}}),
    );
    assert_eq!(state, "resolved");
}

fn final_receipt(decision: &str) -> Value {
    let dimensions = FINAL_DIMENSIONS
        .iter()
        .map(|name| {
            (
                (*name).to_owned(),
                json!({"status": if *name == "real_adopter" || *name == "provider_evidence" { "verified" } else { "conditional" }, "evidence":[format!("evidence/{name}.json")]}),
            )
        })
        .collect::<BTreeMap<_, _>>();
    json!({
        "schemaVersion": 1,
        "repositoryId": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "workItemId": "WI-122",
        "contractDigest": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "summaryDigest": "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "runtimeVersion": "0.2.10",
        "runtimeDigest": "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        "decision": decision,
        "dimensions": dimensions,
        "limitations": []
    })
}

#[test]
fn final_dimensions_require_exact_reference_set_and_go_prerequisites() {
    let report = validate_final_dimensions_value(
        &final_receipt("GO"),
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        Some("WI-122"),
    );
    assert_eq!(report.state, "verified");
    assert_eq!(report.decision.as_deref(), Some("GO"));

    let runtime = RuntimeContext {
        runtime_version: "0.2.10".into(),
        protocol_version: 1,
        runtime_digest: "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
            .parse()
            .unwrap(),
    };
    let report = cockpit_repository::validate_final_dimensions_value_with_runtime(
        &final_receipt("GO"),
        None,
        Some("WI-122"),
        Some(&runtime),
    );
    assert_eq!(report.state, "verified");
    let foreign_runtime = RuntimeContext {
        runtime_digest: "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
            .parse()
            .unwrap(),
        ..runtime
    };
    let report = cockpit_repository::validate_final_dimensions_value_with_runtime(
        &final_receipt("GO"),
        None,
        Some("WI-122"),
        Some(&foreign_runtime),
    );
    assert_eq!(report.state, "blocked");
    assert!(
        report
            .findings
            .iter()
            .any(|item| item.code == "final_runtime_digest_mismatch")
    );

    let mut malformed = final_receipt("GO");
    malformed["dimensions"]
        .as_object_mut()
        .unwrap()
        .remove("installation");
    let report = validate_final_dimensions_value(&malformed, None, Some("WI-122"));
    assert_eq!(report.state, "blocked");
    assert!(
        report
            .unknowns
            .iter()
            .any(|item| item == "final_dimension_missing:installation")
    );

    let mut four_d = final_receipt("CONDITIONAL_GO");
    four_d["fourPillarProjection"] = json!({"4D": ["forbidden"]});
    let report = validate_final_dimensions_value(&four_d, None, Some("WI-122"));
    assert_eq!(report.state, "blocked");
    assert!(
        report
            .findings
            .iter()
            .any(|item| item.code == "ambiguous_four_dimension_field")
    );
}

#[test]
fn recording_controls_is_bounded_to_projection_fields() {
    let directory = tempdir().unwrap();
    let summary_dir = directory.path().join(".ai/work-items/active");
    fs::create_dir_all(&summary_dir).unwrap();
    let summary_path = summary_dir.join("WI-CONTROLS.summary.json");
    fs::write(
        &summary_path,
        r#"{"state":"checkpointed","checkpointCount":1}"#,
    )
    .unwrap();
    let updated = cockpit_repository::record_work_item_governance_controls(
        directory.path(),
        "WI-CONTROLS",
        &json!({"intentAlignment":{"state":"unknown"}}),
    )
    .unwrap();
    assert_eq!(updated["intentAlignment"]["state"], "unknown");
    assert_eq!(updated["state"], "checkpointed");
    let error = cockpit_repository::record_work_item_governance_controls(
        directory.path(),
        "WI-CONTROLS",
        &json!({"state":"finish_ready"}),
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("unsupported governance projection")
    );
}

#[cfg(unix)]
#[test]
fn active_control_inputs_reject_summary_symlinks() {
    use std::os::unix::fs::symlink;

    let directory = tempdir().unwrap();
    let active = directory.path().join(".ai/work-items/active");
    fs::create_dir_all(&active).unwrap();
    fs::write(
        active.join("WI-SYMLINK.contract.json"),
        serde_json::to_vec(&contract("normal", "", vec![])).unwrap(),
    )
    .unwrap();
    let real_summary = directory.path().join("summary.json");
    fs::write(&real_summary, r#"{"state":"checkpointed"}"#).unwrap();
    symlink(&real_summary, active.join("WI-SYMLINK.summary.json")).unwrap();
    let error =
        cockpit_repository::validate_work_item_governance_controls(directory.path(), "WI-SYMLINK")
            .unwrap_err();
    assert!(error.to_string().contains("regular non-symlink"));
}
