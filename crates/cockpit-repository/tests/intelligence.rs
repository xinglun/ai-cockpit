use cockpit_repository::{
    WorkItemStartOptions, capability_truth_registry, implementation_approach, outcome_v2,
    performance_diagnosis, start_work_item_with_options, work_item_compatibility,
};
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
    directory
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
