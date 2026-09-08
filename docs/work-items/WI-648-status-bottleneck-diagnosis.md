---
author: AI Cockpit maintainers
title: WI-648 — status command bottleneck diagnosis
description: Root-cause the dominant cost of the status command before any optimization is attempted.
workItemId: WI-648-status-bottleneck-diagnosis
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-648-status-bottleneck-diagnosis
terminalArchive: .ai/work-items/archive/WI-648-status-bottleneck-diagnosis.contract.json
terminalVerification: .ai/evidence/WI-648-status-bottleneck-diagnosis.verification.json
terminalFinalization: .ai/decisions/WI-648-status-bottleneck-diagnosis.finalize.json
terminalDecision: .ai/decisions/WI-648-status-bottleneck-diagnosis.close.json
---

# WI-648 — status command bottleneck diagnosis

This Work Item is a P0 measurement/diagnosis step of the AI Cockpit
performance initiative. It changes no production code and no governance
behavior; it root-causes, with reproducible evidence, why `ai-cockpit status`
is dramatically slower than `inspect`/`doctor`/`observe` on this repository, so
the next optimization Work Item has a verified target instead of a guess.

## Starting evidence

WI-647's corrected benchmark harness measured, against this repository (macOS
arm64, installed v0.2.87 binary, 10 iterations):

| command | cold (ms) | warm p50 (ms) |
|---|---|---|
| inspect | 58.3 | 58.3 |
| status | 1788.5 | 1848.4 |
| doctor | 48.0 | 40.7 |
| observe | 109.4 | 105.9 |

`status` is ~20–40x slower than every other measured command, cold and warm
alike. `inspect`/`doctor`/`observe` do not exhibit this cost, so the cause is
specific to `status`'s own code path, not a shared cost (process startup,
identity resolution, or Git snapshot capture) paid by every command.

## Method

`crates/cockpit-cli/src/main.rs:686` shows the `status` command is the only
caller of `cockpit_repository::status_with_runtime(&repo, Some(&runtime_context))`;
every other measured command takes a different path. `status_with_runtime`
calls `repository_readiness_from_snapshot_with_runtime`
(`crates/cockpit-repository/src/lib.rs:3452`), which runs five steps in
sequence. To attribute cost per step, a temporary local patch (never
committed) wrapped each step with `std::time::Instant`/`eprintln!`, e.g.:

```rust
let __t4 = std::time::Instant::now();
let historical_finalization = historical_finalization_inventory(root, runtime)?;
eprintln!("__PROFILE__ historical_finalization_inventory {:?} count={}", __t4.elapsed(), historical_finalization.len());
```

built with `cargo build --release -p cockpit-cli`, run directly against this
repository and against freshly `attach`-ed scratch fixtures, then reverted
with `git checkout -- crates/cockpit-repository/src/lib.rs` before committing
anything. This is fully reproducible from the cited line numbers.

## Finding: an O(n²) directory re-scan, not O(n) history growth

Per-step timings against this repository (warm, after first-call OS cache
fill; `.ai/decisions` has 1573 entries, 383 of them `*.finalize.json`;
`.ai/work-items/archive` holds 580+ archived Work Items):

| step | elapsed | notes |
|---|---|---|
| `discover_default_base` | ~9 ms | one Git call |
| `non_governance_changed_paths` | <1 µs | in-memory over the existing snapshot |
| `unclosed_archived_work_items_with_id` | ~90 ms | one `.ai/work-items/archive` scan; O(1) file lookups per item |
| `classify_historical_debt` | ~0 µs | ran over 0 unclosed items in this repository |
| `historical_finalization_inventory` | **~1.7 s** | dominant cost |
| `count_suffix` + `orphaned_active_artifact_names` | <10 µs | small `active/` directory |

Against a freshly `attach`-ed empty repository (0 entries under
`.ai/decisions`), the same two steps take ~0.05–0.2 ms combined — the cost is
not paid by every repository, only one with accumulated history.

`historical_finalization_inventory` (`crates/cockpit-repository/src/lib.rs:3832`)
iterates every `*.finalize.json` file in `.ai/decisions`. For each one whose
recorded `runtimeVersion`/`runtimeDigest` does not equal the currently
running Runtime's (line 3923) — true for essentially every receipt written by
a past Runtime version, i.e. the common case for any repository with real
history, not an edge case — it calls
`resolve_resource_finalization_head` (`crates/cockpit-repository/src/lib.rs:11634`).
That function itself runs `fs::read_dir(root.join(".ai/decisions"))`
(line 11654) and filters for transition files belonging to that one
`work_item_id`. Because this inner scan re-reads the same `.ai/decisions`
directory listing from scratch for every outer iteration, the total cost is
**O(decision-directory entries × matching legacy receipts)**, not O(entries):
with 1573 directory entries and 383 legacy receipts this is on the order of
600,000 directory-entry inspections, plus a `read_resource_finalization_transition`
+ `serde_json::to_value` + `digest_json` cost for every filename that matches
the `{work_item_id}.finalize.` prefix (156 transition files exist in this
repository). A partial reproduction with 100 copied real `.finalize.json`
files but without their corresponding `.ai/work-items/archive/*` companions
measured only ~20–166 ms, confirming the cost requires the full archive
context to reach the expensive branch, not file count alone — this rules out
"file count alone" as a sufficient explanation and narrows the cause to the
resolution chain described above.

`unclosed_archived_work_items_with_id` (`crates/cockpit-repository/src/lib.rs:3660`)
does **not** show this pattern: its per-item check
(`close_decision_is_valid_for_status`, line 14781) is a direct, O(1) path
lookup by known filename, not a directory scan, which is why it costs ~90 ms
(one linear directory listing plus 580+ O(1) lookups) rather than seconds.

## Consequence

This entire ~1.7 s computation is unconditional and uncached: it re-executes
in full on every single `status` invocation, even though its result for any
receipt whose `runtimeVersion`/`runtimeDigest` differs from the current
Runtime is fully deterministic — repeated `status` calls against an unchanged
repository recompute an identical answer every time. Because the cost is
quadratic in the number of `.ai/decisions` entries, it will keep worsening
faster than the repository's historical Work Item count grows, on every
adopter repository with real development history, not only this one.

## Recommendation for the next (P1) Work Item

Two independent, narrow, semantics-preserving candidates, to be measured
separately per WI-402/WI-647 discipline (no stacked claims):

1. Read `.ai/decisions` once per `historical_finalization_inventory` call,
   group entries by `work_item_id` prefix in memory, and pass that index into
   `resolve_resource_finalization_head` instead of re-scanning the directory
   per outer-loop item. This removes the O(n²) term without changing any
   output value, since the candidate set for a given `work_item_id` is
   identical either way.
2. Only after (1) is measured: consider caching the resolved
   `HistoricalFinalizationInventoryItem` for a receipt whose content digest
   and the current Runtime identity are unchanged since the last `status`
   call in the same repository, subject to the same fail-closed/identity
   rules WI-402 established for verification reuse.

This Work Item does not implement either candidate; it hands the next P1
Work Item a verified, line-cited target instead of an assumption.

## Verification

No Rust source, test, or governance file changed. `cargo fmt`, `cargo
clippy`, and `cargo test --workspace` are unaffected (unchanged from the
already-passing baseline). The measurement method above is reproducible by
any reviewer from the cited line numbers and commands.
