# Cross-Work-Item Coordination and Pre-Merge Composition Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement an evidence-bound collaboration loop for explicitly participating linked-worktree Work Items while preserving ordinary single-WI lifecycle behavior and stopping before release.

**Architecture:** Add strict protocol types and a repository-local coordination store under the Git common directory. Route registration, impact reporting, coordination requests, recovery, dependency admission, and composition verification through repository services that validate current Git/Contract/runtime identities. Keep CLI, MCP, and Outcome as thin projections of one typed repository result; keep reads side-effect free and writes explicit.

**Tech Stack:** Rust 2024 workspace, `serde`/`serde_json`, `sha2`, existing `cockpit-git`, `cockpit-repository`, `cockpit-verification`, `cockpit-cli`, `cockpit-mcp`, `cockpit-agent`, Cargo tests, real Git linked worktrees, Python repository gates.

**Spec:** `docs/superpowers/specs/2026-09-24-cross-wi-coordination-composition-design.md`

## Global Constraints

- Same-machine, one Git common directory, multiple linked worktrees only.
- Coordination records live under `<git-common-dir>/.ai-cockpit/coordination/v1/` and are observations/events, not a second authority.
- Fixed lifecycle Runtime: `0.2.105`, digest `sha256:43a8ed731ac3187868e8d0c021be2a1a1b72986885d13bf3373f2ed5a0296b04`.
- Candidate collaboration operations require an explicit candidate capability/runtime binding and fail closed before writes when it is absent or mismatched.
- Existing Contract files remain readable by the fixed Runtime; no claim of bidirectional compatibility is allowed.
- Ordinary single-WI operations do not scan coordination state or require declarations.
- Reads are side-effect free; registration, reporting, requests, acknowledgements, resume, recovery, and composition verification are explicit writes/actions.
- Atomic writes use exclusive lock plus temporary file and rename; corrupt, stale, missing, moved, or deleted state is unknown/recovery-required, never free.
- Multi-resource acquisition has canonical ordering and rollback; generation changes reject old owners and late writes.
- A composition receipt binds target, participant heads, order, Contract/declaration identities, toolchain/configuration inputs, and execution result.
- Preconditions run before expensive verification; a precondition failure starts zero expensive verification processes.
- Historical Runtime-generated Contract, Summary, receipt, archive, decision, and status files are never hand-edited.
- No push, merge, release, tag mutation, provider cleanup, daemon, cross-machine coordination, independent-clone global lock, or global skill change.

## Review Focus

- Runtime skew: old `0.2.105` must not consume or silently ignore candidate collaboration records; candidate operations reject mismatched runtime capability before persistence.
- Crash/partial write: readers must classify incomplete or corrupt records as unknown/recovery-required, and recovery must reject stale generations.
- Late/duplicate event: event identity and generation rules must preserve the newest state without duplicate side effects.
- Safe pause boundary: a running action is not falsely reported as paused; request, acknowledgement, safe pause, expiry, and resume are distinct.
- Composition reuse: unchanged filenames/commands alone do not reuse evidence; all relevant source, dependency, interface, config, toolchain, generated-input, environment, and verifier identities must be considered.

## File Map

- `crates/cockpit-protocol/src/lib.rs`: additive collaboration declarations, stages, event/request/reservation/receipt identities, runtime capability binding, and projection wire types.
- `crates/cockpit-protocol/tests/collaboration.rs`: strict round trips, unknown-field rejection, stage/request transitions, and old Contract compatibility.
- `crates/cockpit-git/src/lib.rs`: Git common-directory and linked-worktree discovery helpers that work when `.git` is a file.
- `crates/cockpit-git/tests/collaboration_topology.rs`: real repository topology identity and unsupported independent-clone cases.
- `crates/cockpit-repository/src/coordination_store.rs`: lock, atomic record IO, registration, reservation, event, request, generation, deduplication, recovery, and read/write boundary.
- `crates/cockpit-repository/src/collaboration.rs`: declaration loading, dependency/impact graph, cycle diagnostics, action admission, runtime capability gate, and unified collaboration projection.
- `crates/cockpit-repository/src/lib.rs`: module registration and public repository service adapters; ordinary paths remain unchanged unless collaboration is explicitly requested.
- `crates/cockpit-repository/tests/coordination_store.rs`: concurrent resource, partial-write, stale-generation, duplicate-event, recovery, request-state, and single-WI cost scenarios.
- `crates/cockpit-repository/tests/collaboration_admission.rs`: dependency stages, local continuation, cycle detection, runtime binding, and refresh-before-action behavior.
- `crates/cockpit-verification/src/composition.rs`: isolated composition construction, precondition ordering, execution persistence, exact identity binding, and node reuse classification.
- `crates/cockpit-verification/src/lib.rs`: public composition service and existing verification integration.
- `crates/cockpit-verification/tests/composition.rs`: interface error before merge, zero-expensive-process precondition failure, identity invalidation, failure persistence, and safe reuse.
- `crates/cockpit-cli/src/main.rs`: candidate-only `work-item coordination` and `work-item composition` action parsing and projection; existing `work-item parallel` remains compatible.
- `crates/cockpit-cli/tests/collaboration_cli.rs`: explicit read/write semantics, runtime mismatch, recovery, and ordinary-WI behavior through subprocesses.
- `crates/cockpit-mcp/src/lib.rs`: schemas and dispatch for the same coordination/composition domain operations.
- `crates/cockpit-mcp/tests/collaboration_rpc.rs`: CLI/MCP input and result parity, read idempotency, and action rejection.
- `crates/cockpit-repository/src/outcome_render.rs` and `crates/cockpit-agent/src/lib.rs`: human Outcome projection from the shared collaboration result.
- `crates/cockpit-cli/tests/collaboration_outcome_parity.rs`: summary/full and en/zh-CN/ja semantic parity.
- `docs/reference/cross-wi-coordination.md`, `.zh-CN.md`, `.ja.md`: supported commands, state model, runtime boundary, and recovery.
- `docs/features/cross-wi-coordination.md`, `.zh-CN.md`, `.ja.md`: user-facing capability and ordinary-WI boundary.
- `tests/ci/repository_gate_manifest.json` and related gate tests: only if the new production-path regressions require a gate entry; use the existing single-writer gate route.
- `tests/performance/cross_wi_coordination.py` and `tests/performance/cross_wi_coordination_test.py`: cold/warm cost observations, raw samples, unavailable reasons, and no unproven benefit claim.

## Execution Order

### Task 0: Bind the governance baseline and prove the clean starting point

**Files:**
- Runtime-owned: `.ai/work-items/active/WI-1029-cross-wi-coordination-composition.*`, `.ai/decisions/observer-snapshot.json`
- Existing committed inputs: `docs/superpowers/specs/2026-09-24-cross-wi-coordination-composition-design.md`

**Interfaces:**
- Consumes: `origin/main=790b89e278e21bc768d3e3e7290f67b8ca26603a`, Runtime `0.2.105` identity, approved spec.
- Produces: checkpointed WI-1029 with scenario coverage and the recorded before-edit snapshot.

- [x] **Step 1: Create and preflight the Work Item.**

  Runtime `start --prepare`, additive `work-item amend`, and `preflight` have already produced an authorized `verification_pending` result with all high-risk scenarios declared.

- [x] **Step 2: Commit the design spec.**

  Commit `a87aeec0` contains only the approved design spec; Runtime records remain unedited.

- [x] **Step 3: Record the before-edit checkpoint.**

  `ai-cockpit checkpoint --repo <worktree> --id WI-1029-cross-wi-coordination-composition` returned `state=checkpointed`.

- [x] **Step 4: Run the clean baseline.**

  Run:

  ```bash
  cargo fmt --all -- --check
  cargo test --locked --workspace
  python3 tests/ci/run_repository_gates.py --help
  ```

  Record failures without rerunning blindly. If the baseline is red, add a recovery note to the Summary and repair only an in-scope defect before implementation.

- [x] **Step 5: Commit the Runtime-generated governance baseline.**

  Stage only Runtime-generated active Contract/Summary/observer snapshot plus the plan once it exists; use `git diff --check`, then commit `chore(ai): bind WI-1029 collaboration composition baseline`.

### Task 1: Add strict protocol identities and atomic coordination storage

**Files:**
- Modify: `crates/cockpit-protocol/src/lib.rs`
- Create: `crates/cockpit-protocol/tests/collaboration.rs`
- Modify: `crates/cockpit-git/src/lib.rs`
- Create: `crates/cockpit-git/tests/collaboration_topology.rs`
- Create: `crates/cockpit-repository/src/coordination_store.rs`
- Modify: `crates/cockpit-repository/src/lib.rs`
- Create: `crates/cockpit-repository/tests/coordination_store.rs`

**Interfaces:**
- Consumes: existing `Contract`, `ConcurrencyBoundary`, `ParallelSlotLease`, repository identity, and Git snapshot helpers.
- Produces: `CollaborationDeclaration`, `OutcomeStage`, `WorktreeRegistration`, `CoordinationEvent`, `CoordinationRequest`, `ResourceReservation`, `CompositionBinding`, `RuntimeCapabilityBinding`, and `CoordinationStore` methods:
  `register`, `publish_event`, `reserve_resources`, `release_resources`, `request_coordination`, `transition_request`, `recover`, `inspect`.

- [x] **Step 1: Write protocol failing tests.**

  Assert JSON round trips for every new identity, `deny_unknown_fields` rejects an unknown field, stage transitions do not equate closure with `composable_head`, old Contracts without collaboration fields remain readable, and a runtime binding mismatch is an error.

- [x] **Step 2: Run the protocol tests and observe the expected failure.**

  ```bash
  cargo test --locked -p cockpit-protocol --test collaboration
  ```

  Expected failure: the new protocol types and runtime binding are not defined.

- [x] **Step 3: Write Git topology failing tests using real linked worktrees.**

  Create a temporary repository, a branch, and a linked worktree. Assert that common-directory discovery resolves the same path from both worktrees and that an independent clone is classified unsupported. Include the `.git`-file topology.

- [x] **Step 4: Implement minimal topology helpers.**

  Add repository-bound helpers that invoke the existing Git abstraction, normalize canonical paths, and return typed topology identity; do not infer common directory from path layout.

- [x] **Step 5: Write coordination-store failing tests.**

  Exercise two processes reserving the same resource, multi-resource rollback, duplicate registration/event, partial record bytes, stale generation, moved worktree, and recovery. Assert no stale/corrupt record becomes free and no old owner can append after generation change.

- [x] **Step 6: Implement atomic storage.**

  Store records beneath `<git-common-dir>/.ai-cockpit/coordination/v1/`. Use an exclusive lock, canonical resource ordering, temporary regular files, flush/rename, and strict read validation. Give every write a stable operation/event identity and make duplicate writes return the existing result without a second mutation.

- [x] **Step 7: Run Task 1 tests and refactor only while green.**

  ```bash
  cargo fmt --all -- --check
  cargo test --locked -p cockpit-protocol --test collaboration
  cargo test --locked -p cockpit-git --test collaboration_topology
  cargo test --locked -p cockpit-repository --test coordination_store
  ```

- [x] **Step 8: Commit the protocol/storage layer.**

  ```bash
  git diff --check
  git add crates/cockpit-protocol crates/cockpit-git crates/cockpit-repository/src/coordination_store.rs crates/cockpit-repository/src/lib.rs crates/cockpit-repository/tests/coordination_store.rs
  git commit -m "feat: add cross-WI coordination identities and atomic store"
  ```

### Task 2: Implement dependency, impact, and coordination control

**Files:**
- Create: `crates/cockpit-repository/src/collaboration.rs`
- Modify: `crates/cockpit-repository/src/lib.rs`
- Create: `crates/cockpit-repository/tests/collaboration_admission.rs`
- Modify: `crates/cockpit-protocol/tests/collaboration.rs`

**Interfaces:**
- Consumes: Task 1 protocol/store, active and archived Work Item facts, Git worktree registrations, and fixed/candidate Runtime identity.
- Produces: `collaboration_projection`, `admit_collaboration_action`, `refresh_dependency_state`, `report_impact`, `request_safe_pause`, `acknowledge_pause`, `resume_and_re_evaluate`, and typed diagnostics for cycles, unsupported runtime, stale requests, and affected-only blocking.

- [ ] **Step 1: Write failing admission tests.**

  Cover `interface_stable`, `composable_head`, and `merged_target`; provider closure must not be required for a composable head. Add a provider change that invalidates only its consumers, an unrelated WI that remains runnable, a two-edge cycle, and a late event that targets an old generation.

- [ ] **Step 2: Write failing coordination state-machine tests.**

  Exercise `requested -> acknowledged -> paused -> resumed`, unavailable/expired requests, duplicate requests, and running-action safe-boundary behavior. Assert that resume refreshes current facts before admission.

- [ ] **Step 3: Write the fixed/candidate Runtime boundary tests.**

  Pass the fixed `0.2.105` identity to candidate collaboration actions and assert `unsupported_runtime_capability` with zero coordination writes. Pass the candidate capability and assert the action proceeds. Keep ordinary single-WI lifecycle tests on the fixed Runtime path.

- [ ] **Step 4: Implement declaration loading and graph evaluation.**

  Read explicit declarations through the store, validate current worktree/Contract/head identity, propagate invalidation by dependency edges, coalesce equivalent consumer invalidations, and mark unknown edges conservatively.

- [ ] **Step 5: Implement explicit write operations.**

  `report_impact` appends an idempotent event; request/acknowledge/resume operations append transitions bound to target generation; recovery consumes only matching identities and never repairs on a read. The admission service refreshes the store and Git facts before every dependent action.

- [ ] **Step 6: Implement cycle and local-continuation diagnostics.**

  Return the cycle path and actionable edge. Block only affected actions; do not pause the entire WI or permit a dependency edge to be bypassed by a coordinator message.

- [ ] **Step 7: Run Task 2 tests.**

  ```bash
  cargo test --locked -p cockpit-repository --test collaboration_admission -- --nocapture
  cargo test --locked -p cockpit-repository --test coordination_store -- --nocapture
  cargo test --locked -p cockpit-protocol --test collaboration
  ```

- [ ] **Step 8: Commit the dependency/coordination layer.**

  ```bash
  git diff --check
  git add crates/cockpit-repository/src/collaboration.rs crates/cockpit-repository/src/lib.rs crates/cockpit-repository/tests/collaboration_admission.rs crates/cockpit-protocol/tests/collaboration.rs
  git commit -m "feat: add cross-WI impact and coordination control"
  ```

### Task 3: Implement exact composition verification and evidence reuse

**Files:**
- Create: `crates/cockpit-verification/src/composition.rs`
- Modify: `crates/cockpit-verification/src/lib.rs`
- Create: `crates/cockpit-verification/tests/composition.rs`
- Modify: `crates/cockpit-repository/src/collaboration.rs`
- Modify: `crates/cockpit-repository/tests/collaboration_admission.rs`

**Interfaces:**
- Consumes: Task 1 `CompositionBinding`, Task 2 admission/refresh service, existing verification executor and repository Git helpers.
- Produces: `CompositionAttempt`, `CompositionPrecondition`, `CompositionExecutionRecord`, `ReuseDecision`, `run_composition`, `classify_reuse`, and a typed result for projection/Outcome.

- [ ] **Step 1: Write failing composition tests.**

  Use real commits and a temporary linked worktree to show an interface error before merge despite no text conflict. Bind target/head/order/Contract/declaration identities and assert the receipt records all of them.

- [ ] **Step 2: Write the zero-expensive-process test.**

  Inject a missing dependency, resource conflict, runtime mismatch, or identity mismatch before verification execution. Assert the execution counter is zero and a failure attempt is readable.

- [ ] **Step 3: Write failure/recovery and safe-reuse tests.**

  Persist timeout/interruption/unknown-exit records, retry without overwriting the old record, change each binding input one at a time, and assert invalidation. Reuse only a node with matching source/transitive-dependency/interface/config/toolchain/lockfile/generated-input/environment/verifier identities.

- [ ] **Step 4: Implement isolated composition construction.**

  Build the exact participant order in a temporary worktree without modifying participant worktrees. Detect text conflicts, direct interface constraints, and resource/dependency invalidation before spawning expensive checks.

- [ ] **Step 5: Implement execution persistence and reuse.**

  Save failed attempts before returning errors; require unambiguous exit status/logs for success; reference immutable predecessor receipts for reuse; never rewrite old receipts or turn an unknown result into success.

- [ ] **Step 6: Run Task 3 tests.**

  ```bash
  cargo test --locked -p cockpit-verification --test composition -- --nocapture
  cargo test --locked -p cockpit-repository --test collaboration_admission -- --nocapture
  ```

- [ ] **Step 7: Commit the composition layer.**

  ```bash
  git diff --check
  git add crates/cockpit-verification crates/cockpit-repository/src/collaboration.rs crates/cockpit-repository/tests/collaboration_admission.rs
  git commit -m "feat: verify exact WI compositions before merge"
  ```

### Task 4: Add explicit CLI/MCP/Outcome projection, docs, measurements, and integrated gates

**Files:**
- Modify: `crates/cockpit-cli/src/main.rs`
- Create: `crates/cockpit-cli/tests/collaboration_cli.rs`
- Modify: `crates/cockpit-mcp/src/lib.rs`
- Create: `crates/cockpit-mcp/tests/collaboration_rpc.rs`
- Modify: `crates/cockpit-repository/src/outcome_render.rs`
- Modify: `crates/cockpit-agent/src/lib.rs`
- Create: `crates/cockpit-cli/tests/collaboration_outcome_parity.rs`
- Create/modify: `docs/reference/cross-wi-coordination.{md,zh-CN.md,ja.md}`
- Create/modify: `docs/features/cross-wi-coordination.{md,zh-CN.md,ja.md}`
- Create: `tests/performance/cross_wi_coordination.py`
- Create: `tests/performance/cross_wi_coordination_test.py`
- Modify only if required: `tests/ci/repository_gate_manifest.json` and its gate tests

**Interfaces:**
- Consumes: Task 2 projection and Task 3 composition result.
- Produces: candidate CLI operations `work-item coordination` and `work-item composition`, MCP `work_item_coordination` and `work_item_composition`, shared Outcome fields, tri-language docs, and cost evidence.

- [ ] **Step 1: Write failing CLI/MCP schema and parity tests.**

  Define read actions (`inspect`, status projection) separately from write actions (register, report, request, acknowledge, resume, recover, composition). Assert repeated reads do not change coordination digests and that CLI/MCP return identical typed facts.

- [ ] **Step 2: Implement candidate CLI/MCP adapters.**

  Discover and validate exact current parser/capability surfaces, then translate arguments to the repository services without owning domain rules or adding implicit writes. Reject fixed `0.2.105` for candidate-only collaboration actions before mutation.

- [ ] **Step 3: Write failing Outcome tests.**

  Assert summary/full and en/zh-CN/ja outputs separately state implementation, composition, target merge, cleanup, owner, blocker, unknown, revalidation, reusable checks, next action, and human decision. A failure without unique attribution must render `joint_diagnosis_required`.

- [ ] **Step 4: Implement shared Outcome projection.**

  Assemble once from the repository projection, pass the typed result to the existing pure renderer, and keep CLI/MCP/agent delivery as adapters. Do not infer authorization from a green composition result.

- [ ] **Step 5: Add documentation and performance probes.**

  Document declarations, event/report/recovery semantics, runtime compatibility, unsupported topology, ordinary single-WI path, and exact commands in all three languages. The performance probe records cold/first/warm samples, raw order, machine/toolchain/runtime identity, file/Git/coordination/process counts, temporary disk, and explicit unavailable reasons; it does not claim benefit without a comparable baseline.

- [ ] **Step 6: Run integrated focused checks.**

  ```bash
  cargo fmt --all -- --check
  cargo test --locked -p cockpit-cli --test collaboration_cli -- --nocapture
  cargo test --locked -p cockpit-mcp --test collaboration_rpc -- --nocapture
  cargo test --locked -p cockpit-cli --test collaboration_outcome_parity -- --nocapture
  cargo test --locked -p cockpit-repository --test collaboration_admission -- --nocapture
  cargo test --locked -p cockpit-verification --test composition -- --nocapture
  python3 tests/performance/cross_wi_coordination_test.py
  python3 tests/ci/run_repository_gates.py --help
  ```

- [ ] **Step 7: Commit the projection/documentation layer.**

  ```bash
  git diff --check
  git add crates/cockpit-cli crates/cockpit-mcp crates/cockpit-repository/src/outcome_render.rs crates/cockpit-agent docs/reference/cross-wi-coordination* docs/features/cross-wi-coordination* tests/performance tests/ci/repository_gate_manifest.json
  git commit -m "feat: expose cross-WI coordination and composition outcomes"
  ```

### Task 5: Full verification, finish, archive, and PR/CI handoff

**Files:**
- Runtime-owned active Contract/Summary/evidence/archive/Outcome records for WI-1029.
- Existing repository gate outputs and verification receipts.

**Interfaces:**
- Consumes: all four implementation commits, real multi-worktree evidence, and current Runtime admission.
- Produces: a verified and archived Work Item, one pushed PR, and hosted CI handoff; no merge or release.

- [ ] **Step 1: Re-query Runtime before every lifecycle boundary.**

  Run `inspect`, `status`, `doctor`, and the relevant current command help. Stop if the Contract/snapshot is stale or the Runtime changes action admission.

- [ ] **Step 2: Run the declared focused and full checks.**

  ```bash
  cargo fmt --all -- --check
  cargo test --locked --workspace
  python3 tests/ci/run_repository_gates.py
  git diff --check
  ```

  Preserve any failure receipt and use the verification-failure route before retrying a failed path.

- [ ] **Step 3: Record verification through Runtime.**

  Use `ai-cockpit verify --repo <worktree> --work-item WI-1029-cross-wi-coordination-composition` with explicit commands and current candidate identity. Re-query the Work Item status and inspect the exact receipt.

- [ ] **Step 4: Finish and archive through Runtime.**

  Run `finish`, inspect the human Outcome, then `archive`. Do not hand-edit generated Contract, Summary, Outcome, archive, or receipt bytes.

- [ ] **Step 5: Commit the archive bundle.**

  Stage only the Runtime-generated records admitted by the current action explanation plus all implementation commits already present. Run `git diff --check` and commit the archive bundle with exact WI identity.

- [ ] **Step 6: Run final local governance checks and push the exact branch.**

  Run the current `check-ai-pr` equivalent discovered from installed capability/help, then push only `codex/wi-collaboration-closure` to `origin` if Runtime admits it. Never push `main` or mutate tags.

- [ ] **Step 7: Open one PR and hand off after hosted CI starts.**

  Bind the PR to the exact local Work Item head and Contract digest, attach the PR artifact, provide the hosted run link, and stop before merge/release. Do not auto-rerun CI, merge, publish, or create a release candidate.

## Plan Self-Review

- Protocol/storage, dependency/coordination, composition/reuse, and projection/docs each have an isolated commit boundary.
- Runtime skew, crash recovery, late events, safe pause, and conservative reuse are explicit review-focus tests.
- All 13 Contract acceptance criteria map to scenarios in the Contract and to Tasks 1-5.
- Ordinary single-WI compatibility is tested independently and is not routed through the coordination scan.
- No step hand-edits Runtime-generated governance records or grants remote authority.
- No incomplete implementation step remains; exact command names are discovered from the current Runtime before execution.
