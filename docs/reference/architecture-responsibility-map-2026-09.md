---
author: AI Cockpit maintainers
title: Architecture responsibility and dependency map (2026-09)
description: Current ownership of observation, governance, lifecycle, evidence, execution, and projection facts, and the factual basis for the P1-P3 architecture Work Items.
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-652-architecture-responsibility-map
---

# Architecture responsibility and dependency map (2026-09)

This is the P0 deliverable of the 2026-09 architecture-optimization
initiative. It documents current responsibility boundaries — not a target
design — so every subsequent Work Item (P1-B observation context, P2-A
lifecycle/storage extraction, P2-B state types, P2-C multi-file consistency,
P3 physical-execution boundary) shares one cited factual basis instead of
re-deriving it. This Work Item changes no code.

North Star: Calibrated Human-Agent Trust. Documenting where responsibilities
are mixed today is not a claim that current behavior is wrong; it is the
prerequisite for reducing edit coupling without changing governance truth.

## Scale of the monolith

`crates/cockpit-repository/src/lib.rs` is 18220 lines — by far the dominant
file in the workspace. The same crate also has `governance_controls.rs`
(1753 lines), `outcome_render.rs` (639 lines, purified by WI-653, not
revisited here), and `project_governance.rs` (402 lines).

## Functional clusters in lib.rs and their current I/O/decision mixing

| Cluster | Approx. lines | Mixes I/O with decision in one function? |
|---|---|---|
| Repository identity (`repository_id`, `new_repository_id`) | 728-763 | No — pure derive/read |
| Verification reuse assessment (`assess_verification_reuse*`, executable identity hashing) | 763-1658 | Yes — e.g. `assess_verification_reuse_measured` (777) both gathers identity facts and returns the reuse verdict |
| Receipt store / capability-scoped nofollow file I/O (`load_reusable_receipt`, `persist_reusable_receipt`, `open_cap_directory_nofollow`, `atomic_replace_cap_strict`) | 1660-2832 | No — a self-contained storage primitive layer, already separated from governance logic |
| Attach / migration (`attach`, `migration_plan*`, `apply_migration`, `compatibility_report`) | 2832-3337 | Yes — file I/O and protocol-version decisions combined per function |
| Status / readiness (`status_with_runtime` 3343, `repository_readiness*` 3426-3542, `historical_finalization_inventory` 3832) | 3337-4209 | Yes — each function reads files/git and computes the readiness verdict inline (this cluster contained the WI-648/649 bottleneck) |
| Lifecycle start/checkpoint/preflight (`start_work_item*`, `checkpoint_work_item`, `preflight_work_item_internal` 5107) | 4209-5293 | Yes — preflight reads Contract+Summary+snapshot, calls governance-decision helpers, and writes the preflight decision in one function |
| Finish (`finish_work_item_internal` 5320) | 5293-5670 | Yes — same read+decide+write mixing |
| Recovery / revalidation | 5670-6846, 16162-16451 | Yes — predecessor/successor binding validation mixed with recovery-decision recording |
| Evidence retention / audit export | 7470-8102 | Partially — policy-driven read+write, reasonably self-contained |
| Delegated (external) evidence import/list | 8102-8328 | No — file-bound, self-contained |
| Snapshot digesting / policy resolution (`source_tree_digest`, `snapshot_digest` 8548, `policy_document`, `effective_policy_for_contract`, `resolve_verification_route`, `evaluate_contract_quality_gate`) | 8328-9016 | No at this layer — read-only over an already-captured snapshot; the actual governance-decision core |
| Governance decision entry points (`governance_decision_for_contract*`, `require_green_governance*`, `evidence_state_for_contract*`) | 9349-10389 | Branchy but centralized — 2-4 near-duplicate `_internal`/`_with_archive`/`_with_runtime` variants each (e.g. 9370, 9385, 9410, 9427) |
| Archive (`archive_work_item_internal` 10554) | 10389-11022 | Yes — same read+decide+write mixing as finish |
| Resource finalization (plan/record/verify/resolve-head) | 11022-13152 | Yes — mixed file I/O + validation + chain-walking (the WI-648/649 area; `resolve_resource_finalization_head_with_candidates` now separates candidate-gathering from chain-walking) |
| Close (`close_work_item_with_structured_decision_internal` 13242, ~300 lines) | 13152-13615 | Yes — reads Contract/Summary/archive, validates policy, validates resource finalization, writes the decision file, all inline |
| Knowledge / capability truth / performance diagnosis / work-item intelligence | 13906-16784 | No — mostly read-only projections |
| Parallel slot leasing | 17001-17322 | No — file-lease-based mutual exclusion, self-contained |
| Compatibility/scope relations (`required_verification_checks`, `scope_pattern_relation`, `work_item_compatibility`) | 17402-17720 | No — pure functions, no I/O, well isolated |
| Shared low-level helpers (`read_json`, `atomic_json`, `atomic_write`, `observe`/`observe_cached` 17848/17957, `collect_files`) | 17770-end | No — but `observe`/`observe_cached` are the generic snapshot-observation entry points that other lifecycle functions often re-derive independently instead of calling |

## Concrete duplication and mixed-responsibility examples

- **Duplicate `repository_id` derivation**: recomputed/re-read from
  `.ai/cockpit.toml` at nearly every call site across lib.rs,
  `governance_controls.rs`, and `project_governance.rs` (e.g.
  `project_governance.rs:306,352,398` each independently call
  `crate::repository_id` rather than receiving one resolved value through a
  request-scoped context).
- **Duplicate `snapshot_digest` computation**: `lib.rs:8548` defines it;
  `project_governance.rs:305,349,395` each recompute it independently when
  loading a declaration, even when the caller already holds a fresh snapshot
  from the same request.
- **Read+decide+persist mixed in one function**: `preflight_work_item_internal`
  (5107), `finish_work_item_internal` (5320), `archive_work_item_internal`
  (10554), and `close_work_item_with_structured_decision_internal` (13242)
  each read Contract/Summary/snapshot files, call multiple governance-decision
  helpers, and write the resulting decision/outcome/archive artifact — with
  no separation between "gather facts," "decide," and "persist" phases
  within the function body.
- **A "validator" that writes**: `governance_controls.rs:1190
  record_work_item_governance_controls` performs a write despite the module's
  own header comment describing it as not generating scenarios or
  final-dimension evidence — the module's stated read-only-validator boundary
  is not fully honored by every function in it.
- **Dependency direction**: no circular dependency was found.
  `governance_controls.rs` and `project_governance.rs` only call back into
  `lib.rs` for shared primitives (`ObserverError`, `repository_id`,
  `snapshot_digest`, `reject_duplicate_json_keys`); `lib.rs` calls forward
  into both. This is one-directional layering (lib.rs → the two modules), not
  circular — but it means the two modules cannot currently be extracted as
  independent crates without lib.rs staying a shared base, since they depend
  on its primitives.

## What is already well-separated and reusable

- **`RepositorySnapshot`** (`cockpit-git/src/lib.rs:212`) and
  `GitRepository::snapshot()` (268) are a clean, single fact-capture point
  already threaded as a parameter into lower-level functions
  (`repository_readiness_from_snapshot`, `project_governance_projection`,
  `source_tree_digest`, `snapshot_digest`). This is the right abstraction for
  P1-B to build on; it is just not consistently threaded end-to-end from
  every entry point to every leaf function today.
- **`RuntimeContext`** (`cockpit-protocol/src/lib.rs:120`) is a clean,
  minimal identity struct, consistently passed as `&RuntimeContext` through
  every `_with_runtime` variant.
- **Pure validators**: `governance_controls.rs`'s `required_verification_checks`,
  `validate_checkpoint_evidence_bindings`, and lib.rs's
  `scope_pattern_relation`/`work_item_compatibility` (17450-17720) already
  take typed facts as input and produce a decision with no I/O or side
  effects — the template P2-B state-type work should follow.
- **Capability-scoped nofollow file layer** (2313-2832) is a reusable,
  self-contained storage primitive already isolated from governance logic —
  P2-A's evidence/storage layer should reuse this, not rebuild it.
- **`OutcomeRenderInput`/`outcome_render_input(_with_runtime)`** (WI-653,
  `outcome_render.rs`) is the first concrete instance of the
  observe-once/assemble/render separation this initiative wants generalized.

## Physical execution vs. governance decision (P3 factual basis)

`PhysicalSingleFlightCoordinator` lives in `crates/cockpit-verification/
src/lib.rs:1473` — a different crate entirely from `cockpit-repository`,
where all governance-decision code
(`governance_decision_for_contract*`, `require_green_governance*`,
`evaluate_contract_quality_gate`) resides. Crate-level separation between
physical execution and governance judgment already exists; P3 does not need
to invent it. What is not yet cleanly separated: `assess_verification_reuse*`
(the "is this receipt reusable" identity-matching decision) lives in
`cockpit-repository::lib.rs:763`, not in `cockpit-verification` alongside the
coordinator/executor it decides for — the reuse decision and the physical
execution/coalescing mechanism are split across two crates without an
obvious single boundary owner. This split was not examined further in this
Work Item; it is the concrete open question P3 should resolve before
deciding whether to expand execution sharing.

## Candidate refactors: problem, target boundary, compatibility risk, verification

### P1-B — Explicit observation context

**Problem**: lower-level decision functions frequently call
`GitRepository::discover`/`.snapshot()` or re-read Contract/Summary
themselves rather than receiving an already-captured snapshot, so two
functions serving the same request can observe the repository at slightly
different instants. **Target boundary**: entry points capture one snapshot
per effective observation phase (pre-edit, post-execution, pre-persist) and
thread it down; lower-level functions accept a snapshot/context parameter
instead of re-observing. **Compatibility risk**: any function that currently
tolerates a slightly-stale internal re-read must be checked for whether
callers depend on that implicit re-check (e.g. detecting concurrent
modification mid-operation); over-sharing a snapshot across a real edit
boundary would be a correctness regression, not just a refactor. **Verification**:
call-count assertions proving no observation phase is skipped, and explicit
tests for concurrent modification between capture and use.

### P2-A — Extract lifecycle/storage/execution/projection responsibilities

**Problem**: `preflight_work_item_internal`, `finish_work_item_internal`,
`archive_work_item_internal`, and `close_work_item_with_structured_decision_internal`
each embed observation, decision, and persistence in one function body.
**Target boundary**: per this initiative's suggested split (Observation /
Governance / Lifecycle / Evidence / Execution / Projection), migrate one
complete use case at a time — start with the smallest (e.g. checkpoint) to
prove the boundary before touching finish/archive/close. **Compatibility
risk**: public API signatures, `.ai/` file layout, and historical-record
readability must not change; any extracted "Port" must be justified by a
real substitution/fault-injection need, not created per function.
**Verification**: existing integration tests must pass unchanged: they
already assert on file contents and layout, which is the right regression
net for this kind of internal reshuffling.

### P2-B — Tighten state types and legal transitions

**Problem**: lifecycle state, evidence freshness/applicability, governance
decision, human authorization, and historical/superseded status are
currently expressed as ad hoc strings and booleans threaded through many
functions rather than a small set of enums with enforced legal transitions.
**Target boundary**: reuse existing enums (`OutcomeState`, `DecisionState`
already exist and are used); add narrow new types only where a real
missing/invalid/expired/not-applicable distinction is currently collapsed
into one string. **Compatibility risk**: external protocol/JSON
representations must not change without an explicit version/migration path;
historical evidence must never be rewritten to fit a new internal type.
**Verification**: table-test legal combinations and illegal transitions;
replay historical archived records (including unknown/legacy schema
versions) through any new type to confirm they still parse as read-only
history.

### P2-C — Multi-file consistency, concurrency, and recovery

**Problem statement only (not yet investigated for defects, per this
initiative's instruction not to presume one)**: finish/archive/close and
their recovery paths write multiple files per operation; this Work Item did
not audit ordering, idempotency keys, or interruption recovery in depth.
**Target boundary**: identify the one record that represents "committed" for
each multi-file operation, and confirm recovery only ever reconstructs
projections from that authoritative record. **Compatibility risk**: any fix
here is high-blast-radius by nature (it touches commit semantics); it must
not be bundled with unrelated refactors. **Verification**: fault injection
(interrupt after each write step, simulate write failure, run the same
operation twice, run two processes against the same Work Item) is required
before any change is proposed, per the initiative's own instruction to
investigate before assuming a defect.

### P3 — Physical execution / governance-binding separation

**Problem**: as noted above, the reuse-eligibility decision and the physical
execution/coalescing mechanism live in different crates without a single
documented boundary owner, though crate-level separation from governance
judgment already exists. **Target boundary**: confirm (not yet done) that a
shared/reused physical execution result cannot by itself grant a Work Item's
passing state — cache hit, execution success, and governance permission must
remain three separately-checked facts. **Compatibility risk**: any change to
`PhysicalSingleFlightCoordinator` wiring directly affects concurrent
verification correctness and repository isolation; per the performance
initiative's own finding, this coordinator currently has no production
caller, so connecting it is a new architectural commitment, not a
restoration of removed behavior. **Verification**: concurrent-verification
scenarios (same identity, different identity, failure propagation,
cancellation) with resource-peak and execution-count assertions, before any
expansion of execution sharing is considered.
