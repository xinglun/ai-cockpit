//! Automated checks for collaboration-language semantic invariant 7 (see
//! docs/reference/collaboration-invariant-coverage.md): the displayed next
//! step must match the current Runtime state/policy. A general oracle over
//! every state is intractable, so this asserts a bounded subset of
//! `sourceType: "observed"` scenarios in
//! docs/reference/collaboration-scenario-matrix.json (SCN-001, SCN-002,
//! SCN-016). The structured expectations are schema-checked and the action
//! expectation is compared with an error returned by a real repository
//! operation; no final Outcome projection is fabricated here.

use cockpit_core::Digest;
use cockpit_repository::{
    WorkItemStartOptions, attach, checkpoint_work_item, finish_work_item, preflight_work_item,
    record_verification, scaffold_work_item, start_work_item_with_options,
};
use serde_json::Value;
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

fn contract(path: &std::path::Path, id: &str) -> std::path::PathBuf {
    path.join(".ai/work-items/active")
        .join(format!("{id}.contract.json"))
}

fn state_message(error: cockpit_repository::ObserverError) -> String {
    match error {
        cockpit_repository::ObserverError::State { message, .. } => message,
        other => panic!("expected ObserverError::State, got {other:?}"),
    }
}

fn scenario_matrix() -> Value {
    let matrix_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/reference/collaboration-scenario-matrix.json"
    );
    serde_json::from_slice(&fs::read(matrix_path).expect("read scenario matrix"))
        .expect("scenario matrix JSON")
}

fn expected_key_message(id: &str) -> String {
    scenario_matrix()["scenarios"]
        .as_array()
        .expect("scenarios array")
        .iter()
        .find(|entry| entry["id"] == id)
        .unwrap_or_else(|| panic!("{id} missing from collaboration-scenario-matrix.json"))["expected"]["keyMessage"]
        .as_str()
        .unwrap_or_else(|| panic!("{id} expected.keyMessage is not a string"))
        .to_owned()
}

fn scenario_entry<'a>(matrix: &'a Value, id: &str) -> &'a Value {
    matrix["scenarios"]
        .as_array()
        .expect("scenarios array")
        .iter()
        .find(|entry| entry["id"] == id)
        .unwrap_or_else(|| panic!("{id} missing from collaboration-scenario-matrix.json"))
}

fn next_action_matches(scenario: &Value, actual: &str) -> Result<(), String> {
    let id = scenario["id"].as_str().unwrap_or("unknown scenario");
    let expected = scenario["expectedAction"]["messageContains"]
        .as_str()
        .ok_or_else(|| format!("{id} expectedAction.messageContains is not a string"))?;
    if actual.contains(expected) {
        Ok(())
    } else {
        Err(format!(
            "{id} expected next action to contain {expected:?}, got {actual:?}"
        ))
    }
}

fn assert_next_action_matches(scenario: &Value, actual: &str) {
    next_action_matches(scenario, actual).unwrap_or_else(|error| panic!("{error}"));
}

fn required_field<'a>(object: &'a Value, key: &str, label: &str) -> &'a Value {
    object
        .as_object()
        .unwrap_or_else(|| panic!("{label} must be an object"))
        .get(key)
        .unwrap_or_else(|| panic!("{label}.{key} is required"))
}

fn non_empty_string<'a>(value: &'a Value, label: &str) -> &'a str {
    value
        .as_str()
        .filter(|text| !text.trim().is_empty())
        .unwrap_or_else(|| panic!("{label} must be a non-empty string"))
}

fn non_empty_string_array(value: &Value, label: &str) {
    let values = value
        .as_array()
        .unwrap_or_else(|| panic!("{label} must be an array"));
    assert!(!values.is_empty(), "{label} must not be empty");
    for (index, value) in values.iter().enumerate() {
        non_empty_string(value, &format!("{label}[{index}]"));
    }
}

fn assert_required_scenario_shape(scenario: &Value) {
    let id = non_empty_string(&scenario["id"], "scenario.id");
    assert_eq!(scenario["sourceType"], "observed", "{id} must be observed");

    let input_facts = required_field(scenario, "inputFacts", id);
    for key in ["lifecycleState", "operation"] {
        non_empty_string(
            required_field(input_facts, key, "inputFacts"),
            &format!("{id}.inputFacts.{key}"),
        );
    }

    let expected_state = required_field(scenario, "expectedState", id);
    non_empty_string(
        required_field(expected_state, "result", "expectedState"),
        &format!("{id}.expectedState.result"),
    );
    non_empty_string(
        required_field(expected_state, "lifecycleState", "expectedState"),
        &format!("{id}.expectedState.lifecycleState"),
    );
    let decision_state = required_field(expected_state, "decisionState", "expectedState");
    assert!(
        decision_state.is_null()
            || decision_state
                .as_str()
                .is_some_and(|value| !value.trim().is_empty()),
        "{id}.expectedState.decisionState must be null or a non-empty string"
    );

    let blockers = required_field(scenario, "expectedBlockers", id)
        .as_array()
        .unwrap_or_else(|| panic!("{id}.expectedBlockers must be an array"));
    assert!(
        !blockers.is_empty(),
        "{id}.expectedBlockers must not be empty"
    );
    for (index, blocker) in blockers.iter().enumerate() {
        let label = format!("{id}.expectedBlockers[{index}]");
        for key in ["code", "severity", "evidence"] {
            non_empty_string(
                required_field(blocker, key, &label),
                &format!("{label}.{key}"),
            );
        }
    }

    let action = required_field(scenario, "expectedAction", id);
    for key in ["kind", "command", "messageContains"] {
        non_empty_string(
            required_field(action, key, "expectedAction"),
            &format!("{id}.expectedAction.{key}"),
        );
    }
    for key in ["mutatesRepository", "requiresHumanDecision"] {
        assert!(
            required_field(action, key, "expectedAction").is_boolean(),
            "{id}.expectedAction.{key} must be boolean"
        );
    }

    non_empty_string_array(
        required_field(scenario, "forbiddenInferences", id),
        &format!("{id}.forbiddenInferences"),
    );

    let verification_plan = required_field(scenario, "verificationPlan", id);
    for key in ["fixture", "exercise", "productionEntryPoint"] {
        non_empty_string(
            required_field(verification_plan, key, "verificationPlan"),
            &format!("{id}.verificationPlan.{key}"),
        );
    }
    for key in ["setupSteps", "assertions"] {
        non_empty_string_array(
            required_field(verification_plan, key, "verificationPlan"),
            &format!("{id}.verificationPlan.{key}"),
        );
    }
    assert!(
        required_field(verification_plan, "noOutcomeProjection", "verificationPlan")
            .as_bool()
            .is_some_and(|value| value),
        "{id}.verificationPlan.noOutcomeProjection must be true"
    );

    assert_eq!(
        scenario["expected"]["result"], scenario["expectedState"]["result"],
        "{id} structured expectedState.result must preserve legacy expected.result"
    );
    assert_eq!(
        scenario["expected"]["keyMessage"], scenario["expectedAction"]["messageContains"],
        "{id} live action expectation must preserve legacy expected.keyMessage"
    );
}

fn repository_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// SCN-001: checkpoint attempted before start on a not_ready scaffold.
#[test]
fn scn_001_checkpoint_before_start_matches_scenario_matrix_key_message() {
    let directory = repository();
    scaffold_work_item(directory.path(), "WI-SCN-001", "code").expect("scaffold");

    let error =
        checkpoint_work_item(directory.path(), "WI-SCN-001").expect_err("checkpoint must reject");
    let message = state_message(error);
    let matrix = scenario_matrix();
    assert_next_action_matches(scenario_entry(&matrix, "SCN-001"), &message);
    assert_eq!(
        message,
        expected_key_message("SCN-001"),
        "next-action text must match docs/reference/collaboration-scenario-matrix.json SCN-001 expected.keyMessage exactly"
    );
}

/// SCN-002: start rejected when the worktree has non-governance changes
/// present before activation.
#[test]
fn scn_002_start_rejects_pre_existing_changes_matches_scenario_matrix_key_message() {
    let directory = repository();
    fs::write(directory.path().join("src-under-review.txt"), b"draft\n").expect("write draft");

    let error = start_work_item_with_options(
        directory.path(),
        "WI-SCN-002",
        "invariant 7 coverage",
        "verify next-action text for pre-existing changes",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            out_of_scope: vec!["target/**".into()],
            acceptance_criteria: vec!["start remains rejected until the worktree is clean".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect_err("start must reject a dirty worktree");
    let message = state_message(error);
    let matrix = scenario_matrix();
    assert_next_action_matches(scenario_entry(&matrix, "SCN-002"), &message);
    assert!(
        message.contains(&expected_key_message("SCN-002")),
        "next-action text must match docs/reference/collaboration-scenario-matrix.json SCN-002 expected.keyMessage substring, got {message:?}"
    );
    assert!(
        message.contains("src-under-review.txt"),
        "next-action text must name the offending path so recovery (stash, start, restore) is actionable, got {message:?}"
    );
}

/// SCN-016: finish rejected until finalize-plan binds a non-provisional
/// resource finalization context.
#[test]
fn scn_016_finish_before_finalize_plan_matches_scenario_matrix_key_message() {
    let directory = repository();
    let id = "WI-SCN-016";
    start_work_item_with_options(
        directory.path(),
        id,
        "invariant 7 coverage",
        "verify next-action text for finish before finalize-plan",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            out_of_scope: vec!["target/**".into()],
            acceptance_criteria: vec!["finish stays rejected until finalize-plan runs".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    preflight_work_item(directory.path(), &contract(directory.path(), id)).expect("preflight");
    checkpoint_work_item(directory.path(), id).expect("checkpoint");
    record_verification(
        directory.path(),
        id,
        &serde_json::json!({"passed": true}),
        "0.2.8",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");

    // Deliberately skip `plan_resource_finalization` -- that omission is
    // exactly what SCN-016 observes and what this test protects.
    let error = finish_work_item(directory.path(), id).expect_err("finish must reject");
    let message = state_message(error);
    let matrix = scenario_matrix();
    assert_next_action_matches(scenario_entry(&matrix, "SCN-016"), &message);
    assert_eq!(
        message,
        expected_key_message("SCN-016"),
        "next-action text must match docs/reference/collaboration-scenario-matrix.json SCN-016 expected.keyMessage exactly"
    );
}

/// Confirms the three scenarios above still exist verbatim in the scenario
/// matrix with `sourceType: "observed"`, so this test file cannot silently
/// drift away from the document it is meant to keep honest.
#[test]
fn scn_001_002_016_are_still_declared_observed_in_the_scenario_matrix() {
    let matrix = scenario_matrix();
    let scenarios = matrix["scenarios"].as_array().expect("scenarios array");
    for id in ["SCN-001", "SCN-002", "SCN-016"] {
        let scenario = scenarios
            .iter()
            .find(|entry| entry["id"] == id)
            .unwrap_or_else(|| panic!("{id} missing from collaboration-scenario-matrix.json"));
        assert_required_scenario_shape(scenario);
        assert_eq!(
            scenario["sourceType"], "observed",
            "{id} must remain sourceType=observed for this test to keep testing a real, previously-hit Runtime behavior"
        );
        assert!(
            scenario["invariantsExercised"]
                .as_array()
                .expect("invariantsExercised array")
                .iter()
                .any(|value| value == 7),
            "{id} must still declare invariant 7 in invariantsExercised"
        );
    }
}

#[test]
fn executable_scenario_checks_bind_facts_semantics_and_tests_without_duplicate_answers() {
    let matrix = scenario_matrix();
    let registry = matrix["executableCheckRegistry"]
        .as_object()
        .expect("executable check registry");
    let known = [
        "human_report_reason_projection",
        "human_report_entrypoint_parity",
        "human_report_summary_retains_blocker",
        "finalization_action_matches_state",
        "finalization_case_cli_mcp_typed_next_action",
        "no_history_handoff",
        "observation_boundary_bounded_retry",
        "runtime_phase_diagnostics",
    ];
    let mut referenced = std::collections::BTreeSet::new();
    for id in known {
        let check = registry.get(id).unwrap_or_else(|| panic!("{id} missing"));
        assert!(
            check["inputFacts"]
                .as_array()
                .is_some_and(|facts| !facts.is_empty())
        );
        let semantics = check["expectedSemantics"]
            .as_object()
            .unwrap_or_else(|| panic!("{id} expectedSemantics must be structured"));
        assert!(
            semantics["description"]
                .as_str()
                .is_some_and(|text| !text.is_empty())
        );
        let tokens = semantics["semanticTokens"]
            .as_array()
            .unwrap_or_else(|| panic!("{id} semanticTokens must be an array"));
        assert!(
            !tokens.is_empty(),
            "{id} must bind at least one semantic token"
        );
        assert!(
            tokens
                .iter()
                .all(|token| token.as_str().is_some_and(|value| !value.is_empty()))
        );
        let test_ref = check["test"]
            .as_str()
            .filter(|text| !text.is_empty())
            .unwrap_or_else(|| panic!("{id} test reference is missing"));
        let (path, _) = test_ref
            .split_once("::")
            .unwrap_or_else(|| panic!("{id} test reference is not path-qualified"));
        let function = test_ref
            .rsplit("::")
            .next()
            .unwrap_or_else(|| panic!("{id} test function is missing"));
        let source = fs::read_to_string(repository_root().join(path))
            .unwrap_or_else(|error| panic!("{id} test source {path} unreadable: {error}"));
        assert!(
            source.contains(&format!("fn {function}")),
            "{id} test reference must resolve to a Rust function: {test_ref}"
        );
    }
    for scenario in matrix["scenarios"].as_array().expect("scenarios array") {
        let Some(checks) = scenario.get("executableChecks") else {
            continue;
        };
        for check in checks.as_array().expect("executableChecks array") {
            let id = check.as_str().expect("check id");
            referenced.insert(id.to_owned());
            assert!(
                registry.contains_key(id),
                "scenario references unknown check {id}"
            );
            assert!(scenario["expected"]["keyMessage"].as_str().is_some());
        }
    }
    assert_eq!(
        referenced,
        known.into_iter().map(str::to_owned).collect(),
        "every executable registry check must be attached to at least one scenario"
    );
}

#[test]
fn required_scenarios_have_structured_expectations_and_live_matrix_mutation_guard() {
    let matrix = scenario_matrix();
    let scenarios = matrix["scenarios"].as_array().expect("scenarios array");
    for id in ["SCN-001", "SCN-002", "SCN-016"] {
        let scenario = scenarios
            .iter()
            .find(|entry| entry["id"] == id)
            .unwrap_or_else(|| panic!("{id} missing from collaboration-scenario-matrix.json"));
        assert_required_scenario_shape(scenario);
    }

    let directory = repository();
    scaffold_work_item(directory.path(), "WI-SCN-MUTATION", "code").expect("scaffold");
    let actual = state_message(
        checkpoint_work_item(directory.path(), "WI-SCN-MUTATION")
            .expect_err("checkpoint must reject"),
    );
    let scenario = scenarios
        .iter()
        .find(|entry| entry["id"] == "SCN-001")
        .expect("SCN-001");
    assert_next_action_matches(scenario, &actual);

    let mut mutated = scenario.clone();
    mutated["expectedAction"]["messageContains"] =
        Value::String("matrix mutation must not match runtime output".into());
    let changed_expected = mutated["expectedAction"]["messageContains"]
        .as_str()
        .expect("mutated expectedAction.messageContains");
    assert!(
        next_action_matches(&mutated, &actual).is_err() && !actual.contains(changed_expected),
        "a changed matrix expectation must be rejected by the live assertion"
    );
}
