#!/usr/bin/env python3
"""Regression test for cold/warm sample grouping and quantile reliability guards."""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from runtime_benchmark_stats import MIN_SAMPLES_FOR_P50, MIN_SAMPLES_FOR_P95, summarize


def check(condition, message):
    if not condition:
        raise SystemExit(f"FAIL: {message}")


# A prior implementation sorted all samples before selecting the minimum as
# "cold", which silently swapped a slow first call for whichever later call
# happened to be fastest. This fixed sequence pins the first process call as
# an outlier so misclassification cannot pass unnoticed.
cold, warm = summarize("fixed-sequence", [120, 20, 22, 21])
check(cold["elapsedMs"] == 120, f"cold sample must be the first call, got {cold['elapsedMs']}")
check(cold["name"] == "fixed-sequence.cold", "cold record name")
check(warm["rawMs"] == [20, 22, 21], f"warm rawMs must preserve original call order, got {warm['rawMs']}")
check(warm["iterations"] == 3, "warm iterations count")

# Original order must survive even when the first call is the fastest one,
# not just when it is the slowest.
cold2, warm2 = summarize("ascending", [5, 100, 6, 7, 8, 9])
check(cold2["elapsedMs"] == 5, f"cold sample must be the first call, got {cold2['elapsedMs']}")
check(warm2["rawMs"] == [100, 6, 7, 8, 9], "warm rawMs preserves call order")

# Below the p50 reliability floor: no p50/p95 claim, but a real elapsedMs
# fallback is still reported so a budget gate can still fail closed.
_, warm_small = summarize("too-few", [10, 11, 12, 13])
check(warm_small["p50Ms"] is None, "p50 must be withheld under the reliability floor")
check("p50Unreliable" in warm_small, "insufficient-sample reason must be recorded")
check(warm_small["elapsedMs"] == max(warm_small["rawMs"]), "fallback elapsedMs uses the worst observed sample")

# At/above the p50 floor but below the p95 floor: p50 is reported, p95 is not.
raw_p50_only = [10] * MIN_SAMPLES_FOR_P50
_, warm_p50 = summarize("p50-only", [999] + raw_p50_only)
check(warm_p50["p50Ms"] is not None, "p50 must be available at/above its floor")
check(warm_p50["p95Ms"] is None, "p95 must stay withheld below its floor")
check("p95Unreliable" in warm_p50, "p95 insufficient-sample reason must be recorded")

# At the p95 floor: both quantiles are reported and no unreliable marker
# remains.
raw_p95 = [10] * MIN_SAMPLES_FOR_P95
_, warm_p95 = summarize("p95-floor", [999] + raw_p95)
check(warm_p95["p50Ms"] is not None, "p50 available at p95 floor")
check(warm_p95["p95Ms"] is not None, "p95 available at its floor")
check("p95Unreliable" not in warm_p95, "no unreliable marker once floor is met")

# Prior probe disclosure must be carried through untouched.
cold_probed, _ = summarize("probed", [1, 2, 3], prior_probe_calls=2)
check(cold_probed["priorProcessInvocationsOfThisCommand"] == 2, "prior probe count recorded")
check("identity probe" in cold_probed["note"], "probed cold record must disclose the prior probe")

cold_unprobed, _ = summarize("unprobed", [1, 2, 3])
check(cold_unprobed["priorProcessInvocationsOfThisCommand"] == 0, "no prior probes by default")
check("OS-cached" in cold_unprobed["note"], "unprobed cold record must still disclose binary-page warmth")

print("runtime_benchmark_stats: all grouping and reliability checks passed")
