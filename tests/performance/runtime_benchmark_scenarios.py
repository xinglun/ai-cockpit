"""Evidence-bound scenario classification for the portable benchmark."""

from __future__ import annotations


def scenario_matrix_entry(
    name: str,
    *,
    dirty: bool,
    tracked_file_count: int,
    changed_path_count: int,
    large_changed_file: bool,
    historical_work_item_count: int,
) -> dict[str, str]:
    """Classify a scenario only from facts captured by the harness.

    The benchmark cannot prove concurrent requests or a resident MCP transport,
    so those scenarios must remain explicitly unmeasured. Scale thresholds are
    deliberately fixed here so a caller-provided label cannot turn an arbitrary
    repository into a misleading scenario result.
    """

    if name == "current-repository":
        return {"name": name, "status": "measured", "reason": "facts_captured"}
    if name in {"small-clean", "many-files-clean"} and dirty:
        return {"name": name, "status": "not_measured", "reason": "repository_dirty"}
    if name == "small-clean":
        if tracked_file_count > 100:
            return {"name": name, "status": "not_measured", "reason": "tracked_file_count_above_100"}
        return {"name": name, "status": "measured", "reason": "clean_repository_and_scale_match"}
    if name == "many-files-clean":
        if tracked_file_count < 1000:
            return {"name": name, "status": "not_measured", "reason": "tracked_file_count_below_1000"}
        return {"name": name, "status": "measured", "reason": "clean_repository_and_scale_match"}
    if name == "single-file-change":
        if changed_path_count != 1:
            return {"name": name, "status": "not_measured", "reason": "changed_path_count_not_one"}
        return {"name": name, "status": "measured", "reason": "one_changed_path"}
    if name == "multi-file-change":
        if changed_path_count < 2:
            return {"name": name, "status": "not_measured", "reason": "changed_path_count_below_two"}
        return {"name": name, "status": "measured", "reason": "multiple_changed_paths"}
    if name == "large-file-change":
        if not large_changed_file:
            return {"name": name, "status": "not_measured", "reason": "no_changed_file_at_least_1MiB"}
        return {"name": name, "status": "measured", "reason": "changed_file_at_least_1MiB"}
    if name == "many-historical-wi":
        if historical_work_item_count < 100:
            return {"name": name, "status": "not_measured", "reason": "historical_work_item_count_below_100"}
        return {"name": name, "status": "measured", "reason": "historical_work_item_scale_match"}
    if name == "concurrent-validation-requests":
        return {"name": name, "status": "not_measured", "reason": "harness_does_not_measure_concurrency"}
    if name == "resident-mcp-repeat-query":
        return {"name": name, "status": "not_measured", "reason": "harness_does_not_measure_resident_mcp"}
    raise ValueError(f"unknown benchmark scenario: {name}")
