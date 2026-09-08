# WI-662 P0 benchmark evidence plan

## Contract

- Work Item: `WI-662-p0-benchmark-evidence`
- Base: `origin/main` at `fb96ffb7c2431af7790e46956d49250e6cfa2ba4`
- Scope: `tests/performance/**`, `tests/ci/repository_gate_manifest.json`, this Work Item record, reference parity rows, and this plan.
- Out of scope: production Runtime behavior, repository-global Agent/MCP configuration, and P1-P3 optimizations.
- North Star: preserve Calibrated Human-Agent Trust while making performance evidence trustworthy.

## Hypothesis

The current benchmark sorts all samples before selecting the cold sample and
uses a schema that cannot bind warm quantiles, environment comparability, or
resource limitations. Preserving acquisition order and recording explicit
measurement boundaries will remove a measurement-validity defect without
changing the governed Runtime path.

## Implementation sequence

1. Add a small percentile/grouping module and tests for the fixed sequence
   `120, 20, 22, 21`, minimum sample counts, and no fallback from an
   unreliable percentile to another metric.
2. Update the benchmark harness to emit schema-versioned evidence containing
   raw samples, measurement model, Runtime/repository identity, environment,
   scenario metadata, phase/resource fields, and explicit unavailable reasons.
3. Extend the comparator with a versioned evidence path that permits distinct
   Runtime identities only when each record is complete and the comparison
   environment is equivalent. Keep the legacy gate and its negative tests.
4. Register the schema 2 P0 regression gate in the repository gate manifest
   and run its manifest regression test so hosted quality executes the new
   evidence contract.
5. Run focused tests, docs/parity checks, and one sanity benchmark. Report
   limitations and avoid performance claims until the scenario matrix has
   enough samples for the declared percentiles.

## Verification

- `bash tests/ci/repository_gate_manifest_test.sh`
- `python3 tests/performance/runtime_benchmark_stats_test.py`
- `bash tests/performance/regression_gate_test.sh`
- `bash tests/performance/p0_regression_gate_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

## Acceptance boundary

This Work Item establishes a credible measurement instrument and evidence
contract. It does not claim a latency, CPU, I/O, or memory improvement and it
does not modify production governance execution.
