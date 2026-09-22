"""Measure the real CLI knowledge query with explicit phase boundaries."""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import time
from typing import Any

from runtime_benchmark_stats import summarize


def _finite_ms(value: Any, field: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f"{field} must be numeric")
    value = float(value)
    if value < 0:
        raise ValueError(f"{field} must be non-negative")
    return round(value, 3)


def measurement_record(
    payload: dict[str, Any],
    *,
    elapsed_ms: float,
    resource_metrics: dict[str, Any],
    phase: str,
) -> dict[str, Any]:
    projection = payload.get("projection")
    metrics = payload.get("metrics", {}).get("knowledgeQuery")
    if not isinstance(projection, dict) or not isinstance(metrics, dict):
        raise ValueError("CLI payload does not contain knowledge query metrics")
    materialization = projection.get("materialization")
    if materialization not in {"created", "rebuilt", "reused"}:
        raise ValueError("CLI payload has an invalid projection materialization")
    projection_ms = _finite_ms(metrics.get("projectionMs"), "projectionMs")
    query_ms = _finite_ms(metrics.get("queryMs"), "queryMs")
    accessed = metrics.get("candidateRecordAccessCount")
    if isinstance(accessed, bool) or not isinstance(accessed, int) or accessed < 0:
        raise ValueError("candidateRecordAccessCount must be a non-negative integer")

    result: dict[str, Any] = {
        "phase": phase,
        "elapsedMs": _finite_ms(elapsed_ms, "elapsedMs"),
        "projectionMaterialization": materialization,
        "projectionMs": projection_ms,
        "queryMs": query_ms,
        "candidateRecordAccessCount": accessed,
        "resourceUsage": resource_metrics,
    }
    if materialization == "reused":
        result["cacheValidationMs"] = projection_ms
    else:
        result["cacheValidationMs"] = None
        result["cacheValidationUnavailableReason"] = "projection_not_reused"
    return result


def _number_from_line(line: str) -> int | None:
    match = re.search(r"([0-9][0-9,]*)", line)
    if not match:
        return None
    return int(match.group(1).replace(",", ""))


def parse_resource_metrics(
    text: str,
    *,
    platform_name: str | None = None,
) -> dict[str, dict[str, Any]]:
    platform_name = platform_name or platform.system().lower()
    rss: dict[str, Any] | None = None
    read: dict[str, Any] | None = None
    for line in text.splitlines():
        lowered = line.lower()
        value = _number_from_line(line)
        if value is None:
            continue
        if "maximum resident set size" in lowered:
            rss = {"available": True, "value": value}
            if "kbytes" in lowered or "kb" in lowered:
                rss["value"] *= 1024
        elif "bytes read" in lowered:
            read = {"available": True, "value": value}
        elif "file system inputs" in lowered:
            read = {"available": True, "value": value * 512}
    unavailable_reason = (
        "host_does_not_expose_resource_metrics"
        if not text.strip()
        else "resource_metric_not_found_in_host_output"
    )
    return {
        "rssBytes": rss
        or {"available": False, "reason": unavailable_reason},
        "readBytes": read
        or {"available": False, "reason": unavailable_reason},
    }


def _phase_summary(samples: list[dict[str, Any]], phase: str) -> dict[str, Any]:
    values = [float(sample["elapsedMs"]) for sample in samples if sample["phase"] == phase]
    if not values:
        return {
            "sampleCount": 0,
            "rawElapsedMs": [],
            "percentiles": {
                "p50Ms": None,
                "p95Ms": None,
                "p99Ms": None,
                "unavailableReason": {"all": "no_samples"},
            },
        }
    stats = summarize(
        f"knowledge-query:{phase}",
        [0.0],
        valid_warm_samples_ms=values,
    )
    candidate_counts = [
        sample["candidateRecordAccessCount"]
        for sample in samples
        if sample["phase"] == phase
    ]
    percentiles = dict(stats["warm"])
    percentiles["sampleCount"] = len(values)
    return {
        "sampleCount": len(values),
        "rawElapsedMs": values,
        "percentiles": percentiles,
        "candidateRecordAccess": {
            "raw": candidate_counts,
            "min": min(candidate_counts),
            "max": max(candidate_counts),
            "average": round(sum(candidate_counts) / len(candidate_counts), 3),
        },
    }


def _phase_metric_summary(
    samples: list[dict[str, Any]],
    field: str,
    *,
    phase: str,
) -> dict[str, Any]:
    values = [
        float(sample[field])
        for sample in samples
        if sample["phase"] == phase and sample.get(field) is not None
    ]
    if not values:
        return {
            "sampleCount": 0,
            "rawMs": [],
            "percentiles": {
                "p50Ms": None,
                "p95Ms": None,
                "p99Ms": None,
                "unavailableReason": {"all": "no_reused_projection_samples"},
            },
        }
    stats = summarize(
        f"knowledge-query:{phase}:{field}",
        [0.0],
        valid_warm_samples_ms=values,
    )
    percentiles = dict(stats["warm"])
    percentiles["sampleCount"] = len(values)
    return {
        "sampleCount": len(values),
        "rawMs": values,
        "percentiles": percentiles,
    }


def summarize_measurements(samples: list[dict[str, Any]]) -> dict[str, Any]:
    if not samples:
        raise ValueError("knowledge query measurements must not be empty")
    return {
        "rawSampleCount": len(samples),
        "rawElapsedMs": [float(sample["elapsedMs"]) for sample in samples],
        "cold": _phase_summary(samples, "cold"),
        "warm": _phase_summary(samples, "warm"),
        "cacheValidation": {
            "cold": _phase_metric_summary(samples, "cacheValidationMs", phase="cold"),
            "warm": _phase_metric_summary(samples, "cacheValidationMs", phase="warm"),
        },
        "projection": {
            "cold": _phase_metric_summary(samples, "projectionMs", phase="cold"),
            "warm": _phase_metric_summary(samples, "projectionMs", phase="warm"),
        },
        "query": {
            "cold": _phase_metric_summary(samples, "queryMs", phase="cold"),
            "warm": _phase_metric_summary(samples, "queryMs", phase="warm"),
        },
    }


def _time_command(timing_path: pathlib.Path) -> list[str]:
    time_path = pathlib.Path("/usr/bin/time")
    if not time_path.is_file():
        return []
    if platform.system().lower() == "darwin":
        return [str(time_path), "-l", "-o", str(timing_path)]
    return [str(time_path), "-v", "-o", str(timing_path)]


def _invoke(binary: pathlib.Path, fixture: pathlib.Path, phase: str) -> dict[str, Any]:
    with tempfile.TemporaryDirectory(prefix="knowledge-query-time-") as directory:
        timing_path = pathlib.Path(directory) / "time.txt"
        command = _time_command(timing_path) + [
            str(binary),
            "knowledge",
            "query",
            "--repo",
            str(fixture),
        ]
        started = time.perf_counter_ns()
        completed = subprocess.run(
            command,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=300,
        )
        elapsed_ms = (time.perf_counter_ns() - started) / 1_000_000
        if completed.returncode != 0:
            detail = completed.stderr.decode("utf-8", "replace").strip()
            raise RuntimeError(f"knowledge query failed in {phase}: {detail}")
        try:
            payload = json.loads(completed.stdout)
        except json.JSONDecodeError as error:
            raise RuntimeError(f"knowledge query returned invalid JSON in {phase}") from error
        timing_text = timing_path.read_text(encoding="utf-8") if timing_path.is_file() else ""
        return measurement_record(
            payload,
            elapsed_ms=elapsed_ms,
            resource_metrics=parse_resource_metrics(timing_text),
            phase=phase,
        )


def _git(repo: pathlib.Path, *args: str) -> str:
    completed = subprocess.run(
        ["git", "-C", str(repo), *args],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        text=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.strip() or f"git command failed: {args}")
    return completed.stdout.strip()


def run(binary: pathlib.Path, repo: pathlib.Path, output: pathlib.Path, warm_samples: int) -> None:
    if not binary.is_file() or binary.is_symlink() or not os.access(binary, os.X_OK):
        raise ValueError("runtime binary must be an executable regular file")
    repo = repo.resolve()
    binary = binary.resolve()
    if binary.is_relative_to(repo):
        raise ValueError("runtime binary inside measured repository is not accepted")
    if warm_samples < 1:
        raise ValueError("warm sample count must be positive")
    revision = _git(repo, "rev-parse", "--verify", "HEAD^{commit}")
    with tempfile.TemporaryDirectory(prefix="knowledge-query-fixture-") as directory:
        fixture = pathlib.Path(directory) / "fixture"
        _git(repo, "worktree", "add", "--detach", str(fixture), revision)
        try:
            shutil.rmtree(fixture / ".ai" / "knowledge", ignore_errors=True)
            samples = [_invoke(binary, fixture, "cold")]
            samples.extend(_invoke(binary, fixture, "warm") for _ in range(warm_samples))
        finally:
            subprocess.run(
                ["git", "-C", str(repo), "worktree", "remove", "--force", str(fixture)],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
                text=True,
            )
    result = {
        "schemaVersion": 1,
        "operation": "knowledge-query-cli",
        "repository": str(repo),
        "sourceRevision": revision,
        "command": ["knowledge", "query"],
        "samples": samples,
        "summary": summarize_measurements(samples),
        "unknowns": [
            "cacheValidationMs_is_only_available_for_reused_projection_samples",
            "candidateRecordAccessCount_is_a_query_work_metric_not_a_full_memory_cost",
        ],
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=pathlib.Path)
    parser.add_argument("repo", type=pathlib.Path)
    parser.add_argument("output", type=pathlib.Path)
    parser.add_argument("warm_samples", type=int, nargs="?", default=20)
    args = parser.parse_args()
    try:
        run(args.binary, args.repo, args.output, args.warm_samples)
    except (OSError, RuntimeError, ValueError) as error:
        print(f"knowledge query benchmark failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
