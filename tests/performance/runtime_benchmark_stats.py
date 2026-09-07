"""Cold/warm sample grouping and quantile reliability for runtime_benchmark.sh.

Kept in its own module so the grouping algorithm is testable with a fixed,
hand-picked sample sequence instead of only through real subprocess timings.
"""

# A quantile needs enough samples to mean anything at its resolution: p50
# needs several points to avoid a single-sample swing, and p95 needs enough
# that the top 5% band contains at least one real observation.
MIN_SAMPLES_FOR_P50 = 5
MIN_SAMPLES_FOR_P95 = 20


def _percentile(sorted_items, fraction):
    if not sorted_items:
        return None
    index = min(len(sorted_items) - 1, max(0, int((len(sorted_items) - 1) * fraction)))
    return round(sorted_items[index], 3)


def summarize(name, raw_ms, prior_probe_calls=0):
    """Group raw_ms (original call order) into cold/warm records.

    raw_ms[0] is always the first process call and is reported as cold,
    regardless of its value relative to the rest. raw_ms[1:] are the warm
    samples; a sorted copy (not the stored rawMs) is used to compute
    percentiles, so caller-visible sample order is never lost to sorting.
    """
    if len(raw_ms) < 2:
        raise ValueError("summarize requires at least one cold and one warm sample")
    first_call_ms = raw_ms[0]
    warm_raw = list(raw_ms[1:])
    warm_sorted = sorted(warm_raw)
    warm_count = len(warm_sorted)
    p50 = _percentile(warm_sorted, 0.50) if warm_count >= MIN_SAMPLES_FOR_P50 else None
    p95 = _percentile(warm_sorted, 0.95) if warm_count >= MIN_SAMPLES_FOR_P95 else None
    warm_record = {
        "name": f"{name}.warm",
        "elapsedMs": p95 if p95 is not None else (p50 if p50 is not None else warm_sorted[-1]),
        "iterations": warm_count,
        "rawMs": warm_raw,
        "quantileMethod": "nearest-rank-on-sorted-samples",
        "p50Ms": p50,
        "p95Ms": p95,
    }
    if p50 is None:
        warm_record["p50Unreliable"] = f"insufficient_samples:{warm_count}<{MIN_SAMPLES_FOR_P50}"
    if p95 is None:
        warm_record["p95Unreliable"] = f"insufficient_samples:{warm_count}<{MIN_SAMPLES_FOR_P95}"
    cold_record = {
        "name": f"{name}.cold",
        "elapsedMs": first_call_ms,
        "iterations": 1,
        "rawMs": [first_call_ms],
        "warmupCount": 0,
        "priorProcessInvocationsOfThisCommand": prior_probe_calls,
    }
    if prior_probe_calls > 0:
        cold_record["note"] = (
            "an identity probe already invoked this command before measurement; "
            "this is the first *measured* call, not a true OS-cold-cache call"
        )
    else:
        cold_record["note"] = (
            "first process invocation of this command in this run; the binary's own "
            "pages were already OS-cached by an earlier --version/inspect/status probe"
        )
    return [cold_record, warm_record]
