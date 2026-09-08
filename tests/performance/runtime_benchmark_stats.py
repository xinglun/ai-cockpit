"""Order-preserving grouping and explicit reliability for benchmark samples."""

from __future__ import annotations

import math
from typing import Iterable


QUANTILE_METHOD = "nearest-rank-on-sorted-samples"
PERCENTILE_MINIMUMS = {"p50": 5, "p95": 20, "p99": 100}


def _number(value: float | int) -> float:
    result = float(value)
    if not math.isfinite(result) or result < 0:
        raise ValueError("benchmark samples must be finite and non-negative")
    return round(result, 3)


def _nearest_rank(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    rank = max(1, math.ceil(fraction * len(ordered)))
    return ordered[rank - 1]


def summarize(name: str, raw_samples_ms: Iterable[float | int]) -> dict:
    if not isinstance(name, str) or not name.strip():
        raise ValueError("benchmark sample name must be non-empty")

    raw = [_number(value) for value in raw_samples_ms]
    if not raw:
        raise ValueError("benchmark samples must not be empty")

    warm = raw[1:]
    percentiles = {"p50": 0.50, "p95": 0.95, "p99": 0.99}
    values: dict[str, float | None] = {}
    reliable: dict[str, bool] = {}
    unavailable_reason: dict[str, str] = {}
    for label, fraction in percentiles.items():
        minimum = PERCENTILE_MINIMUMS[label]
        is_reliable = len(warm) >= minimum
        reliable[label] = is_reliable
        if is_reliable:
            values[label] = _nearest_rank(warm, fraction)
        else:
            values[label] = None
            unavailable_reason[label] = f"insufficient_samples:{len(warm)}<{minimum}"

    return {
        "name": name,
        "rawSamplesMs": raw,
        "sampleCount": len(raw),
        "firstMeasurementMs": raw[0],
        "warmSamplesMs": warm,
        "warm": {
            "sampleCount": len(warm),
            "quantileMethod": QUANTILE_METHOD,
            "p50Ms": values["p50"],
            "p95Ms": values["p95"],
            "p99Ms": values["p99"],
            "reliable": reliable,
            "unavailableReason": unavailable_reason,
        },
    }
