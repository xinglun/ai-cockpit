---
author: AI Cockpit maintainers
title: WI-649 — Fix the status O(n^2) history-scan bottleneck
description: Remove the redundant per-item .ai/decisions re-scan that WI-648 identified, with measured before/after evidence.
workItemId: WI-649-status-history-scan-fix
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-649-status-history-scan-fix
terminalArchive: .ai/work-items/archive/WI-649-status-history-scan-fix.contract.json
terminalVerification: .ai/evidence/WI-649-status-history-scan-fix.verification.json
terminalFinalization: .ai/decisions/WI-649-status-history-scan-fix.finalize.json
terminalDecision: .ai/decisions/WI-649-status-history-scan-fix.close.json
---

# WI-649 — Fix the status O(n^2) history-scan bottleneck

This Work Item is the P1 fix for the bottleneck WI-648 root-caused: `status`
was ~20-40x slower than `inspect`/`doctor`/`observe` because
`historical_finalization_inventory` re-scanned the entire `.ai/decisions`
directory once per legacy receipt via `resolve_resource_finalization_head`.

## Change

`crates/cockpit-repository/src/lib.rs`:

- `historical_finalization_inventory` now reads `.ai/decisions` exactly once,
  materializes the listing, and builds an in-memory `work_item_id ->
  transition candidates` index from that single listing (grouping by the
  first `.finalize.` boundary in each filename; work item IDs cannot contain
  `.`, so this is an exact, unambiguous split).
- `resolve_resource_finalization_head` is split into a thin wrapper (unchanged
  signature, behavior, and directory scan) and a new
  `resolve_resource_finalization_head_with_candidates`, which does the same
  chain-walking but consumes a caller-supplied candidate list instead of
  scanning the directory itself.
- `historical_finalization_inventory` calls the `_with_candidates` variant
  with candidates it already grouped, instead of calling
  `resolve_resource_finalization_head` (which would re-scan). The three other
  call sites (`finalize`, `finalize-verify`, `finalize-recovery-plan`) keep
  calling the unchanged wrapper — this Work Item does not touch their
  behavior, cost, or signature.

No output field, schema, or governance decision changes. This is a pure
internal reuse-of-one-directory-listing refactor.

## Correctness evidence

- `cargo test --locked --workspace` passes unchanged (121 test result blocks,
  0 failures).
- New test `crates/cockpit-repository/tests/historical_finalization_scan.rs`
  builds two independent legacy Work Items in one repository — ALPHA with a
  two-step transition chain (sequence 2) and BETA with none (sequence 0) —
  and asserts `status_with_runtime`'s `historical_finalization` entries match
  the ground truth independently computed by the *unchanged*
  `resolve_resource_finalization_head` (via `verify_resource_finalization`)
  for both, proving the new index does not cross-contaminate candidates
  between work items.
- Direct byte-for-byte comparison: `ai-cockpit status --repo` on this actual
  repository (580+ archived Work Items, 383 legacy `.finalize.json` receipts),
  comparing the installed v0.2.87 binary's JSON output against this Work
  Item's release build — every field is identical except `runtimeDigest`
  (which necessarily changes because the binary bytes changed). This includes
  all 383 `historicalFinalization` entries: same `state`, `sequence`,
  `predecessorDigest`, `historicalKind`, and `safeActions` for every one.

## Measured performance (advisory)

On 2026-09-08, macOS arm64 (same repository as WI-648's diagnosis), using the
corrected cold/warm grouping harness from WI-647 (`tests/performance/
runtime_benchmark.sh`, run externally against this repository; this Work Item
does not itself modify that script), 12 iterations, baseline vs. this Work
Item's release build, repeated across two independent measurement pairs:

| command | run 1 delta | run 2 delta |
|---|---|---|
| status.cold | -20.3% | -22.9% |
| status.warm | -21.2% | -22.0% |

`status` improves by a consistent ~20-23% (from ~1.86-1.96s to ~1.48-1.51s).
A finer per-step profile (same temporary, never-committed instrumentation
method as WI-648) shows `historical_finalization_inventory` itself dropping
from ~1.7s (WI-648's baseline) to ~1.24-1.37s — the eliminated redundant
directory listing accounts for the difference; the remaining ~1.24s is spent
in per-receipt work (`closed_finalization_projection_kind`,
`archived_contract_digest`, and the transition file reads themselves) that
this Work Item does not change and that a future Work Item would need to
separately diagnose before optimizing further.

`inspect`/`doctor`/`observe` deltas were noisy and sign-flipping across
repeated runs (e.g. `doctor.warm` measured +9.2% in one pair, and separately
+0.9%/-14.1% for `inspect.cold` across two comparisons of the *same*
unmodified binary against itself at different filesystem locations) — these
commands never call `historical_finalization_inventory` or
`resolve_resource_finalization_head` (only `status_with_runtime` does, per
`crates/cockpit-cli/src/main.rs:686`), so there is no code path through which
this change could affect them, and the corrected harness's own
`p95Unreliable`/`p50Unreliable` reliability guard (WI-647) applies to their
small (<120ms) absolute magnitudes. These are local process-latency
observations, not provider or enterprise guarantees.

## Out of scope

Caching `historical_finalization_inventory` results across separate `status`
invocations (WI-648's second candidate), and diagnosing the remaining ~1.24s
per-receipt cost, are left for a future Work Item.
