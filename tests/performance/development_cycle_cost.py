"""Keep development-cycle costs separate from Runtime latency samples.

The input is a small, append-only capture produced by a host or fixture.  The
tool deliberately does not invent provider timings: an absent stage is an
explicit unavailable result, and agent-operation/preflight-rejection counts
remain separate dimensions rather than being folded into elapsed time.
"""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
from typing import Any


STAGES = (
    "contract_to_reviewable",
    "verification_to_finish",
    "post_merge_cleanup",
)


def _number(value: Any) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError("cycle samples must be finite non-negative numbers")
    value = float(value)
    if not math.isfinite(value) or value < 0:
        raise ValueError("cycle samples must be finite non-negative numbers")
    return round(value, 3)


def _nearest_rank(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    index = max(0, math.ceil(fraction * len(ordered)) - 1)
    return ordered[index]


def summarize_stage(stage: str, records: list[dict[str, Any]]) -> dict[str, Any]:
    if stage not in STAGES:
        raise ValueError(f"unknown development-cycle stage: {stage}")
    raw = [_number(record["elapsedMs"]) for record in records if record.get("valid", True)]
    result: dict[str, Any] = {
        "stage": stage,
        "sourceRecords": records,
        "rawSamplesMs": raw,
        "rawSampleCount": len(raw),
        "agentOperations": [record.get("agentOperations") for record in records],
        "preflightRejects": [record.get("preflightRejects") for record in records],
    }
    agent_values = [record.get("agentOperations") for record in records]
    reject_values = [record.get("preflightRejects") for record in records]
    result["agentOperationsBinding"] = (
        {"available": True, "values": agent_values}
        if all(isinstance(value, int) and not isinstance(value, bool) and value >= 0 for value in agent_values)
        else {"available": False, "reason": "agent operation count not persisted for one or more captures"}
    )
    result["preflightRejectsBinding"] = (
        {"available": True, "values": reject_values}
        if all(isinstance(value, int) and not isinstance(value, bool) and value >= 0 for value in reject_values)
        else {"available": False, "reason": "preflight rejection count not persisted for one or more captures"}
    )
    if not raw:
        result["available"] = False
        result["reason"] = "no_valid_stage_samples"
        result["p50Ms"] = None
        result["p95Ms"] = None
        return result
    result.update(
        {
            "available": True,
            "p50Ms": _nearest_rank(raw, 0.50),
            "p95Ms": _nearest_rank(raw, 0.95),
        }
    )
    return result


def build_report(document: dict[str, Any]) -> dict[str, Any]:
    environment = document.get("environment")
    if not isinstance(environment, dict):
        raise ValueError("environment identity is required")
    captures = document.get("captures", [])
    if not isinstance(captures, list):
        raise ValueError("captures must be an array")
    grouped: dict[str, list[dict[str, Any]]] = {stage: [] for stage in STAGES}
    for record in captures:
        if not isinstance(record, dict) or record.get("stage") not in grouped:
            raise ValueError("each capture must name one known stage")
        grouped[record["stage"]].append(record)
    stages = [
        summarize_stage(stage, grouped[stage]) if grouped[stage] else {
            "stage": stage,
            "sourceRecords": [],
            "available": False,
            "reason": "stage_not_captured",
            "rawSamplesMs": [],
            "rawSampleCount": 0,
            "p50Ms": None,
            "p95Ms": None,
            "agentOperations": [],
            "preflightRejects": [],
            "agentOperationsBinding": {"available": False, "reason": "stage_not_captured"},
            "preflightRejectsBinding": {"available": False, "reason": "stage_not_captured"},
        }
        for stage in STAGES
    ]
    return {
        "schemaVersion": 1,
        "kind": "development-cycle-cost",
        "environment": environment,
        "stages": stages,
        "measurementBoundary": {
            "contractToReviewable": "from Contract creation to the first state where a Draft PR may be created",
            "verificationToFinish": "from verification start to a successful finish boundary",
            "postMergeCleanup": "from reviewed merge observation to exact resource cleanup",
            "overlap": "not inferred; each stage is reported from its own capture",
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    source = json.loads(args.input.read_text(encoding="utf-8"))
    report = build_report(source)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
