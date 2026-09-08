# Status Fact Reuse Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce repeated finalization-directory reads and transition parsing inside one repository `status` observation while preserving every existing fail-closed boundary.

**Architecture:** Build a request-scoped `FinalizationTransitionIndex` from the repository's regular, repository-local `.ai/decisions` entries once. Resolve each Work Item's append-only finalization chain from the already parsed transition values, retaining the original path and digest inputs for existing validation. Pass the index only through the status/readiness projection; no process-global or cross-request state is introduced.

**Tech Stack:** Rust 2024 workspace, `serde_json`, existing `cockpit-repository` finalization validators, shell benchmark and JSON evidence.

**Spec:** `.ai/work-items/active/WI-685-p1-status-fact-reuse.contract.json`

## Global Constraints

- Preserve the Contract's repository/worktree identity, evidence binding, authorization boundary, and fail-closed errors.
- Use only the existing status observation; do not add a resident cache, incremental content proof, parallel I/O, or scheduler change.
- Keep canonical receipts and transition bytes immutable; retain deterministic ordering and digest validation.
- Measure baseline before candidate and report unavailable CPU/I/O/memory metrics as unavailable.
- Do not modify other agent worktrees, branches, PRs, or generated archive records.

### Task 1: Add a failing transition-index read-count regression

**Files:**
- Modify: `crates/cockpit-repository/src/lib.rs` test-only counters and finalization helper tests
- Modify: `crates/cockpit-repository/src/status_projection.rs` test fixture entry point
- Test: `crates/cockpit-repository/tests/status_projection.rs` for observable status parity

**Interfaces:**
- Consumes: existing `ResourceFinalizationReceipt`, `ResourceFinalizationTransitionReceipt`, and `status_with_runtime` paths.
- Produces: a test-only counter that distinguishes transition bytes parsed from directory entries enumerated, without adding a production metric.

- [ ] **Step 1: Write the failing test**

  Create a temporary repository fixture with one canonical legacy finalization receipt and one valid transition, invoke the status projection path that reaches historical inventory, and assert that the transition parser is invoked once per transition file for one observation. Add a second assertion that two independent status calls do not reuse the first observation's counter or parsed value.

- [ ] **Step 2: Run the focused test to verify the expected failure**

  Run `cargo test --locked -p cockpit-repository --test status_projection historical_finalization_transition_is_parsed_once_per_observation -- --exact`.

  Expected result: the new test fails because the current resolver reparses or rescans the transition candidate for each finalization head and does not expose one-observation parsing.

- [ ] **Step 3: Commit only the test and test-only counter**

  Use `git diff --check`, then commit the focused red test as `test: bind WI-685 transition parse count`.

### Task 2: Implement the bounded request-scoped index

**Files:**
- Modify: `crates/cockpit-repository/src/status_projection.rs`
- Modify: `crates/cockpit-repository/src/lib.rs`
- Test: `crates/cockpit-repository/tests/status_projection.rs` and the Task 1 focused test

**Interfaces:**
- Consumes: one fresh repository root, one optional Runtime identity, and the current decisions directory.
- Produces: `FinalizationTransitionIndex` keyed by Work Item ID, with parsed transition, source path, content digest, and deterministic candidate ordering; `resolve_resource_finalization_head_with_index` consumes it without re-enumerating or reparsing the same transition bytes.

- [ ] **Step 1: Materialize only regular, non-symlink transition files**

  Enumerate `.ai/decisions` once, retain canonical finalization paths and transition paths, reject or preserve the same malformed/path errors as the current resolver, validate the filename digest against the parsed transition JSON, and sort by path before indexing.

- [ ] **Step 2: Route only one `status` observation through the index**

  Create the index after the existing Runtime/repository identity and Git snapshot are captured. Pass it to historical inventory and the finalization-head resolver. Keep migration, explicit finalization, close, and other commands on their existing paths unless the same helper is required without changing their observation boundary.

- [ ] **Step 3: Resolve chains from indexed values**

  Match exactly one predecessor digest at each sequence, preserve `validate_resource_finalization_transition` and `validate_governance_append_revision`, reject forks and missing predecessors, and return the same head path/digest/sequence. Never reuse the index after the status call returns.

- [ ] **Step 4: Run the focused test to verify green**

  Run `cargo test --locked -p cockpit-repository --test status_projection historical_finalization_transition_is_parsed_once_per_observation -- --exact` and then `cargo test --locked -p cockpit-repository --test resource_finalization_transition --test status_projection`.

  Expected result: the counter test passes, all existing transition negative tests pass, and the second independent status call remains isolated.

### Task 3: Measure candidate and decide acceptance

**Files:**
- Create: `.ai/evidence/external/WI-685-p1-status-fact-reuse.*`
- Modify: `.ai/work-items/active/WI-685-p1-status-fact-reuse.contract.json` only through Contract amendment/revalidation when evidence scope changes

**Interfaces:**
- Consumes: baseline `/private/tmp/wi-684-baseline-round20.json`, candidate release binary, latest-base repository facts, and the existing regression gate.
- Produces: raw baseline/candidate samples, comparability, normalized output comparison, count evidence, rejection or acceptance decision, and platform limitation records.

- [ ] **Step 1: Build unchanged baseline and candidate binaries**

  Build both from the same `2dee60a8aa97ba229d117cb210b245894b3347d2` base/toolchain into isolated `/private/tmp` paths; record Runtime version and SHA-256 identity for both.

- [ ] **Step 2: Capture the required scenarios without changing the benchmark**

  Run the corrected `tests/performance/runtime_benchmark.sh` for `many-historical-wi` with at least 20 warm samples in development and 40 warm samples for the final comparison. Preserve first measurement, warmup, raw samples, p50/p95, p99 availability, environment, data scale, and unavailable resource metrics.

- [ ] **Step 3: Compare governance results and failure paths**

  Normalize only Runtime identity and capture-time fields; require inspect/status/doctor/observe exit-code and output parity, plus malformed/forked/missing transition parity. Record actual counter deltas and do not infer CPU/I/O/memory improvements from wall time.

- [ ] **Step 4: Apply the Contract gate**

  Accept only if status p50 and p95 improve and all other declared budgets hold. Otherwise write an explicit `reject_candidate` evidence record and revert the production candidate before finish.

### Task 4: Verify, document, and deliver through the lifecycle

**Files:**
- Create: `docs/work-items/WI-685-p1-status-fact-reuse.md`
- Create: `docs/work-items/WI-685-p1-status-fact-reuse.zh-CN.md`
- Create: `docs/work-items/WI-685-p1-status-fact-reuse.ja.md`
- Modify: `docs/reference/reference-parity.md`
- Modify: `docs/reference/reference-parity.zh-CN.md`
- Modify: `docs/reference/reference-parity.ja.md`
- Generated by Runtime: `.ai/work-items/archive/**`, `.ai/decisions/**`, `.ai/evidence/**` lifecycle records

**Interfaces:**
- Consumes: final candidate/rejection evidence and all focused correctness results.
- Produces: current Summary/Outcome, three-language parity, reviewed PR, merged head or explicit blocked finalization, and closed Work Item only after provider and cleanup evidence are valid.

- [ ] **Step 1: Run all declared local checks**

  Run `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`, `cargo test --locked --workspace`, governance/documentation checks, and `git diff --check`.

- [ ] **Step 2: Update the Summary and three-language docs**

  Report bottleneck evidence, optimization boundary, raw and summary measurements, correctness, regression/platform limits, unverified metrics, PR identity, and final governance state; never claim a user-visible benefit without evidence.

- [ ] **Step 3: Finish and archive through Runtime**

  Execute `verify → finish → archive`, commit only the active Work Item's declared paths and generated archive bundle, run the repository PR gate, push the dedicated branch, and open one PR.

- [ ] **Step 4: Review, merge, close, and prove cleanup**

  Wait for all required hosted checks, merge only the reviewed PR, synchronize `origin/main`, run Runtime close with a structured human decision, prove exact branch/worktree cleanup, and run `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`.

## Self-review

The plan covers the only currently measured bottleneck (`status` on 594 archived Work Items), the independent transition-parsing hypothesis, the required valid and fail-closed scenarios, the full benchmark comparison, and the governed delivery boundary. It deliberately excludes the previously rejected candidate-path-only index, all P2/P3 candidates, and unrelated repository refactors.
