# WI-696 P0 scenario measurement plan

## Contract

- Work Item: `WI-696-p0-scenario-measurement`
- Base: `origin/main` at `513e7e72523097893c658ec76c25706fd5b266b8`
- Scope: existing P0 benchmark evidence under `.ai/evidence/external/`, this plan, the three-language Work Item record, and parity rows.
- Out of scope: `crates/**`, `src/**`, `tests/**`, Runtime implementation, governance semantics, authorization rules, and production optimization.
- Authority: `human:repository-owner`

## Hypothesis

The P0 harness is now order-preserving and fail-closed, but the repository has
not yet established which real scenario and public command dominate the path
to a trusted governance decision. Running the matrix with sufficient warm
samples will identify a target without treating unavailable CPU/I/O/memory
metrics as zero or conflating independent CLI processes with resident MCP.

## Execution

1. Bind the latest remote default base and declared verification commands in the Contract.
2. Use attached temporary repositories for small and change shapes; use the clean latest main checkout for many-file and many-history shapes.
3. Build one external development release binary from the Work Item base and record both reported and file digests.
4. Run six portable scenarios with 20 warm samples, preserving raw evidence and a scenario-specific guardrail budget.
5. Run the existing independent-CLI concurrency harness with 20 rounds and concurrency four; retain its cancellation and failure evidence.
6. Inventory the current `PhysicalSingleFlightCoordinator` call graph at the latest source revision.
7. Derive a bottleneck order and explicit successor prerequisites; do not implement a candidate optimization in this Work Item.

## Acceptance boundary

The final record must preserve first/warm boundaries, raw order, quantile
reliability, Runtime and repository identities, scenario facts, unavailable
metrics, and budget inputs. A later optimization Work Item must use paired
baseline/candidate records and the P0 regression gate. Resident MCP and
in-process single-flight remain unmeasured/declined until their real request
path and evidence boundaries are available.

## Verification

- `python3 tests/performance/runtime_benchmark_stats_test.py`
- `bash tests/performance/p0_regression_gate_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh .`
- `bash tests/docs/work_item_status_consistency_test.sh`
- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `git diff --check`
