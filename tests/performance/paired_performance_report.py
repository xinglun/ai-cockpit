#!/usr/bin/env python3
"""Compare paired schema-2 Runtime captures without inventing evidence.

The collector deliberately emits one file per repository/scenario.  This
report joins those files, retains the raw warm samples, and keeps Runtime
latency separate from development-cycle cost.  A comparison is release-grade
only when every compared operation has at least 100 valid warm samples.
"""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
from typing import Any, Iterable

from runtime_benchmark_stats import MIN_VALID_WARM_SAMPLES, _nearest_rank


def _digest(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value.startswith("sha256:"):
        raise ValueError(f"{field} must be a sha256 digest")
    suffix = value.removeprefix("sha256:")
    if len(suffix) != 64 or any(char not in "0123456789abcdefABCDEF" for char in suffix):
        raise ValueError(f"{field} must be a sha256 digest")
    return value


def _capture(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"capture unreadable: {path}: {error}") from error
    if not isinstance(value, dict) or value.get("schemaVersion") != 2:
        raise ValueError(f"capture is not schema 2: {path}")
    for field in ("runtimeDigest", "repositoryId", "binaryDigest"):
        _digest(value.get(field), f"{path}:{field}")
    if not isinstance(value.get("runtimeVersion"), str) or not value["runtimeVersion"].strip():
        raise ValueError(f"capture runtimeVersion missing: {path}")
    if not isinstance(value.get("scenario"), str) or not value["scenario"].strip():
        raise ValueError(f"capture scenario missing: {path}")
    if not isinstance(value.get("samples"), list) or not value["samples"]:
        raise ValueError(f"capture samples missing: {path}")
    return value


def _captures(paths: Iterable[str | Path]) -> list[dict[str, Any]]:
    result = [_capture(Path(path)) for path in paths]
    scenarios = [item["scenario"] for item in result]
    if len(set(scenarios)) != len(scenarios):
        raise ValueError("a capture set contains duplicate scenarios")
    return result


def _sample_map(capture: dict[str, Any], label: str) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for sample in capture["samples"]:
        if not isinstance(sample, dict) or not isinstance(sample.get("name"), str):
            raise ValueError(f"{label} has a malformed sample")
        if sample["name"] in result:
            raise ValueError(f"{label} has duplicate operation: {sample['name']}")
        result[sample["name"]] = sample
    return result


def _warm(sample: dict[str, Any], label: str) -> list[float]:
    warm = sample.get("validWarmSamplesMs")
    if not isinstance(warm, list) or any(
        isinstance(value, bool)
        or not isinstance(value, (int, float))
        or not math.isfinite(value)
        or value < 0
        for value in warm or []
    ):
        raise ValueError(f"{label} has malformed validWarmSamplesMs")
    if len(warm) < MIN_VALID_WARM_SAMPLES:
        raise ValueError(
            f"{label} has insufficient valid warm samples: "
            f"{len(warm)}<{MIN_VALID_WARM_SAMPLES}"
        )
    return [round(float(value), 3) for value in warm]


def _identity(capture: dict[str, Any], label: str) -> dict[str, Any]:
    environment = capture.get("environment")
    if not isinstance(environment, dict):
        raise ValueError(f"{label} environment is missing")
    comparison_key = environment.get("comparisonKey")
    toolchain = environment.get("toolchain")
    if not isinstance(comparison_key, str) or comparison_key.startswith("unavailable:"):
        raise ValueError(f"{label} environment comparison key is unavailable")
    if not isinstance(toolchain, dict):
        raise ValueError(f"{label} toolchain identity is missing")
    return {
        "repositoryId": capture["repositoryId"],
        "comparisonKey": comparison_key,
        "repository": environment.get("repository"),
        "toolchain": toolchain,
    }


def _assert_pair_identity(baseline: list[dict[str, Any]], candidate: list[dict[str, Any]]) -> dict[str, Any]:
    if {item["scenario"] for item in baseline} != {item["scenario"] for item in candidate}:
        raise ValueError("baseline and candidate scenario sets differ")
    baseline_toolchain = _identity(baseline[0], "baseline")["toolchain"]
    candidate_toolchain = _identity(candidate[0], "candidate")["toolchain"]
    if baseline_toolchain != candidate_toolchain:
        raise ValueError("baseline/candidate identity mismatch: toolchain")
    baseline_by_scenario = {item["scenario"]: item for item in baseline}
    candidate_by_scenario = {item["scenario"]: item for item in candidate}
    scenario_identities = {}
    for scenario in sorted(baseline_by_scenario):
        before = _identity(baseline_by_scenario[scenario], f"baseline:{scenario}")
        after = _identity(candidate_by_scenario[scenario], f"candidate:{scenario}")
        for field in ("repositoryId", "comparisonKey"):
            if before[field] != after[field]:
                raise ValueError(f"baseline/candidate identity mismatch for {scenario}: {field}")
        scenario_identities[scenario] = {
            "repositoryId": before["repositoryId"],
            "comparisonKey": before["comparisonKey"],
            "baselineRepository": before["repository"],
            "candidateRepository": after["repository"],
        }
    return {
        "toolchain": baseline_toolchain,
        "scenarios": scenario_identities,
    }


def _percentiles(values: list[float]) -> dict[str, float]:
    return {
        "p50Ms": _nearest_rank(values, 0.50),
        "p95Ms": _nearest_rank(values, 0.95),
    }


def _diagnostic_overhead(
    diagnostics_off: list[dict[str, Any]], diagnostics_on: list[dict[str, Any]]
) -> dict[str, Any]:
    _assert_pair_identity(diagnostics_off, diagnostics_on)
    off_by_scenario = {item["scenario"]: item for item in diagnostics_off}
    on_by_scenario = {item["scenario"]: item for item in diagnostics_on}
    comparisons = []
    for scenario in sorted(off_by_scenario):
        off_samples = _sample_map(off_by_scenario[scenario], f"diagnostics-off:{scenario}")
        on_samples = _sample_map(on_by_scenario[scenario], f"diagnostics-on:{scenario}")
        if set(off_samples) != set(on_samples):
            raise ValueError(f"diagnostic operation sets differ for scenario: {scenario}")
        for operation in sorted(off_samples):
            off = _warm(off_samples[operation], f"diagnostics-off:{scenario}:{operation}")
            on = _warm(on_samples[operation], f"diagnostics-on:{scenario}:{operation}")
            off_q = _percentiles(off)
            on_q = _percentiles(on)
            comparisons.append(
                {
                    "scenario": scenario,
                    "operation": operation,
                    "rawWarmSamplesMs": {"off": off, "on": on},
                    "p50P95Ms": {"off": off_q, "on": on_q},
                    "overheadMs": {
                        key: round(on_q[key] - off_q[key], 3) for key in off_q
                    },
                }
            )
    return {
        "available": True,
        "basis": "paired same-binary diagnostics=off versus diagnostics=on captures",
        "comparisons": comparisons,
    }


def build_report(
    baseline_captures: list[dict[str, Any]],
    candidate_captures: list[dict[str, Any]],
    *,
    noise_budget_ms: float | None = None,
    diagnostics_off: list[dict[str, Any]] | None = None,
    diagnostics_on: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    if noise_budget_ms is not None and (
        isinstance(noise_budget_ms, bool)
        or not isinstance(noise_budget_ms, (int, float))
        or not math.isfinite(noise_budget_ms)
        or noise_budget_ms < 0
    ):
        raise ValueError("noise budget must be a finite non-negative number")
    identity = _assert_pair_identity(baseline_captures, candidate_captures)
    baseline_by_scenario = {item["scenario"]: item for item in baseline_captures}
    candidate_by_scenario = {item["scenario"]: item for item in candidate_captures}
    comparisons: list[dict[str, Any]] = []
    for scenario in sorted(baseline_by_scenario):
        baseline_samples = _sample_map(baseline_by_scenario[scenario], f"baseline:{scenario}")
        candidate_samples = _sample_map(candidate_by_scenario[scenario], f"candidate:{scenario}")
        if set(baseline_samples) != set(candidate_samples):
            raise ValueError(f"operation sets differ for scenario: {scenario}")
        for operation in sorted(baseline_samples):
            before = _warm(baseline_samples[operation], f"baseline:{scenario}:{operation}")
            after = _warm(candidate_samples[operation], f"candidate:{scenario}:{operation}")
            before_quantiles = _percentiles(before)
            after_quantiles = _percentiles(after)
            delta = {
                key: round(after_quantiles[key] - before_quantiles[key], 3)
                for key in before_quantiles
            }
            if noise_budget_ms is None:
                decision = "unknown"
                reason = "noise_budget_unavailable"
            elif all(value <= -noise_budget_ms for value in delta.values()):
                decision = "improved"
                reason = "both_percentiles_clear_noise_budget"
            elif any(value > noise_budget_ms for value in delta.values()):
                decision = "regressed"
                reason = "percentile_exceeds_noise_budget"
            else:
                decision = "within_noise"
                reason = "delta_within_noise_budget"
            comparisons.append(
                {
                    "scenario": scenario,
                    "operation": operation,
                    "validWarmSampleCount": {
                        "baseline": len(before),
                        "candidate": len(after),
                    },
                    "rawWarmSamplesMs": {"baseline": before, "candidate": after},
                    "p50P95Ms": {"baseline": before_quantiles, "candidate": after_quantiles},
                    "deltaMs": delta,
                    "decision": decision,
                    "reason": reason,
                }
            )
    decisions = {item["decision"] for item in comparisons}
    if "regressed" in decisions:
        status = "regressed"
    elif "unknown" in decisions:
        status = "unknown"
    else:
        status = "compared"
    if (diagnostics_off is None) != (diagnostics_on is None):
        raise ValueError("diagnostics off and on captures must be provided together")
    overhead = (
        _diagnostic_overhead(diagnostics_off, diagnostics_on)
        if diagnostics_off is not None and diagnostics_on is not None
        else {
            "available": False,
            "reason": "provide otherwise identical diagnostics=on and diagnostics=off captures separately",
        }
    )
    return {
        "schemaVersion": 1,
        "kind": "paired-performance-comparison",
        "status": status,
        "validWarmSampleMinimum": MIN_VALID_WARM_SAMPLES,
        "noiseBudgetMs": noise_budget_ms,
        "identity": identity,
        "scenarios": sorted({item["scenario"] for item in baseline_captures}),
        "comparisons": comparisons,
        "cacheInvalidationReasons": {
            "baseline": [item.get("cacheInvalidationReasons") for item in baseline_captures],
            "candidate": [item.get("cacheInvalidationReasons") for item in candidate_captures],
        },
        "diagnostics": {
            "baselineModes": [item.get("diagnostics", {}).get("mode") for item in baseline_captures],
            "candidateModes": [item.get("diagnostics", {}).get("mode") for item in candidate_captures],
            "onOffOverhead": overhead,
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", action="append", required=True, help="schema-2 baseline capture (repeat per scenario)")
    parser.add_argument("--candidate", action="append", required=True, help="schema-2 candidate capture (repeat per scenario)")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--noise-budget-ms", type=float)
    parser.add_argument("--diagnostics-off", action="append", help="same-binary diagnostics=off capture (repeat per scenario)")
    parser.add_argument("--diagnostics-on", action="append", help="same-binary diagnostics=on capture (repeat per scenario)")
    args = parser.parse_args()
    try:
        report = build_report(
            _captures(args.baseline),
            _captures(args.candidate),
            noise_budget_ms=args.noise_budget_ms,
            diagnostics_off=_captures(args.diagnostics_off) if args.diagnostics_off else None,
            diagnostics_on=_captures(args.diagnostics_on) if args.diagnostics_on else None,
        )
    except ValueError as error:
        parser.error(str(error))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"state": report["status"], "comparisons": len(report["comparisons"])}, sort_keys=True))
    return 0 if report["status"] in {"compared", "unknown"} else 1


if __name__ == "__main__":
    raise SystemExit(main())
