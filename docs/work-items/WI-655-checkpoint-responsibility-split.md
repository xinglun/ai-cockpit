---
author: AI Cockpit maintainers
title: WI-655 — Split checkpoint_work_item into Observation/Governance/Persistence
description: The first proven P2-A use case, a same-file, same-signature reorganization with zero behavior change.
workItemId: WI-655-checkpoint-responsibility-split
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-655-checkpoint-responsibility-split
terminalArchive: .ai/work-items/archive/WI-655-checkpoint-responsibility-split.contract.json
terminalVerification: .ai/evidence/WI-655-checkpoint-responsibility-split.verification.json
terminalFinalization: .ai/decisions/WI-655-checkpoint-responsibility-split.finalize.json
terminalDecision: .ai/decisions/WI-655-checkpoint-responsibility-split.close.json
---

# WI-655 — Split checkpoint_work_item into Observation/Governance/Persistence

This Work Item is P2-A of the architecture initiative: the first complete,
proven use case for extracting Observation/Governance/Evidence+Projection
responsibilities, per the P0 map's recommendation to start with the smallest
lifecycle function before touching `finish`/`archive`/`close`.

## Why this function, and why same-file

`checkpoint_work_item` (`crates/cockpit-repository/src/lib.rs`) mixed
observation (read `summary.json`/`contract.json`, capture the Git snapshot),
governance decision (state/duplicate/freshness/verification-ordering checks,
plus a call into the shared preflight governance decision), and persistence
(the `atomic_json` write and `LifecycleReceipt` construction) inline in one
~120-line function. It already has test coverage across 17 files in
`crates/cockpit-repository/tests/`, giving a real regression net.

WI-654 showed that a seemingly-obvious change to adjacent governance code
can silently alter behavior in an edge case that existing tests do not
happen to cover. Given that risk, this Work Item deliberately reorganizes
`checkpoint_work_item`'s body into named internal helpers **within the same
file, with no change to the public signature, error messages, JSON fields
written, or write order** — not a cross-module or cross-crate extraction.
That is a conservative, low-risk way to prove the responsibility boundary
before considering a larger, riskier move.

## A subtlety that shaped the split

The original code does not read the Contract or capture the Git snapshot
until *after* three cheap checks that only need already-read `summary.json`
fields (duplicate-checkpoint, lifecycle state, preflight-state presence).
That ordering is fail-fast on purpose: if a checkpoint is invalid for a cheap
reason, the function must not first pay for a Git snapshot only to still
reject it, and — more importantly — if multiple things are wrong at once,
the *first* violated check's error is what must surface, not whichever error
a reordered version happens to hit first.

The split therefore preserves this exact ordering: the three cheap checks
stay inline in `checkpoint_work_item`, before the new `checkpoint_observe`
call; only the checks that already required the Contract and snapshot in the
original code (snapshot-freshness, contract-freshness, preflight governance,
verification-ordering) move into the new `checkpoint_governance_checks`,
called after observation, in their original relative order.

## Change

- `CheckpointObservation` (private struct): `contract_path`, `contract`,
  `snapshot`, `current_snapshot_digest`, `current_contract_digest`.
- `checkpoint_observe(root, work_item_id) -> Result<CheckpointObservation,
  ObserverError>`: exactly the read-contract + discover-snapshot +
  compute-digests code that used to sit inline, unchanged in content or
  error paths.
- `checkpoint_governance_checks(root, summary_path, summary, preflight_state,
  observation) -> Result<(), ObserverError>`: the four checks that ran after
  observation in the original code, in the same order, with identical error
  messages. This helper is **not** claimed to be I/O-free:
  `require_green_or_yellow_preflight_governance` still performs its own
  observation internally, exactly as before.
- `checkpoint_work_item` keeps the three cheap pre-checks, then calls
  `checkpoint_observe`, then `checkpoint_governance_checks`, then the
  unchanged persistence code (`append_checkpoint_evidence`, the summary
  field writes, `atomic_json`, `LifecycleReceipt` construction).

## Correctness evidence

All 17 existing test files that call `checkpoint_work_item`
(`agent_risk_checkpoint.rs`, `archive_integrity.rs`, `contract_preflight.rs`,
`intelligence.rs`, `evidence_assurance.rs`, `knowledge_projection.rs`,
`knowledge_cache.rs`, `lifecycle_order.rs`, `outcome_report.rs`,
`preflight_review.rs`, `recovery_events.rs`, `recovery_revalidation.rs`,
`task_outcome_events.rs`, `recovery_decision.rs`, `status_projection.rs`,
`verification_route.rs`, `resource_finalization_transition.rs`) pass
unchanged — no test was added, removed, or had an assertion altered. `cargo
test --locked --workspace` passes (120 test result blocks, 0 failures).
`cargo fmt --all -- --check` and `cargo clippy --locked --workspace
--all-targets --all-features -- -D warnings` pass.

## Out of scope / follow-up

`finish_work_item`, `archive_work_item`, and
`close_work_item_with_structured_decision` are larger, higher-risk functions
with the same read+decide+persist mixing; they are not attempted here.
`docs/reference/architecture-responsibility-map-2026-09.md` (WI-652, not yet
merged, does not exist on this branch) should be updated once merged to
record this as the first proven P2-A use case.
