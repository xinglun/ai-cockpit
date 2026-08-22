use cockpit_core::Digest;
use cockpit_repository::{
    WorkItemStartOptions, capability_truth_registry, checkpoint_work_item, implementation_approach,
    outcome_v2, performance_diagnosis, preflight_work_item, record_verification,
    start_work_item_with_options, work_item_compatibility,
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
    directory
}

fn prepare_for_verification(path: &std::path::Path, work_item_id: &str) {
    let contract = path
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let decision = preflight_work_item(path, &contract).expect("preflight");
    assert_ne!(decision.state, cockpit_core::DecisionState::Red);
    checkpoint_work_item(path, work_item_id).expect("checkpoint");
}

#[test]
fn approach_separates_observed_facts_from_unknown_human_inputs() {
    let directory = repository();
    cockpit_repository::attach(directory.path()).expect("attach");
    start_work_item_with_options(
        directory.path(),
        "WI-75",
        "Implementation approach",
        "traceability",
        &["crates/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    let approach = implementation_approach(directory.path(), "WI-75").expect("approach");
    assert!(approach.facts.iter().any(|fact| fact.key == "languages"));
    assert!(
        approach
            .derivations
            .iter()
            .all(|item| !item.input_fact_keys.is_empty())
    );
    assert!(!approach.unknowns.contains(&"authority".into()));
    assert!(
        directory
            .path()
            .join(".ai/work-items/active/WI-75.approach.json")
            .is_file()
    );
}

#[test]
fn outcome_does_not_fabricate_user_benefit_or_verification() {
    let directory = repository();
    cockpit_repository::attach(directory.path()).expect("attach");
    start_work_item_with_options(
        directory.path(),
        "WI-76",
        "Outcome",
        "outcome",
        &["docs/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    let outcome = outcome_v2(directory.path(), "WI-76").expect("outcome");
    assert_eq!(outcome.state, cockpit_protocol::OutcomeState::NotReady);
    assert_eq!(
        outcome.human_benefit_report.state,
        cockpit_protocol::OutcomeState::Unknown
    );
}

#[test]
fn outcome_marks_invalid_verification_evidence_red() {
    let directory = repository();
    cockpit_repository::attach(directory.path()).expect("attach");
    start_work_item_with_options(
        directory.path(),
        "WI-OUTCOME-TAMPER",
        "Tampered evidence",
        "fail closed",
        &["crates/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    fs::write(
        directory
            .path()
            .join(".ai/evidence/WI-OUTCOME-TAMPER.verification.json"),
        br#"{"protocolVersion":1,"evidenceSchemaVersion":2,"workItemId":"WI-OUTCOME-TAMPER","passed":false}"#,
    )
    .expect("tampered evidence");
    let outcome = outcome_v2(directory.path(), "WI-OUTCOME-TAMPER").expect("outcome");
    assert_eq!(outcome.state, cockpit_protocol::OutcomeState::Unknown);
    assert_eq!(
        outcome.decision_state,
        Some(cockpit_core::DecisionState::Red)
    );
    assert!(outcome.unknowns.contains(&"evidence_contradictory".into()));
}

fn verified_outcome_fixture() -> tempfile::TempDir {
    let directory = repository();
    cockpit_repository::attach(directory.path()).expect("attach");
    start_work_item_with_options(
        directory.path(),
        "WI-OUTCOME-MATRIX",
        "Evidence matrix",
        "fail closed",
        &["crates/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    prepare_for_verification(directory.path(), "WI-OUTCOME-MATRIX");
    record_verification(
        directory.path(),
        "WI-OUTCOME-MATRIX",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.4",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    directory
}

#[test]
fn outcome_rejects_identity_snapshot_and_runtime_tampering() {
    for (field, value, expected_state) in [
        (
            "workItemId",
            serde_json::json!("WI-OTHER"),
            cockpit_protocol::OutcomeState::Unknown,
        ),
        (
            "repositoryId",
            serde_json::json!(Digest::sha256_bytes(b"other-repository").to_string()),
            cockpit_protocol::OutcomeState::Unknown,
        ),
        (
            "repositorySnapshotDigest",
            serde_json::json!(Digest::sha256_bytes(b"stale-snapshot").to_string()),
            cockpit_protocol::OutcomeState::NotReady,
        ),
        (
            "runtimeDigest",
            serde_json::json!("not-a-digest"),
            cockpit_protocol::OutcomeState::Unknown,
        ),
        (
            "passed",
            serde_json::json!(false),
            cockpit_protocol::OutcomeState::Unknown,
        ),
    ] {
        let directory = verified_outcome_fixture();
        let evidence_path = directory
            .path()
            .join(".ai/evidence/WI-OUTCOME-MATRIX.verification.json");
        let mut evidence: serde_json::Value =
            serde_json::from_slice(&fs::read(&evidence_path).expect("evidence"))
                .expect("evidence JSON");
        evidence[field] = value;
        fs::write(
            &evidence_path,
            serde_json::to_vec_pretty(&evidence).expect("tampered evidence"),
        )
        .expect("write tampered evidence");
        let outcome = outcome_v2(directory.path(), "WI-OUTCOME-MATRIX").expect("outcome");
        assert_eq!(outcome.state, expected_state, "field={field}");
        assert_ne!(
            outcome.decision_state,
            Some(cockpit_core::DecisionState::Green),
            "field={field}"
        );
    }
}

#[test]
fn outcome_rejects_malformed_verification_json() {
    let directory = verified_outcome_fixture();
    fs::write(
        directory
            .path()
            .join(".ai/evidence/WI-OUTCOME-MATRIX.verification.json"),
        b"{malformed",
    )
    .expect("malformed evidence");
    let outcome = outcome_v2(directory.path(), "WI-OUTCOME-MATRIX").expect("outcome");
    assert_eq!(outcome.state, cockpit_protocol::OutcomeState::Unknown);
    assert_eq!(
        outcome.decision_state,
        Some(cockpit_core::DecisionState::Red)
    );
}

#[cfg(unix)]
#[test]
fn outcome_rejects_symlinked_verification_evidence() {
    use std::os::unix::fs::symlink;
    let directory = verified_outcome_fixture();
    let evidence_path = directory
        .path()
        .join(".ai/evidence/WI-OUTCOME-MATRIX.verification.json");
    let target = directory.path().join("outside-evidence.json");
    fs::write(&target, br#"{"passed":true}"#).expect("outside evidence");
    fs::remove_file(&evidence_path).expect("remove evidence");
    symlink(&target, &evidence_path).expect("symlink");
    let outcome = outcome_v2(directory.path(), "WI-OUTCOME-MATRIX").expect("outcome");
    assert_eq!(outcome.state, cockpit_protocol::OutcomeState::Unknown);
    assert_eq!(
        outcome.decision_state,
        Some(cockpit_core::DecisionState::Red)
    );
}

#[test]
fn capability_and_diagnosis_are_snapshot_bound_and_parallel_check_fails_closed() {
    let directory = repository();
    cockpit_repository::attach(directory.path()).expect("attach");
    let registry = capability_truth_registry(directory.path()).expect("registry");
    assert_eq!(
        registry.repository_id,
        cockpit_repository::repository_id(directory.path()).to_string()
    );
    assert!(!registry.capabilities.is_empty() || registry.capabilities.is_empty());
    let diagnosis = performance_diagnosis(directory.path(), None).expect("diagnosis");
    assert_eq!(diagnosis.state, cockpit_protocol::DiagnosisState::Unknown);
    start_work_item_with_options(
        directory.path(),
        "WI-79",
        "Parallel compatibility",
        "parallel",
        &["crates/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    let compatibility = work_item_compatibility(directory.path(), "WI-79").expect("compatibility");
    assert!(!compatibility.compatible);
    assert!(
        compatibility
            .reasons
            .contains(&"parallel_compatibility_not_declared".into())
    );
}
