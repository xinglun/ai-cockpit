use cockpit_verification::runtime_benchmark_scenarios::{
    SCENARIO_NAMES, ScenarioFacts, scenario_matrix_entry, unselected_scenario_entry,
};

fn facts() -> ScenarioFacts {
    ScenarioFacts {
        dirty: false,
        tracked_file_count: 100,
        changed_path_count: 1,
        large_changed_file: false,
        historical_work_item_count: 99,
    }
}

#[test]
fn fixed_scenario_matrix_preserves_thresholds_and_reasons() {
    assert_eq!(SCENARIO_NAMES.len(), 10);
    let cases = [
        (
            "small-clean",
            facts(),
            "measured",
            "clean_repository_and_scale_match",
        ),
        (
            "many-files-clean",
            facts(),
            "not_measured",
            "tracked_file_count_below_1000",
        ),
        (
            "single-file-change",
            facts(),
            "measured",
            "one_changed_path",
        ),
        (
            "multi-file-change",
            facts(),
            "not_measured",
            "changed_path_count_below_two",
        ),
        (
            "large-file-change",
            facts(),
            "not_measured",
            "no_changed_file_at_least_1MiB",
        ),
        (
            "many-historical-wi",
            facts(),
            "not_measured",
            "historical_work_item_count_below_100",
        ),
        (
            "invalid-evidence",
            facts(),
            "not_measured",
            "repository_not_dirty_with_invalid_evidence",
        ),
        (
            "concurrent-validation-requests",
            facts(),
            "not_measured",
            "harness_does_not_measure_concurrency",
        ),
        (
            "resident-mcp-repeat-query",
            facts(),
            "not_measured",
            "harness_does_not_measure_resident_mcp",
        ),
        ("current-repository", facts(), "measured", "facts_captured"),
    ];
    for (name, facts, status, reason) in cases {
        let entry = scenario_matrix_entry(name, facts).expect("known scenario");
        assert_eq!(
            (
                entry.name.as_str(),
                entry.status.as_str(),
                entry.reason.as_str()
            ),
            (name, status, reason)
        );
    }
}

#[test]
fn boundary_cases_and_invalid_evidence_are_observation_only() {
    let mut case = facts();
    case.tracked_file_count = 101;
    assert_eq!(
        scenario_matrix_entry("small-clean", case).unwrap().reason,
        "tracked_file_count_above_100"
    );
    case.tracked_file_count = 1000;
    assert_eq!(
        scenario_matrix_entry("many-files-clean", case)
            .unwrap()
            .status,
        "measured"
    );
    case.dirty = true;
    assert_eq!(
        scenario_matrix_entry("many-files-clean", case)
            .unwrap()
            .reason,
        "repository_dirty"
    );
    case.changed_path_count = 0;
    assert_eq!(
        scenario_matrix_entry("invalid-evidence", case)
            .unwrap()
            .reason,
        "invalid_evidence_change_not_observed"
    );
    case.changed_path_count = 1;
    assert_eq!(
        scenario_matrix_entry("invalid-evidence", case)
            .unwrap()
            .reason,
        "evidence_change_requires_runtime_freshness_decision"
    );
    case.changed_path_count = 2;
    assert_eq!(
        scenario_matrix_entry("multi-file-change", case)
            .unwrap()
            .status,
        "measured"
    );
    case.large_changed_file = true;
    assert_eq!(
        scenario_matrix_entry("large-file-change", case)
            .unwrap()
            .status,
        "measured"
    );
    case.historical_work_item_count = 100;
    assert_eq!(
        scenario_matrix_entry("many-historical-wi", case)
            .unwrap()
            .status,
        "measured"
    );
}

#[test]
fn unknown_names_fail_closed_and_unselected_is_explicit() {
    assert_eq!(
        unselected_scenario_entry("small-clean").unwrap().reason,
        "not selected for this invocation"
    );
    assert_eq!(
        scenario_matrix_entry("typo", facts()).unwrap_err(),
        "unknown benchmark scenario: typo"
    );
    assert_eq!(
        unselected_scenario_entry("typo").unwrap_err(),
        "unknown benchmark scenario: typo"
    );
}

#[test]
fn scenario_entry_serializes_to_the_python_oracle_shape() {
    let entry = scenario_matrix_entry(
        "invalid-evidence",
        ScenarioFacts {
            dirty: true,
            tracked_file_count: 1000,
            changed_path_count: 1,
            large_changed_file: false,
            historical_work_item_count: 100,
        },
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(entry).unwrap(),
        serde_json::json!({
            "name": "invalid-evidence",
            "status": "measured",
            "reason": "evidence_change_requires_runtime_freshness_decision"
        })
    );
}
