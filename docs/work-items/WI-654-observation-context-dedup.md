---
author: AI Cockpit maintainers
title: WI-654 — Observation context deduplication (investigation)
description: A measured, real duplicate-read finding for preflight, and why the obvious fix is not safe as first conceived.
workItemId: WI-654-observation-context-dedup
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-654-observation-context-dedup
terminalArchive: .ai/work-items/archive/WI-654-observation-context-dedup.contract.json
terminalVerification: .ai/evidence/WI-654-observation-context-dedup.verification.json
terminalFinalization: .ai/decisions/WI-654-observation-context-dedup.finalize.json
terminalDecision: .ai/decisions/WI-654-observation-context-dedup.close.json
---

# WI-654 — Observation context deduplication (investigation)

This Work Item is P1-B of the architecture initiative, narrowly scoped to the
most concrete duplicate-read example the P0 map
(`docs/reference/architecture-responsibility-map-2026-09.md`) identified. It
ships no production code change: the fix that looked obviously safe turned
out not to be, and this document is that finding.

## Measurement

A temporary, never-committed instrumentation (atomic counters wrapping
`repository_id` and `snapshot_digest`, reverted before any commit — same
method as WI-648/649/650) was added and run against a freshly attached
fixture repository with one active Work Item:

| command | `repository_id` calls | `snapshot_digest` calls |
|---|---|---|
| `preflight` | 3 | 2 |
| `checkpoint` | 3 | 2 |
| `status` (with an active Work Item) | 1 | 0 |

This confirms real, request-scoped duplicate computation for
`preflight`/`checkpoint` specifically, not merely a theoretical concern.

## The hypothesis, and why it is false

One of the three `repository_id` calls is inside
`project_governance_unknowns` (`crates/cockpit-repository/src/
project_governance.rs:352`), which calls `repository_id(&root)` to bind an
expected identity for validating `.ai/project/capabilities.json`. The
hypothesis was: `contract_freshness_findings`
(`crates/cockpit-repository/src/lib.rs:9330`) runs earlier in the same
governance decision
(`governance_decision_for_contract_base_internal_with_archive`, which calls
`contract_freshness_findings` at line 9434 and `project_governance_unknowns`
at line 9461) and already checks `contract.repository_id !=
repository_id(&root)`, so by the time `project_governance_unknowns` runs,
`contract.repository_id` should already equal the fresh value, and reusing it
would eliminate a redundant disk read of `.ai/cockpit.toml` with no behavior
change.

Verifying this against the actual source disproves it:

```rust
// crates/cockpit-repository/src/lib.rs:9330
if contract.repository_id != repository_id(&root).to_string() {
    findings.push("stale_contract".into());
}
```

`contract_freshness_findings` **records** a `stale_contract` finding on
mismatch; it does not `return` early and does not stop
`governance_decision_for_contract_base_internal_with_archive` from continuing
to call `project_governance_unknowns` afterward. A foreign or stale Contract
(one whose `repository_id` field no longer matches the actual repository) can
therefore still reach `project_governance_unknowns` with an unverified,
possibly-wrong `contract.repository_id`. Substituting it for
`repository_id(&root)` would change `load_declaration`'s `expected_repository_id`
comparison in exactly that edge case, potentially producing a different
`project_capabilities_repository_mismatch`-class unknown than the current
code does — a real behavior difference, not a pure refactor, even though it
would be invisible in the common (fresh, matching) case that most tests
exercise.

A trial edit implementing the substitution was written, then reverted after
this verification, per this initiative's own rule not to ship a change whose
compatibility risk was not actually checked.

## What a safe fix would require

Eliminating this call safely requires threading one freshly-resolved
`repository_id` value — computed once — through three functions:
`contract_freshness_findings`, `governance_decision_for_contract_base_
internal_with_archive`, and `project_governance_unknowns`. `contract_freshness_findings`
is `pub fn` with two call sites in `lib.rs` (8900, 9434); the governance
decision function family has, per the P0 map, 2-4 near-duplicate
`_internal`/`_with_archive`/`_with_runtime` variants each, used well beyond
the preflight path this Work Item measured. Changing that signature family
correctly, verifying every call site keeps identical behavior, and proving it
via tests is a substantially wider change than this Work Item's measured
justification (eliminating one small local TOML read) supports. It is left
for a future Work Item, gated on either a stronger measured need or being
bundled with other required changes to the same function family.

The second measured duplication (`snapshot_digest` called twice on the
identical, already-in-memory `RepositorySnapshot` value, once inside
`project_governance_unknowns` and once inside `apply_preflight_review_evidence`)
is a cleaner candidate in principle — both calls provably operate on the same
input — but threading it through the same function family carries the same
signature-change scope described above, and the cost being eliminated (an
in-process digest computation that only spawns a `git` subprocess when the
working tree is dirty) has not been measured to be significant. It is
declined here for the same reason.

## P0 map follow-up (deferred)

`docs/reference/architecture-responsibility-map-2026-09.md` (WI-652) named
this pattern as a duplicate-read example but did not evaluate whether any
specific fix is safe. That document is not amended in this Work Item because
it does not exist on this branch (WI-652 has not yet merged); once WI-652
lands, it should be extended with this Work Item's more specific finding
(the obvious fix is unsafe, and why) so a future reader does not re-attempt
the same disproven substitution.

## Verification

No production code changed in the final state of this Work Item. `cargo fmt`,
`cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test
--locked --workspace` pass against the unchanged workspace.
