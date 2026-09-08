---
author: AI Cockpit maintainers
title: Performance initiative synthesis (2026-09)
description: What was measured and fixed, what was measured and deliberately not pursued, and what remains unmeasured.
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-651-performance-initiative-synthesis
---

# Performance initiative synthesis (2026-09)

This document closes out the 2026-09 AI Cockpit performance initiative. Its
purpose is to record, in one place, what was measured and changed, what was
measured and deliberately *not* changed (with the evidence for that
decision), and what remains unmeasured — so a future contributor starts from
verified facts instead of re-deriving them or guessing at priority.

North Star: Calibrated Human-Agent Trust. Every change below preserves
governance decisions, repository isolation, evidence validity, and recovery
behavior; none weakens a required check or invents a performance claim
without a captured before/after number.

## Delivered Work Items

### WI-647 — Benchmark cold/warm grouping correctness (P0)

`tests/performance/runtime_benchmark.sh` sorted all timing samples before
selecting the minimum as "cold," silently swapping the first process call
for whichever later call happened to be fastest. Fixed to group on original
call order first (`tests/performance/runtime_benchmark_stats.py`), pinned by
a fixed-sequence test (`[120, 20, 22, 21]` → cold must be `120`). Also
withholds unreliable `p50`/`p95` claims below fixed sample-count floors,
records measurement environment and pre-measurement identity-probe
disclosure, and fixes `regression_gate.sh`'s `int`-only type check (which
had rejected every real float `elapsedMs` measurement this harness ever
produced). No Rust source changed. See
`docs/work-items/WI-647-benchmark-cold-warm-grouping.md`.

### WI-648 — status bottleneck diagnosis (P0)

Using the corrected harness, `status` measured ~1.8s cold/warm on this
repository vs. <110ms for `inspect`/`doctor`/`observe`. Root-caused via
temporary, never-committed profiling to `historical_finalization_inventory`
(`crates/cockpit-repository/src/lib.rs:3832`) calling
`resolve_resource_finalization_head` once per legacy `.ai/decisions/
*.finalize.json` receipt, and that function re-scanning the entire
`.ai/decisions` directory every time — O(decision-entries × legacy
receipts), not O(entries). Diagnosis only; no code changed. See
`docs/work-items/WI-648-status-bottleneck-diagnosis.md`.

### WI-649 — status history-scan fix (P1)

Fixed the bottleneck WI-648 found: `historical_finalization_inventory` now
reads `.ai/decisions` once and passes each work item's pre-grouped
transition candidates into a new `resolve_resource_finalization_head_
with_candidates`, instead of re-scanning per receipt. The original
`resolve_resource_finalization_head` (used by `finalize`/`finalize-verify`/
`finalize-recovery-plan`) is unchanged. Verified byte-for-byte identical
`status` JSON output (383 legacy receipts, only `runtimeDigest` differs
because the binary changed) plus a new regression test proving no
cross-contamination between independent work items' candidate sets.
Measured: `status` improved ~20-23% cold and warm across two independent
measurement pairs; the remaining ~1.24s of `historical_finalization_
inventory` cost (down from ~1.7s) is per-receipt work
(`closed_finalization_projection_kind`, `archived_contract_digest`,
transition-file reads) not addressed by this Work Item. See
`docs/work-items/WI-649-status-history-scan-fix.md`.

### WI-650 — blocking wait for verification child processes (P1)

`execute_captured` waited for every child process with a `child.try_wait()`
+ `sleep(10ms)` loop. An isolated micro-benchmark (outside the CLI, to
remove unrelated noise) measured ~11ms of pure added waiting for a
near-instant command (12.19ms vs 1.19ms mean) and no measurable difference
for a multi-second command. Fixed on Unix only: the child moves into a
dedicated thread that blocks in `child.wait()` and reports over a channel;
the caller does one bounded `recv_timeout`. Windows is textually unchanged
and explicitly not verified (no Windows environment available). Measured
end-to-end via `ai-cockpit verify --command true`: ~104-108ms baseline vs.
~94-98ms candidate. See `docs/work-items/WI-650-verification-wait-blocking.md`.

## Candidates evaluated and not pursued, with evidence

### P1 — streaming hash for large files / bounded parallel read

Not pursued. `ai-cockpit inspect --repo` on this repository reports
`filesRead: 2, filesHashed: 2` regardless of the repository's 9614 tracked
files — WI-395's prior optimization already makes this request-scoped and
size-independent rather than a full-tree walk. The largest tracked file in
this repository is ~2.1 MB (`tests/conformance/reference_file_inventory.json`);
several adopter-acceptance manifest files are ~1.3-1.6 MB. Whole-file reads
at this size are sub-millisecond on local SSD/APFS. There is no measured
evidence in this repository that either the file count or file size drives
any of the latency found in P0/P1 work — the entire measured bottleneck
(`status`'s ~1.8s) was in directory-entry re-scanning and per-receipt
resolution logic, not file I/O volume. Implementing streaming hashing or
bounded parallel read now would add concurrency-control complexity
(bounded file handles, deterministic output ordering) with no measured
benefit. This should be revisited only if a future repository or scenario
(e.g. the still-unbuilt "large file modified" P0 scenario) produces a
measurement showing otherwise.

### P2 — request/verification deduplication (`PhysicalSingleFlightCoordinator`)

Not wired in. Direct source inspection
(`crates/cockpit-verification/src/lib.rs:1473`) confirms
`PhysicalSingleFlightCoordinator` has no caller outside its own test file
(`crates/cockpit-verification/tests/physical_execution.rs`) in
`cockpit-cli`, `cockpit-mcp`, `cockpit-agent`, or `cockpit-core` — it is
implemented and tested but not connected to any command path. Wiring it in
would require, per the initiative's own acceptance bar, exact execution
identity matching, per-Work-Item authorization and evidence-binding
verification, and explicit fail-closed behavior for checks that must stay
fresh — a nontrivial change with real repository-isolation and
authorization risk. No measurement in this initiative captured a concrete
concurrent-duplication cost: a 3-way concurrent `status` run (a read-only,
unrelated command) showed wall time consistent with I/O contention, not a
verification-execution dedup opportunity, and no scenario exercising
concurrent *verification* of the same Work Item/command was measured. Absent
that measurement, connecting the coordinator now would be a speculative
architecture change, not a measured optimization. The concrete prerequisite
for revisiting this is the still-unbuilt P0 scenario "multiple concurrent
verification requests" (same repository, Work Item, and command) with a
captured duplicate-execution count.

### P2 — `IncrementalMerkle` trustworthy caching

Not pursued, and not currently a risk. Direct source inspection
(`crates/cockpit-git/src/lib.rs:18`) confirms `IncrementalMerkle` has no
caller outside its own test file (`crates/cockpit-git/tests/snapshot.rs`) —
it is implemented and tested but not wired into `GitRepository::snapshot()`
or any other production path. Because nothing in production currently
trusts a cached digest from this type, none of the cache-validity risks the
initiative asked to check (same-length edits with restored mtime, file
replace/delete/rename/type change, repository-root changes, concurrent
modification during read, watcher event loss) apply today — there is no
live protected decision depending on its output. If a future Work Item wires
it in, it must first establish those validity guarantees (or explicitly
re-read/return-unknown when it cannot prove them) before any performance
claim; this document intentionally does not pre-authorize that connection.

### P3 — resident MCP repository-bound cache, in-process Git, PGO

Not pursued. Each P3 candidate is gated on "only if prior measurement proves
necessary." Nothing measured in P0-P2 pointed at a bottleneck any of these
three would address: the confirmed bottleneck (`historical_finalization_
inventory`'s directory re-scan) was fixed as a pure algorithmic change
(WI-649) with no need for a resident cache, a different Git implementation,
or profile-guided compilation. Pursuing any of these now would be
optimizing without a target, which the initiative's own ground rules
(measure first, no assumed multipliers) explicitly rule out.

## Known gaps (not silently dropped)

The following P0 scope items were not completed in this initiative and
remain open for a future measurement-first Work Item:

- The 8-scenario fixture matrix (small/large clean repo, single/multi/large
  file change, many historical Work Items, concurrent verification, resident
  MCP repeat queries) — only the "many historical Work Items" scenario was
  effectively exercised, via this repository's own real history.
- Phase-level resource-metric exposure (bytes read/hashed, git call count,
  spawned process count, cache-invalidation reasons, peak memory) on
  `status`/`doctor`/`observe`/`diagnose`/`work-item status` — today only
  `inspect` exposes `filesRead`/`filesHashed`/`gitCalls`.
- Persistent MCP session latency measurement — the corrected harness
  (WI-647) explicitly measures independent CLI process latency only.
- A dev-only comparator that tolerates explicitly bound baseline/candidate
  Runtime identity differences while still verifying each side's evidence
  integrity — `regression_gate.sh` still requires identical
  `runtimeVersion`/`runtimeDigest` between baseline and candidate, correct
  for release acceptance but an open gap for comparing builds with differing
  dev-identity bindings.

Each of these remains a legitimate, independently scopable Work Item; none
is implemented here because none has been measured yet, consistent with
this initiative's rule to measure before modifying.
