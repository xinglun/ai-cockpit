# AI Cockpit Collaboration, Observation, and Performance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver typed collaboration semantics, bounded consistent Outcome assembly, real CLI/MCP acceptance, correctly scoped performance evidence, measured optimizations, and a published installable N-1-compatible release.

**Architecture:** Keep one repository-bound assembly path and one typed finalization action projection for CLI, MCP, machine JSON, and localized human rendering. Add a request-local dependency ledger that reuses parsed immutable facts but independently rechecks every dependency and directory candidate at the attempt boundary. Instrument the same command operation with explicit trace identity and real boundary counters; only then evaluate isolated performance optimizations.

**Tech Stack:** Rust workspace (`cockpit-protocol`, `cockpit-repository`, `cockpit-cli`, `cockpit-mcp`, `cockpit-git`), serde JSON, shell/Python performance harnesses, AI Cockpit Runtime 0.2.87, GitHub Actions/Release, and repository-local Work Item evidence.

**Spec:** `docs/superpowers/specs/2026-09-11-collaboration-observability-performance-design.md`

## Global Constraints

- Base revision is `c72254c2ae6bc712139fc755698694653ae004ba`; the active Work Item is `WI-798-collaboration-observability-performance`.
- Preserve the immutable published `v0.2.90` release and discover the next version only after the reviewed merge.
- Do not treat existence, mtime, size, cache hits, or equal summary digests as proof of governance fact validity or cross-file atomicity.
- Pure render functions must not perform filesystem, Git, Runtime, or subprocess I/O.
- Historical Contracts, receipts, archive bytes, and decision records are append-only and must not be rewritten.
- CLI and MCP must consume the same production Outcome assembly and action projection.
- Do not invent human identity, authorization, benefits, external confirmations, or provider-side resource actions.
- Every work package has its own commit and focused acceptance; no unmeasured optimization is retained as a benefit.
- No persistent cross-request cache or resident process is introduced unless a fresh measured bottleneck and invalidation design justify a separately governed change.

Execution order is Task 1, Tasks 2-5 in parallel on disjoint paths, Task 6
integration/baseline, Task 7, Task 8, then Appendix A and Appendix B. Appendix
C records the conflict scan that governs dispatch.

---

## Appendix A: Full integration, documentation, governance, and hosted PR

**Files:**
- Modify: `docs/reference/collaboration-invariant-coverage*.md`.
- Modify: `docs/reference/collaboration-language-contract*.md`.
- Modify: `docs/release/distribution*.md` and `docs/architecture/release-distribution*.md` as required by current release truth.
- Modify: `docs/work-items/WI-798-collaboration-observability-performance*.md`.
- Modify: active Summary/evidence only through Runtime.

**Interfaces:**
- Consumes: implementation commits, focused evidence, performance records, and current remote default branch.
- Produces: finish-ready Work Item and exactly one reviewed PR with no undocumented changed boundary.

- [ ] **Step 1: Run complete local verification**

```bash
cargo fmt --all -- --check
cargo test --locked --workspace
python3 tests/docs/promote_closed_work_item.py --repo "$PWD" --check-all
git diff --check
```

Run all current repository CI/release/governance scripts discovered from manifests. Record every command and result through Runtime evidence; failed gates remain visible and are repaired in scope.

- [ ] **Step 2: Update tri-language behavior and release documentation**

Document correct finalization commands, observation limits, real acceptance scope, measured performance limits, and no-benefit conclusions in English, Simplified Chinese, and Japanese without claiming unsupported platforms or benefits.

- [ ] **Step 3: Finish and archive through Runtime**

```bash
ai-cockpit verify --repo "$PWD" --work-item WI-798-collaboration-observability-performance --command cargo --args test,--locked,--workspace --workers 1 --stage task
ai-cockpit finish --repo "$PWD" --id WI-798-collaboration-observability-performance
ai-cockpit archive --repo "$PWD" --id WI-798-collaboration-observability-performance
```

Before archive, deliver a human Outcome with state, issue count, blockers/stopping reason, resolved issues, unknowns, risks, verification, impact, and next action.

- [ ] **Step 4: Push and open exactly one PR**

Push only `codex/wi-798-collab-observability-performance`, create one PR, and wait for every required hosted check on the exact PR head. Do not merge locally or allow provider-side branch deletion.

---

## Appendix B: Merge, close, publish the next version, and run artifact acceptance

**Files:**
- Modify: `Cargo.toml`, `Cargo.lock`, and release projections only after version discovery and reviewed merge.
- Modify: release/adopter evidence paths through the formal release process.
- Modify: archived Work Item records only through Runtime.

**Interfaces:**
- Consumes: reviewed green PR head, archived Work Item, current tags/Releases, release manifest, and N-1 acceptance harness.
- Produces: merged synchronized main, closed Work Item, immutable next-version Release, downloaded artifact installation evidence, N-1 upgrade evidence, and exact cleanup.

- [ ] **Step 1: Merge only the reviewed green PR head**

Record the exact PR head SHA and hosted check conclusions. Merge through GitHub; do not substitute a local-main merge.

- [ ] **Step 2: Finalize and close the Work Item**

After merge, bind provider/branch/worktree context with `work-item finalize-plan`, record the strict provider receipt with `work-item finalize`, run `work-item finalize-verify`, and close only after archive/decision receipts, merged SHA, fast-forward main synchronization, and exact cleanup are proven.

- [ ] **Step 3: Discover and publish the next immutable version**

Inspect tags and provider Releases independently. Select exactly one patch after the highest reserved version, including tag-only/draft/failed reservations. Publish annotated tag/assets from synchronized main and bind manifest/checksum metadata to the merged commit.

- [ ] **Step 4: Run downloaded installation and N-1 upgrade acceptance**

Use only immutable public artifacts, isolated HOME/XDG_CONFIG_HOME/TMPDIR/CARGO_HOME roots, and the release acceptance harness. Prove version, checksum, Runtime identity, historical-byte preservation, compatibility, cleanup, and built-versus-tested platform distinction. Do not use a workspace binary fallback.

- [ ] **Step 5: Promote closed documentation and clean exact resources**

```bash
python3 tests/docs/promote_closed_work_item.py --repo "$PWD" --check-all
ai-cockpit inspect --repo "$PWD"
ai-cockpit status --repo "$PWD"
ai-cockpit doctor --repo "$PWD"
git status --short --branch
git worktree list --porcelain
```

Delete only the exact merged branch/worktree and verify the remote branch is absent. Preserve PR history, tags, release assets, and evidence-bearing resources. The final human Outcome must begin `Outcome: 🟢`, `Outcome: 🟡`, or `Outcome: 🔴`; use green only with direct current evidence for every acceptance and release requirement.

---

## Appendix C: Pre-dispatch conflict scan

| Pair | Shared interface/path | Finding | Ruling |
|---|---|---|---|
| A/B | `outcome_render.rs` and finalization metadata | A needs typed action data; B needs assembly dependency registration. | B consumes A's projection shape; only the integration owner resolves shared hunks. |
| A/C | protocol projection and CLI/MCP parity tests | C must assert action semantics without recreating A's classifier. | C consumes production structured output and may not duplicate category logic. |
| B/D | `execution_context.rs` and read/hash counters | B boundary rechecks are necessary reads; D must count them separately from reuse. | B defines ledger events; D instruments the ledger API and labels rechecks explicitly. |
| D/E | trace schema and optimization decisions | E is invalid until D's baseline is trustworthy. | Serialize E after Task 6 and require one candidate per commit. |
| A/D | CLI/MCP operation identity | A projection is semantic; D trace identity is diagnostic. | Trace fields never enter semantic/evidence digests. |
| C/docs | matrix and tri-language docs | Matrix is executable acceptance; docs describe behavior and limits. | Update both only from current implementation/evidence. |

| Task | Self-consistency check |
|---|---|
| 1 | Runtime owns generated evidence; Contract amendment precedes design/plan files. |
| 2 | Typed projection is produced before C consumes it; focused tests cover all named states. |
| 3 | Ledger API separates read/reuse/recheck and covers every listed dependency class. |
| 4 | Matrix parser, real fixtures, both endpoints, three languages, two views, and follow-up commands are specified. |
| 5 | Counter tests precede instrumentation; benchmark floors precede optimization. |
| 6 | Integration accepts reconciled A-D commits and freezes evidence before E. |
| 7 | Optimization 1 is independently measurable and reversible. |
| 8 | Each later candidate has its own measurement, correctness check, and commit/revert conclusion. |
| 9 | Finish/archive and hosted PR happen after local evidence; docs promotion is rerun. |
| 10 | Merge precedes finalization/close; release publication uses the newly discovered version and downloaded artifacts. |

### Task 6: Integrate A-D and freeze the corrected performance baseline

**Files:**
- Modify: `.ai/work-items/active/WI-798-collaboration-observability-performance.summary.json` through Runtime controls/evidence commands.
- Modify: `docs/reference/collaboration-invariant-coverage*.md` when implemented evidence changes invariant status.
- Modify: `docs/reference/collaboration-language-contract*.md` when action or observation boundaries change.
- Test: all focused tests from Tasks 2-5.

**Interfaces:**
- Consumes: committed Agent A-D packages and focused results.
- Produces: one integrated state with no overlapping generated evidence, a reproducible baseline, and acceptance records for A-D.

- [ ] **Step 1: Reconcile packages against the current branch**

Inspect each commit's changed paths and Contract scope. Resolve shared `lib.rs`, `outcome_render.rs`, and execution-context hunks as integration-owner changes; never copy conflicting code blindly.

- [ ] **Step 2: Run integrated focused acceptance**

```bash
cargo fmt --all -- --check
cargo test --locked -p cockpit-repository outcome_assembly -- --nocapture
cargo test --locked -p cockpit-repository --test scenario_matrix_next_action -- --nocapture
cargo test --locked -p cockpit-cli --test cli_mcp_outcome_parity -- --nocapture
cargo test --locked -p cockpit-cli --test collaboration_handoff -- --nocapture
cargo test --locked -p cockpit-cli --test performance -- --nocapture
```

- [ ] **Step 3: Freeze baseline scenarios and measurements**

Run the corrected benchmark on controlled small/medium/large repositories with selected clean/change, history, evidence, lifecycle, CLI, MCP, and bounded-concurrency dimensions. Retain raw samples, environment, runtime identity, dataset manifest, trace IDs, p50/p95, and overlap warnings in Runtime-bound evidence. Do not start Task 7 until counters and operation binding pass.

- [ ] **Step 4: Commit integrated A-D result**

```bash
git diff --check
git commit --allow-empty -m "test: freeze collaboration and performance baseline"
```

---

### Task 7: Evaluate measured optimization 1, request-local fact reuse

**Files:**
- Modify: `crates/cockpit-repository/src/execution_context.rs` and/or `observation_ledger.rs`.
- Modify: `crates/cockpit-repository/src/lib.rs` and `outcome_render.rs` consumers of Contract/config/policy/receipt/outcome facts.
- Test: focused observation reuse and invalidation tests.
- Evidence: `.ai/evidence/WI-798-collaboration-observability-performance.optimization-1.json` through Runtime evidence recording.

**Interfaces:**
- Consumes: Task 3 ledger and Task 6 baseline trace.
- Produces: one isolated commit that either improves the declared target without semantic drift or records a measured rollback/no-benefit conclusion.

- [ ] **Step 1: Add a before/after counter assertion**

Run the same operation against the same manifest and assert fewer repeated reads/parses while final boundary rechecks remain present.

- [ ] **Step 2: Implement only content/identity-bound reuse**

Reuse typed parsed facts inside one assembly attempt. Never use mtime, size, watcher events, or an unverified cache entry as validity proof; a failed identity or digest check forces a fresh read.

- [ ] **Step 3: Benchmark and decide**

Require correctness tests, at least 100 valid warm samples, p50/p95, and trace identity. Retain only with measured target improvement and no repeatable key-path regression above 5%; otherwise revert code and record the no-benefit conclusion.

- [ ] **Step 4: Commit or revert the candidate**

```bash
git diff --check
git add crates/cockpit-repository/src/execution_context.rs crates/cockpit-repository/src/observation_ledger.rs crates/cockpit-repository/src/lib.rs crates/cockpit-repository/src/outcome_render.rs
git commit -m "perf: reuse validated facts within Outcome assembly"
```

---

### Task 8: Evaluate optimizations 2-6 one at a time

**Files:**
- Modify: only files justified by evidence, primarily `crates/cockpit-repository/src/lib.rs`, `status_projection.rs`, `execution_context.rs`, `outcome_render.rs`, and `crates/cockpit-git/src/lib.rs`.
- Test: affected focused tests plus Tasks 2-6 regressions.
- Evidence: one Runtime-bound optimization record per candidate.

**Interfaces:**
- Consumes: corrected baseline and retained Task 7 measurements.
- Produces: independent commits/evidence for target-only loading, candidate-index reuse, Git snapshot reuse, reduced cloning/serialization, and bounded parallelism; attempted candidates are explicitly retained or rejected.

- [ ] **Step 1: Select the next candidate only from a measured bottleneck**

If no candidate is a measured bottleneck, record `not_selected` and add no complexity.

- [ ] **Step 2: Implement one bounded candidate**

Keep global status semantics complete, preserve all ledger invalidation cases, and cap any parallel work with an explicit upper bound. Do not introduce cross-request persistence.

- [ ] **Step 3: Repeat correctness and performance acceptance**

Repeat the frozen scenario set, compare raw samples and p50/p95, inspect memory/task growth, and compare typed state/blocker/authorization/action output.

- [ ] **Step 4: Commit retained work or revert with evidence**

Each candidate ends in its own commit or a documented no-benefit/rejected conclusion. Never combine candidates into one unreviewable change.

---

### Task 1: Establish the Work Item and baseline evidence

**Files:**
- Modify: `.ai/work-items/active/WI-798-collaboration-observability-performance.contract.json` through Runtime-owned lifecycle/amendment commands only.
- Modify: `.ai/work-items/active/WI-798-collaboration-observability-performance.summary.json` through Runtime commands only.
- Create: `.ai/evidence/WI-798-collaboration-observability-performance.*` through Runtime evidence commands.
- Test: `crates/cockpit-repository/tests/scenario_matrix_next_action.rs`

**Interfaces:**
- Consumes: base `origin/main`, repository identity, project profile, and the design spec.
- Produces: a checkpointed Contract with sources, six required scenario entries, explicit scope/out-of-scope, and a recorded non-red preflight; red or `needs_human_confirmation` stops the task.

- [ ] **Step 1: Re-read current repository identity and lifecycle state**

```bash
git fetch origin main
ai-cockpit inspect --repo "$PWD"
ai-cockpit status --repo "$PWD"
ai-cockpit doctor --repo "$PWD"
ai-cockpit agent doctor --repo "$PWD" --json
```

Expected: the branch remains bound to the current remote default base; any changed base or stale Contract is repaired before implementation.

- [ ] **Step 2: Record Contract amendment and fresh preflight**

```bash
ai-cockpit work-item revalidate-amendment --repo "$PWD" --id WI-798-collaboration-observability-performance --reason "Bind the approved design and implementation-plan paths and preserve the before-edit checkpoint."
ai-cockpit preflight --repo "$PWD" --contract .ai/work-items/active/WI-798-collaboration-observability-performance.contract.json
```

Expected: green or yellow `verification_pending`; red or `needs_human_confirmation` is a visible stop.

- [ ] **Step 3: Record checkpoint and verify the scenario registry**

```bash
ai-cockpit checkpoint --repo "$PWD" --id WI-798-collaboration-observability-performance
cargo test --locked -p cockpit-repository --test scenario_matrix_next_action
```

Expected: the before-edit checkpoint remains immutable and the registry has no malformed required entry.

- [ ] **Step 4: Commit the governance baseline**

```bash
git add .ai/work-items/active/WI-798-collaboration-observability-performance.contract.json .ai/work-items/active/WI-798-collaboration-observability-performance.summary.json
git commit -m "chore(ai): bind collaboration observability work item"
```

---

### Task 2: Build the typed finalization state and action projection (Agent A)

**Files:**
- Modify: `crates/cockpit-protocol/src/lib.rs` near `ResourceFinalizationError` and `OutcomeFinalizationProjection`.
- Modify: `crates/cockpit-repository/src/lib.rs` near `verify_resource_finalization_internal` and finalization receipt validation.
- Modify: `crates/cockpit-repository/src/outcome_render.rs` near `finalization_projection`, `classify_finalization_error`, and action rendering.
- Modify: `crates/cockpit-cli/src/main.rs` and `crates/cockpit-mcp/src/lib.rs` only for additive projection fields.
- Test: repository Outcome tests and `crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs`.

**Interfaces:**
- Consumes: `ResourceFinalizationError`, receipt/disposition validation, `OutcomeAssemblyMetadata`, and the existing wire projection.
- Produces: `FinalizationObservationState` and `FinalizationActionProjection` with stable serialized identifiers, optional real Runtime command argv, authorization requirement, safety property, and evidence references. Existing string fields remain compatible.

- [ ] **Step 1: Write failing production-fixture tests**

Exercise missing receipt, retained, deleted, abandoned, corrupt, identity mismatch, cleanup postcondition failure, and indeterminate verification. Assert typed state, error category, action identifier, command purpose, authorization requirement, and forbidden deletion/close claims. Run:

```bash
cargo test --locked -p cockpit-repository outcome_render::tests::finalization -- --nocapture
```

- [ ] **Step 2: Replace string classification with exhaustive typed mapping**

Map protocol validation results and explicit missing/cleanup conditions to a typed category. Keep `to_string()` only for diagnostic text; never use it to choose a category or action.

- [ ] **Step 3: Derive machine and human output from one projection**

Make `finalization_projection` construct one additive object. Missing receipt points to the receipt-recording or recovery route followed by verification; retained resources emit no-delete; corrupt/identity mismatch remain blocked and use inspection/recovery; unknown never becomes a close or cleanup authorization.

- [ ] **Step 4: Run focused checks and commit**

```bash
cargo fmt --all -- --check
cargo test --locked -p cockpit-repository outcome_render -- --nocapture
cargo test --locked -p cockpit-cli --test cli_mcp_outcome_parity -- --nocapture
git diff --check
git add crates/cockpit-protocol/src/lib.rs crates/cockpit-repository/src/lib.rs crates/cockpit-repository/src/outcome_render.rs crates/cockpit-cli/src/main.rs crates/cockpit-mcp/src/lib.rs
git commit -m "fix: make finalization recovery actions typed"
```

---

### Task 3: Add the request-scoped dependency ledger and boundary recheck (Agent B)

**Files:**
- Create: `crates/cockpit-repository/src/observation_ledger.rs` if a focused module is needed.
- Modify: `crates/cockpit-repository/src/execution_context.rs` for request-scoped dependency and directory observations.
- Modify: `crates/cockpit-repository/src/outcome_render.rs` for registration and final boundary checks.
- Modify: `crates/cockpit-repository/src/lib.rs` for archive/events/recovery dependency registration.
- Test: Outcome assembly tests and focused repository integration tests.

**Interfaces:**
- Consumes: `RepositoryExecutionContext`, `ObservationContext`, archive manifest verification, events validation, recovery/successor resolution, and Task 2's projection.
- Produces: `ObservationLedger` with focused registration, read-once/parse-once, recheck, and typed drift operations. It never feeds I/O into rendering.

- [ ] **Step 1: Write deterministic mutation tests**

Cover stable assembly; summary, close, finalization, and events mutation; dependency addition/removal/corruption/type replacement; repository/Contract identity change; one-change-then-stable retry; and continuous mutation exhausting the two-attempt budget. No mixed-fact success is accepted.

- [ ] **Step 2: Register all decision dependencies**

Register active/archive Contract, Summary, Outcome, close/preflight decisions, finalization head/transition candidates, task Outcome events, archive-manifest references, historical artifacts, and recovery/successor records. Record complete member sets for decisions, archive, and other candidate directories.

- [ ] **Step 3: Reuse facts inside an attempt and recheck independently**

Route consumers through parsed fact handles. Recheck content, type, identity, and directory member sets after assembly. One drift causes one retry; unstable second attempt returns explicit unknown/error.

- [ ] **Step 4: Separate reuse and boundary counters**

Expose fact reads/parses, reused facts, and final-boundary rechecks independently. Add a test proving a necessary recheck is not mislabeled as a duplicate read.

- [ ] **Step 5: Run focused checks and commit**

```bash
cargo fmt --all -- --check
cargo test --locked -p cockpit-repository outcome_assembly -- --nocapture
cargo test --locked -p cockpit-repository --test scenario_matrix_next_action -- --nocapture
git diff --check
git add crates/cockpit-repository/src/observation_ledger.rs crates/cockpit-repository/src/execution_context.rs crates/cockpit-repository/src/outcome_render.rs crates/cockpit-repository/src/lib.rs
git commit -m "fix: bound Outcome assembly to observed dependencies"
```

---

### Task 4: Convert the collaboration matrix to real production-path acceptance (Agent C)

**Files:**
- Modify: `docs/reference/collaboration-scenario-matrix.json`.
- Modify: `crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs`.
- Modify: `crates/cockpit-cli/tests/collaboration_handoff.rs`.
- Modify: `crates/cockpit-repository/tests/scenario_matrix_next_action.rs`.

**Interfaces:**
- Consumes: Task 2's typed projection, Task 3's observation metadata, production CLI `work-item outcome`, MCP `work_item_outcome`, and lifecycle fixture commands.
- Produces: a matrix executor parsing `inputFacts`, expected state/blockers/action, forbidden inferences, and follow-up commands; it invokes real endpoints rather than constructing only `OutcomeRenderInput`.

- [ ] **Step 1: Extend matrix schema and parser tests**

Require structured state, blocker, action, and forbidden-inference fields for each required scenario. Add a regression proving a changed matrix expectation fails without an implementation change.

- [ ] **Step 2: Build fixtures through lifecycle commands**

Use temporary repositories and repository-bound Runtime commands for active, archived, closed, recovery, stale authorization, missing/corrupt receipt, retained resource, and mutation states. Do not fabricate final projection input.

- [ ] **Step 3: Invoke real CLI and MCP paths**

For each fixture invoke CLI and production MCP with summary/full in en/zh-CN/ja. Compare structured state/action fields and assert blockers, unknowns, authorization boundaries, and no-history recovery information.

- [ ] **Step 4: Execute safe follow-ups**

Run emitted Runtime commands only when the fixture authorizes them, then verify resulting state. For human/provider actions assert blocked state and no deletion/close.

- [ ] **Step 5: Run focused checks and commit**

```bash
cargo fmt --all -- --check
cargo test --locked -p cockpit-cli --test cli_mcp_outcome_parity -- --nocapture
cargo test --locked -p cockpit-cli --test collaboration_handoff -- --nocapture
cargo test --locked -p cockpit-repository --test scenario_matrix_next_action -- --nocapture
git diff --check
git add docs/reference/collaboration-scenario-matrix.json crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs crates/cockpit-cli/tests/collaboration_handoff.rs crates/cockpit-repository/tests/scenario_matrix_next_action.rs
git commit -m "test: exercise collaboration semantics through real paths"
```

---

### Task 5: Correct trace scope, counters, and benchmark sampling (Agent D)

**Files:**
- Modify: `crates/cockpit-repository/src/lib.rs` around `performance_diagnosis`.
- Modify: `crates/cockpit-repository/src/execution_context.rs` for trace identity/counters.
- Modify: `crates/cockpit-git/src/lib.rs` at actual Git invocation boundaries if needed.
- Modify: `crates/cockpit-cli/src/main.rs` and `crates/cockpit-mcp/src/lib.rs` only where operation identity is propagated.
- Modify: `tests/performance/runtime_benchmark.sh`, `runtime_benchmark_stats.py`, and `runtime_benchmark_scenarios.py`.
- Test: `crates/cockpit-cli/tests/performance.rs` and focused trace/counter tests.

**Interfaces:**
- Consumes: request-scoped observation context and actual file/Git/parse functions.
- Produces: operation-bound traces with operation/scenario/measurement IDs, runtime/repository identity, parent spans, overlap metadata, raw sample order, and unavailable reasons.

- [ ] **Step 1: Write failing scope and counter tests**

Use controlled fixtures with known files, bytes, parses, and Git calls. Assert that an added governance-file read increments the operation counter, diagnose counters do not appear in status, and benchmark metadata/probes are not Runtime child processes.

- [ ] **Step 2: Instrument actual boundaries**

Count file reads, content hashes, JSON parses, and Git calls where they occur. Create one trace per measured operation and preserve parent/child relationships without adding nested wall spans as independent elapsed time.

- [ ] **Step 3: Separate benchmark metrics**

Keep process startup, probes, warmups, metadata Git calls, and Runtime spans distinct. Unmeasurable values are unavailable with a reason, never guessed zeroes.

- [ ] **Step 4: Enforce valid sampling**

Default to at least 100 valid warm samples, preserve the first sample separately, retain raw order, and report p50/p95 only after validity floors. State that first sample is not cold-cache without OS cache control. Add diagnostics off/on and absolute short-command overhead.

- [ ] **Step 5: Run focused checks and commit**

```bash
cargo fmt --all -- --check
cargo test --locked -p cockpit-cli --test performance -- --nocapture
python3 tests/performance/runtime_benchmark_stats.py --help
tests/performance/runtime_benchmark.sh --help
git diff --check
git add crates/cockpit-repository/src/lib.rs crates/cockpit-repository/src/execution_context.rs crates/cockpit-git/src/lib.rs crates/cockpit-cli/src/main.rs crates/cockpit-mcp/src/lib.rs tests/performance crates/cockpit-cli/tests/performance.rs
git commit -m "perf: bind diagnostics to measured operations"
```

---
