# Plan: trust diagnostics and observation-bound Outcome assembly

> **For the implementing agent:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to execute this plan task by task.

**Goal:** Complete the five strict-review work packages and leave a verified,
reviewable implementation ready for a separate release Work Item.

**Architecture:** Keep repository I/O in a bounded Outcome assembly layer that
produces an immutable render input. Add typed reason/finalization projections
to that input, centralize locale-independent semantic keys, and make summary
and full rendering projections share the same keys. Expose measured Runtime
diagnostics through an independent JSON channel consumed by the benchmark
harness. Bind collaboration scenarios to executable semantic assertions.

**Tech stack:** Rust workspace, serde JSON protocol, CLI/MCP shared repository
rendering, shell/Python performance harness, existing AI Cockpit Runtime.

## Task 1: Establish the observation-bound assembly contract

**Files:** `crates/cockpit-repository/src/execution_context.rs`,
`crates/cockpit-repository/src/lib.rs`,
`crates/cockpit-repository/src/outcome_render.rs`, focused repository tests.

1. Add a bounded assembly result/context that records the repository snapshot,
   relevant lifecycle/close/finalization identities, and whether validation
   required a retry. Reuse `ObservationContext`; do not add global caching.
2. Assemble Outcome, human decision, lifecycle status, and finalization facts
   from that boundary. Detect deterministic mutations of close/lifecycle/
   finalization records and return an explicit unknown/failure after the fixed
   retry budget.
3. Keep `render_human_outcome_with_view` and its helpers free of filesystem or
   Runtime I/O. Preserve the legacy helper only as an I/O-to-input adapter.
4. Add deterministic stable, mutation, corruption, identity-drift, bounded
   retry, and pure-render tests; retain historical fixture coverage.
5. Run `cargo fmt --all -- --check` and the focused repository tests.

## Task 2: Add shared governance reason and finalization action projections

**Files:** `crates/cockpit-protocol/src/lib.rs`,
`crates/cockpit-repository/src/outcome_render.rs`,
`crates/cockpit-repository/src/lib.rs`, protocol/repository/CLI tests.

1. Add optional serde-compatible typed fields for reason keys and finalization
   recovery facts; derive them from existing failed-gate, evidence, contract,
   resource, and receipt validation models.
2. Project distinct keys for verification failure, invalid/mismatched/stale
   evidence, acceptance evidence, intent alignment, range, authority, unknown,
   and finalization-specific states. Preserve multiple simultaneous reasons in
   stable order.
3. Project actions only when supported by current valid resource facts and
   Runtime rules: inspect/repair record, re-observe, re-verify, clean, retain,
   or human decision. Never turn prose into authorization.
4. Make summary/full and all three locales consume the same keys. Add A/B
   counterexamples for valid verification plus red acceptance/intent gaps,
   invalid/identity-mismatched/stale evidence, and all required finalization
   states.
5. Run protocol serialization compatibility and focused rendering/recovery
   tests.

## Task 3: Bind collaboration scenarios to human semantic checks

**Files:** `docs/reference/collaboration-scenario-matrix.json`,
`crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs`,
`crates/cockpit-cli/tests/collaboration_handoff.rs`,
`crates/cockpit-repository/tests/scenario_matrix_next_action.rs`, related test
helpers/docs.

1. Extend the scenario matrix with stable fact selectors, semantic invariants,
   and executable check identifiers; load those values from tests rather than
   duplicating expected answers.
2. Assert human summary/full reports directly across CLI/MCP and locales for
   blockers, unknowns, authority boundary, and operational consequence. Keep
   exact comparison only for fixed protocol messages.
3. Add no-chat-history handoff tests for target, scope, state, evidence,
   applicable authorization, and legal next action, including stale/inapplicable
   authority and A/B counterexamples.
4. Make changing a scenario expectation without implementation coverage fail a
   binding test. Run focused collaboration and matrix tests.

## Task 4: Add measured Runtime performance diagnostics

**Files:** repository/runtime diagnostics implementation, CLI output route,
`tests/performance/runtime_benchmark.sh`, performance Python tests and
tri-language README files.

1. Add versioned independent diagnostics for identity, snapshot/Git, read/hash,
   parse, evidence/governance, scheduling/subprocess, and projection/serialization
   spans, with explicit nesting/overlap metadata.
2. Publish measured read bytes, hashed bytes, Git calls, and child-process counts
   where Runtime supports them. Mark unsupported platform metrics with a reason;
   do not use zero or benchmark-tool counts as Runtime counts.
3. Fix and unit-test macOS and Linux filesystem-type discovery, including command
   failure and unknown output. Retain existing sample grouping/raw data and
   insufficient-sample handling.
4. Capture ordinary and large-history repository scenarios and report bottleneck
   evidence without claiming an unverified speedup. Include diagnostic overhead
   in the declared budget.
5. Run performance unit/gate checks and inspect raw JSON artifacts.

## Task 5: Integrate, verify, and prepare handoff

**Files:** all changed files plus governance evidence.

1. Re-run the full workspace format, static checks, tests, docs/governance gates,
   scenario binding, historical compatibility, CLI/MCP multilingual checks,
   deterministic assembly mutation checks, and performance captures.
2. Review the diff for scope, immutable-history preservation, unsupported claims,
   and documentation/capability consistency. Do not update snapshots merely to
   hide a failure.
3. Update WI-781 verification evidence, finish/archive it, push the branch, and
   open the reviewed PR. Merge only after hosted checks pass, then close and run
   the required closed-work-item projection audit.
4. Synchronize main and verify `ready_on_base`. Create a separate release Work
   Item only after this implementation WI is closed; discover the next version
   from current package/tag/release state, publish through the existing release
   workflow, perform artifact install/upgrade adoption checks, and close that WI.

## Verification commands

```text
cargo fmt --all -- --check
cargo test --locked --workspace
python3 tests/docs/governance_integrity_gate.py --repo <repo> --report <report>
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
tests/performance/runtime_benchmark.sh --help
```

The final command set is expanded with the repository's current release and
performance checks after implementation; no command is treated as passed until
its output is captured.
