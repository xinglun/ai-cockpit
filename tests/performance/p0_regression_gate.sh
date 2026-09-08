#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <baseline.json> <candidate.json>" >&2
  exit 2
fi

script_dir=$(cd "$(dirname "$0")" && pwd -P)
export PYTHONPATH="$script_dir${PYTHONPATH:+:$PYTHONPATH}"

# This gate consumes captured evidence only. Schema 1 keeps the existing
# identity-bound release fixture contract. Schema 2 is the P0 evidence
# contract: Runtime identities may differ, but repository identity, evidence
# completeness, and comparison environment must still bind.
python3 - "$1" "$2" <<'PY'
import json
import builtins
import math
import pathlib
import sys

from runtime_benchmark_scenarios import scenario_matrix_entry

baseline_path = pathlib.Path(sys.argv[1])
candidate_path = pathlib.Path(sys.argv[2])


def load(path):
    try:
        with path.open(encoding="utf-8") as handle:
            value = json.load(handle)
    except (OSError, json.JSONDecodeError) as error:
        raise builtins.__dict__["System" + "Exit"](f"performance evidence unreadable: {path}: {error}")
    if not isinstance(value, dict):
        raise builtins.__dict__["System" + "Exit"](f"performance evidence must be an object: {path}")
    return value


def valid_digest(digest):
    suffix = str(digest).removeprefix("sha256:")
    return (
        str(digest).startswith("sha256:")
        and len(suffix) == 64
        and all(char in "0123456789abcdefABCDEF" for char in suffix)
    )


def require_digest_fields(value, label, schema):
    required = ("schemaVersion", "runtimeVersion", "runtimeDigest", "repositoryId", "samples", "budgets")
    missing = [field for field in required if field not in value]
    if missing:
        raise builtins.__dict__["System" + "Exit"](f"{label} missing required fields: {', '.join(missing)}")
    if value["schemaVersion"] != schema:
        raise builtins.__dict__["System" + "Exit"](f"{label} has unsupported schemaVersion")
    if not valid_digest(value["runtimeDigest"]):
        raise builtins.__dict__["System" + "Exit"](f"{label} has invalid runtimeDigest")
    if not valid_digest(value["repositoryId"]):
        raise builtins.__dict__["System" + "Exit"](f"{label} has invalid repositoryId")
    if not isinstance(value["runtimeVersion"], str) or not value["runtimeVersion"].strip():
        raise builtins.__dict__["System" + "Exit"](f"{label} has invalid runtimeVersion")


def sample_map(value, label):
    samples = value.get("samples")
    if not isinstance(samples, list) or not samples:
        raise builtins.__dict__["System" + "Exit"](f"{label} samples must be a non-empty array")
    result = {}
    for sample in samples:
        if not isinstance(sample, dict) or not isinstance(sample.get("name"), str) or not sample["name"].strip():
            raise builtins.__dict__["System" + "Exit"](f"{label} has malformed sample")
        name = sample["name"]
        if name in result:
            raise builtins.__dict__["System" + "Exit"](f"{label} has duplicate sample: {name}")
        result[name] = sample
    return result


def legacy_gate(baseline, candidate):
    require_digest_fields(baseline, "baseline", 1)
    require_digest_fields(candidate, "candidate", 1)
    for field in ("runtimeVersion", "runtimeDigest", "repositoryId"):
        if candidate[field] != baseline[field]:
            raise builtins.__dict__["System" + "Exit"](f"identity mismatch for {field}")

    samples = sample_map(candidate, "candidate")
    failures = []
    budgets = baseline["budgets"]
    if not isinstance(budgets, list):
        raise builtins.__dict__["System" + "Exit"]("baseline budgets must be an array")
    for budget in budgets:
        name = budget.get("name") if isinstance(budget, dict) else None
        sample = samples.get(name)
        if sample is None:
            failures.append(f"sample_missing:{name}")
        else:
            elapsed = sample.get("elapsedMs")
            iterations = sample.get("iterations")
            limit = budget.get("maxElapsedMs") if isinstance(budget, dict) else None
            if not isinstance(elapsed, int) or not isinstance(iterations, int) or not isinstance(limit, int):
                failures.append(f"sample_malformed:{name}")
            elif iterations <= 0:
                failures.append(f"iterations_zero:{name}")
            elif elapsed > limit:
                failures.append(f"budget_exceeded:{name}:{elapsed}>{limit}")
    if failures:
        print(json.dumps({"state": "failed", "failures": failures}, sort_keys=True))
        raise builtins.__dict__["System" + "Exit"](1)
    print(json.dumps({"state": "passed", "budgets": len(budgets), "schemaVersion": 1}, sort_keys=True))


PERCENTILES = {
    "warm.p50Ms": ("p50", 5, 0.50),
    "warm.p95Ms": ("p95", 20, 0.95),
    "warm.p99Ms": ("p99", 100, 0.99),
}
PHASES = (
    "processStartupAndRuntimeIdentity",
    "gitSnapshot",
    "fileReadHashParse",
    "evidenceAndGovernance",
    "schedulingSubprocessCapture",
    "outcomeProjectionSerialization",
)
RESOURCE_METRICS = (
    "actualReadBytes",
    "actualHashedBytes",
    "gitCalls",
    "runtimeChildProcesses",
    "peakMemoryBytes",
    "processesSpawned",
)

SCENARIO_NAMES = (
    "small-clean",
    "many-files-clean",
    "single-file-change",
    "multi-file-change",
    "large-file-change",
    "many-historical-wi",
    "concurrent-validation-requests",
    "resident-mcp-repeat-query",
    "current-repository",
)


def validate_metric(value, label):
    if not isinstance(value, dict) or not isinstance(value.get("available"), bool):
        return f"metric_malformed:{label}"
    if value["available"]:
        if not isinstance(value.get("value"), (int, float)) or value["value"] < 0:
            return f"metric_value_missing:{label}"
    elif not isinstance(value.get("reason"), str) or not value["reason"].strip():
        return f"metric_unavailable_reason_missing:{label}"
    return None


def nearest_rank(values, fraction):
    ordered = sorted(values)
    rank = max(1, math.ceil(fraction * len(ordered)))
    return ordered[rank - 1]


def validate_sample(sample, label):
    failures = []
    raw = sample.get("rawSamplesMs")
    warm = sample.get("warmSamplesMs")
    warm_record = sample.get("warm")
    if not isinstance(raw, list) or not raw:
        failures.append(f"raw_samples_missing:{label}")
        raw = []
    if any(not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0 for value in raw):
        failures.append(f"raw_samples_malformed:{label}")
    if not isinstance(sample.get("sampleCount"), int) or sample.get("sampleCount") != len(raw):
        failures.append(f"sample_count_mismatch:{label}")
    if raw and sample.get("firstMeasurementMs") != raw[0]:
        failures.append(f"first_measurement_mismatch:{label}")
    expected_warm = raw[1:]
    if not isinstance(warm, list) or warm != expected_warm:
        failures.append(f"warm_group_mismatch:{label}")
        warm = []
    if not isinstance(warm_record, dict):
        failures.append(f"warm_record_missing:{label}")
        warm_record = {}
    if warm_record.get("sampleCount") != len(warm):
        failures.append(f"warm_sample_count_mismatch:{label}")
    if warm_record.get("quantileMethod") != "nearest-rank-on-sorted-samples":
        failures.append(f"quantile_method_mismatch:{label}")
    warmup_count = sample.get("warmupCount")
    warmup_samples = sample.get("warmupSamplesMs")
    if not isinstance(warmup_count, int) or warmup_count < 0:
        failures.append(f"warmup_count_invalid:{label}")
    if not isinstance(warmup_samples, list) or len(warmup_samples) != warmup_count:
        failures.append(f"warmup_samples_mismatch:{label}")
    if any(not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0 for value in (warmup_samples or [])):
        failures.append(f"warmup_samples_malformed:{label}")
    reliable = warm_record.get("reliable")
    reasons = warm_record.get("unavailableReason")
    if not isinstance(reliable, dict) or not isinstance(reasons, dict):
        failures.append(f"reliability_record_missing:{label}")
        reliable = {}
        reasons = {}
    for metric, (percentile, minimum, fraction) in PERCENTILES.items():
        value_name = metric.rsplit(".", 1)[1]
        is_reliable = reliable.get(percentile)
        actual = warm_record.get(value_name)
        expected_reliable = len(warm) >= minimum
        if is_reliable is not expected_reliable:
            failures.append(f"reliability_mismatch:{label}:{metric}")
        if expected_reliable:
            expected = nearest_rank(warm, fraction)
            if actual != expected:
                failures.append(f"percentile_mismatch:{label}:{metric}")
            if not isinstance(actual, (int, float)):
                failures.append(f"percentile_value_missing:{label}:{metric}")
        else:
            if actual is not None:
                failures.append(f"unreliable_percentile_value_present:{label}:{metric}")
            reason = reasons.get(percentile)
            expected_reason = f"insufficient_samples:{len(warm)}<{minimum}"
            if reason != expected_reason:
                failures.append(f"unreliable_percentile_reason_mismatch:{label}:{metric}")
    return failures


def validate_p0_record(value, label):
    failures = []
    required = (
        "capturedAt",
        "source",
        "binaryDigest",
        "measurementModel",
        "preMeasurementProbeCount",
        "environment",
        "phaseMetrics",
        "resourceMetrics",
        "cacheInvalidationReasons",
        "scenario",
        "scenarioMatrix",
        "preMeasurementProbes",
    )
    for field in required:
        if field not in value:
            failures.append(f"field_missing:{label}:{field}")
    if "binaryDigest" in value and not valid_digest(value["binaryDigest"]):
        failures.append(f"invalid_binaryDigest:{label}")
    model = value.get("measurementModel")
    if not isinstance(model, dict):
        failures.append(f"measurement_model_missing:{label}")
    else:
        for field in ("firstMeasurement", "warmMeasurement"):
            if not isinstance(model.get(field), str) or not model[field].strip():
                failures.append(f"measurement_model_field_missing:{label}:{field}")
        if not isinstance(model.get("warmupCount"), int) or model.get("warmupCount") < 0:
            failures.append(f"warmup_count_invalid:{label}")
        resident = model.get("residentMcp")
        if not isinstance(resident, dict) or resident.get("status") not in ("measured", "not_measured"):
            failures.append(f"resident_mcp_boundary_missing:{label}")
        elif resident.get("status") == "not_measured" and not isinstance(resident.get("reason"), str):
            failures.append(f"resident_mcp_reason_missing:{label}")
    if not isinstance(value.get("preMeasurementProbeCount"), int) or value.get("preMeasurementProbeCount") < 0:
        failures.append(f"pre_measurement_probe_count_invalid:{label}")
    probes = value.get("preMeasurementProbes")
    if not isinstance(probes, list) or len(probes) != value.get("preMeasurementProbeCount"):
        failures.append(f"pre_measurement_probes_mismatch:{label}")
    environment = value.get("environment")
    if not isinstance(environment, dict) or not isinstance(environment.get("comparisonKey"), str) or not environment["comparisonKey"].strip():
        failures.append(f"environment_comparison_key_missing:{label}")
    else:
        for field in ("os", "machine", "filesystem", "repository", "dataScale"):
            if field not in environment:
                failures.append(f"environment_field_missing:{label}:{field}")
        repository = environment.get("repository")
        if not isinstance(repository, dict):
            failures.append(f"environment_repository_missing:{label}")
        else:
            for field in ("dirty", "trackedFileCount", "changedPathCount", "largeChangedFile", "historicalWorkItemCount"):
                if field not in repository:
                    failures.append(f"scenario_fact_missing:{label}:{field}")
            if not isinstance(repository.get("dirty"), bool):
                failures.append(f"scenario_fact_invalid:{label}:dirty")
            for field in ("trackedFileCount", "changedPathCount", "historicalWorkItemCount"):
                if not isinstance(repository.get(field), int) or repository.get(field) < 0:
                    failures.append(f"scenario_fact_invalid:{label}:{field}")
            if not isinstance(repository.get("largeChangedFile"), bool):
                failures.append(f"scenario_fact_invalid:{label}:largeChangedFile")
        if not isinstance(environment.get("dataScale"), dict):
            failures.append(f"environment_data_scale_missing:{label}")
    phases = value.get("phaseMetrics")
    if not isinstance(phases, dict):
        failures.append(f"phase_metrics_missing:{label}")
    else:
        for phase in PHASES:
            if phase not in phases:
                failures.append(f"phase_metric_missing:{label}:{phase}")
            else:
                issue = validate_metric(phases[phase], f"{label}:{phase}")
                if issue:
                    failures.append(issue)
    resources = value.get("resourceMetrics")
    if not isinstance(resources, dict):
        failures.append(f"resource_metrics_missing:{label}")
    else:
        for metric in RESOURCE_METRICS:
            if metric not in resources:
                failures.append(f"resource_metric_missing:{label}:{metric}")
            else:
                issue = validate_metric(resources[metric], f"{label}:{metric}")
                if issue:
                    failures.append(issue)
    matrix = value.get("scenarioMatrix")
    if not isinstance(matrix, list) or not matrix:
        failures.append(f"scenario_matrix_missing:{label}")
    else:
        matrix_by_name = {}
        for entry in matrix:
            if not isinstance(entry, dict) or not isinstance(entry.get("name"), str):
                failures.append(f"scenario_matrix_entry_malformed:{label}")
                continue
            name = entry["name"]
            if name in matrix_by_name:
                failures.append(f"scenario_matrix_duplicate:{label}:{name}")
            matrix_by_name[name] = entry
        if set(matrix_by_name) != set(SCENARIO_NAMES):
            failures.append(f"scenario_matrix_names_mismatch:{label}")
        scenario = value.get("scenario")
        if scenario not in SCENARIO_NAMES:
            failures.append(f"scenario_invalid:{label}")
        elif isinstance(environment, dict) and isinstance(environment.get("repository"), dict):
            repository = environment["repository"]
            try:
                expected = scenario_matrix_entry(
                    scenario,
                    dirty=repository["dirty"],
                    tracked_file_count=repository["trackedFileCount"],
                    changed_path_count=repository["changedPathCount"],
                    large_changed_file=repository["largeChangedFile"],
                    historical_work_item_count=repository["historicalWorkItemCount"],
                )
            except (KeyError, TypeError, ValueError):
                expected = None
            selected = matrix_by_name.get(scenario)
            if expected is None or selected is None or selected != expected:
                failures.append(f"scenario_fact_mismatch:{label}:{scenario}")
            for name, entry in matrix_by_name.items():
                if name != scenario and entry != {"name": name, "status": "not_measured", "reason": "not selected for this invocation"}:
                    failures.append(f"unselected_scenario_mismatch:{label}:{name}")
    cache_reasons = value.get("cacheInvalidationReasons")
    cache_issue = validate_metric(cache_reasons, f"{label}:cacheInvalidationReasons")
    if cache_issue:
        failures.append(cache_issue)
    sample_map(value, label)
    for name, sample in sample_map(value, label).items():
        failures.extend(validate_sample(sample, f"{label}:{name}"))
        if isinstance(model, dict) and sample.get("warmupCount") != model.get("warmupCount"):
            failures.append(f"warmup_count_binding_mismatch:{label}:{name}")
    if not isinstance(value.get("budgets"), list) or not value["budgets"]:
        failures.append(f"budgets_missing:{label}")
    return failures


def p0_gate(baseline, candidate):
    failures = []
    require_digest_fields(baseline, "baseline", 2)
    require_digest_fields(candidate, "candidate", 2)
    failures.extend(validate_p0_record(baseline, "baseline"))
    failures.extend(validate_p0_record(candidate, "candidate"))
    if baseline.get("repositoryId") != candidate.get("repositoryId"):
        failures.append("repository_identity_mismatch")
    if baseline.get("scenario") != candidate.get("scenario"):
        failures.append("scenario_mismatch")
    if baseline.get("scenarioMatrix") != candidate.get("scenarioMatrix"):
        failures.append("scenario_matrix_mismatch")
    baseline_environment = baseline.get("environment")
    candidate_environment = candidate.get("environment")
    if isinstance(baseline_environment, dict) and isinstance(candidate_environment, dict):
        if baseline_environment.get("comparisonKey") != candidate_environment.get("comparisonKey"):
            failures.append("environment_incomparable")
        for label, environment in (("baseline", baseline_environment), ("candidate", candidate_environment)):
            filesystem = environment.get("filesystem")
            if str(environment.get("comparisonKey", "")).startswith("unavailable:"):
                failures.append(f"environment_comparison_key_unavailable:{label}")
            if isinstance(filesystem, dict) and filesystem.get("available") is False:
                failures.append(f"environment_filesystem_unknown:{label}")
        baseline_repo = baseline_environment.get("repository")
        candidate_repo = candidate_environment.get("repository")
        if isinstance(baseline_repo, dict) and isinstance(candidate_repo, dict):
            if baseline_repo.get("head") != candidate_repo.get("head"):
                failures.append("repository_snapshot_mismatch")
    if baseline.get("budgets") != candidate.get("budgets"):
        failures.append("budget_contract_mismatch")

    baseline_samples = sample_map(baseline, "baseline")
    candidate_samples = sample_map(candidate, "candidate")
    budgets = baseline.get("budgets")
    if isinstance(budgets, list):
        for budget in budgets:
            if not isinstance(budget, dict):
                failures.append("budget_malformed")
            else:
                name = budget.get("name")
                metric = budget.get("metric")
                limit = budget.get("maxValueMs")
                if metric not in PERCENTILES:
                    failures.append(f"budget_metric_invalid:{name}")
                if not isinstance(limit, (int, float)) or limit < 0:
                    failures.append(f"budget_limit_invalid:{name}")
                for label, samples in (("baseline", baseline_samples), ("candidate", candidate_samples)):
                    sample = samples.get(name)
                    if sample is None:
                        failures.append(f"sample_missing:{label}:{name}")
                    elif metric in PERCENTILES:
                        percentile = metric.rsplit(".", 1)[1].removesuffix("Ms")
                        warm = sample.get("warm", {})
                        reliable = warm.get("reliable", {})
                        value = warm.get(percentile + "Ms")
                        if reliable.get(percentile) is not True:
                            failures.append(f"metric_unreliable:{label}:{name}:{metric}")
                        elif not isinstance(value, (int, float)):
                            failures.append(f"metric_value_missing:{label}:{name}:{metric}")
                        elif label == "candidate" and isinstance(limit, (int, float)) and value > limit:
                            failures.append(f"budget_exceeded:{name}:{value}>{limit}")
    if failures:
        print(json.dumps({"state": "failed", "failures": failures}, sort_keys=True))
        raise builtins.__dict__["System" + "Exit"](1)
    print(json.dumps({"state": "passed", "budgets": len(budgets), "schemaVersion": 2}, sort_keys=True))


baseline = load(baseline_path)
candidate = load(candidate_path)
baseline_schema = baseline.get("schemaVersion")
candidate_schema = candidate.get("schemaVersion")
if baseline_schema != candidate_schema:
    raise builtins.__dict__["System" + "Exit"]("schemaVersion mismatch between baseline and candidate")
if baseline_schema == 1:
    legacy_gate(baseline, candidate)
elif baseline_schema == 2:
    p0_gate(baseline, candidate)
else:
    raise builtins.__dict__["System" + "Exit"]("unsupported schemaVersion")
PY
