//! Pure, evidence-bound classification for the portable Runtime benchmark.
//! A label alone never proves a concurrent or resident-MCP measurement.

pub const SCENARIO_NAMES: [&str; 10] = [
    "small-clean",
    "many-files-clean",
    "single-file-change",
    "multi-file-change",
    "large-file-change",
    "many-historical-wi",
    "invalid-evidence",
    "concurrent-validation-requests",
    "resident-mcp-repeat-query",
    "current-repository",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScenarioFacts {
    pub dirty: bool,
    pub tracked_file_count: usize,
    pub changed_path_count: usize,
    pub large_changed_file: bool,
    pub historical_work_item_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ScenarioEntry {
    pub name: String,
    pub status: String,
    pub reason: String,
}

fn entry(name: &str, status: &str, reason: &str) -> ScenarioEntry {
    ScenarioEntry {
        name: name.into(),
        status: status.into(),
        reason: reason.into(),
    }
}

pub fn scenario_id(name: &str) -> Result<&str, String> {
    if SCENARIO_NAMES.contains(&name) {
        Ok(name)
    } else {
        Err(format!("unknown benchmark scenario: {name}"))
    }
}

pub fn unselected_scenario_entry(name: &str) -> Result<ScenarioEntry, String> {
    scenario_id(name)?;
    Ok(entry(
        name,
        "not_measured",
        "not selected for this invocation",
    ))
}

pub fn scenario_matrix_entry(name: &str, facts: ScenarioFacts) -> Result<ScenarioEntry, String> {
    scenario_id(name)?;
    let (status, reason) = match name {
        "current-repository" => ("measured", "facts_captured"),
        "small-clean" | "many-files-clean" if facts.dirty => ("not_measured", "repository_dirty"),
        "small-clean" if facts.tracked_file_count > 100 => {
            ("not_measured", "tracked_file_count_above_100")
        }
        "small-clean" => ("measured", "clean_repository_and_scale_match"),
        "many-files-clean" if facts.tracked_file_count < 1000 => {
            ("not_measured", "tracked_file_count_below_1000")
        }
        "many-files-clean" => ("measured", "clean_repository_and_scale_match"),
        "single-file-change" if facts.changed_path_count != 1 => {
            ("not_measured", "changed_path_count_not_one")
        }
        "single-file-change" => ("measured", "one_changed_path"),
        "multi-file-change" if facts.changed_path_count < 2 => {
            ("not_measured", "changed_path_count_below_two")
        }
        "multi-file-change" => ("measured", "multiple_changed_paths"),
        "large-file-change" if !facts.large_changed_file => {
            ("not_measured", "no_changed_file_at_least_1MiB")
        }
        "large-file-change" => ("measured", "changed_file_at_least_1MiB"),
        "many-historical-wi" if facts.historical_work_item_count < 100 => {
            ("not_measured", "historical_work_item_count_below_100")
        }
        "many-historical-wi" => ("measured", "historical_work_item_scale_match"),
        "invalid-evidence" if !facts.dirty => {
            ("not_measured", "repository_not_dirty_with_invalid_evidence")
        }
        "invalid-evidence" if facts.changed_path_count < 1 => {
            ("not_measured", "invalid_evidence_change_not_observed")
        }
        "invalid-evidence" => (
            "measured",
            "evidence_change_requires_runtime_freshness_decision",
        ),
        "concurrent-validation-requests" => {
            ("not_measured", "harness_does_not_measure_concurrency")
        }
        "resident-mcp-repeat-query" => ("not_measured", "harness_does_not_measure_resident_mcp"),
        _ => unreachable!("scenario_id validates all names"),
    };
    Ok(entry(name, status, reason))
}
