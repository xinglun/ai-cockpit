# Cross-Work-Item Behavioral Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the seven independent acceptance blockers in the existing cross-WI collaboration implementation without weakening the fixed Runtime boundary, ordinary single-WI serial behavior, or evidence preservation.

**Architecture:** Keep the existing protocol and repository-local coordination store, but make every mutating path validate observed Git/Contract/runtime facts. Route concrete action admission through one refresh service, adapt composition nodes to the existing bounded verification executor, persist shared attempts in the common directory, and project those facts into the existing CLI/MCP/Outcome surfaces.

**Tech Stack:** Rust 2024 workspace, `serde`/`serde_json`, existing `cockpit-git`, `cockpit-repository`, `cockpit-verification`, `cockpit-cli`, `cockpit-mcp`, Python acceptance/gate scripts, real Git linked worktrees, fixed Runtime `0.2.105`.

**Spec:** `docs/superpowers/specs/2026-09-24-cross-wi-behavioral-closure-design.md`

## Global Constraints

- Keep one active Work Item, branch, linked worktree, and repository context: `WI-1030-cross-wi-behavioral-closure` on `codex/wi-collaboration-behavioral-closure`.
- Use Runtime `0.2.105` only for lifecycle responsibility; candidate Runtime capability is required for candidate collaboration actions. Do not claim bidirectional compatibility.
- Validate repository/common directory, Contract/digest, linked-worktree topology, branch, head, runtime binding, generation, and required evidence from observed facts before writes or action admission.
- Keep reads side-effect free. Registration, report, coordination, recovery, and composition are explicit writes/actions.
- `SafelyPaused` rejects the concrete target action before any process spawn; unrelated Work Items remain runnable; resume refreshes facts.
- Reuse the existing bounded executor; do not add a second unbounded `Command::output()` loop.
- Preserve immutable failed/interrupted evidence and make cleanup results explicit.
- Do not release, tag, publish, merge, mutate Provider resources, or modify global Agent/MCP configuration.
- The repository has no Makefile. The canonical verification route is Runtime CLI plus Cargo/CI gates; only absence of both would be fail-closed.

## Review Focus

- A forged registration must fail before persistence even when its fields are well-formed; owned by Task 1 tests.
- A safe pause must stop the concrete composition process, not merely change a state enum; owned by Task 2 process-sentinel test.
- An old generation must not release new-generation resources or deadlock recovery; owned by Task 1/2 generation tests.
- Empty or incomplete composition must start zero project processes; owned by Task 3 precondition test.
- A second identical request must reduce process count, while a partial identity change reruns only affected nodes; owned by Task 3 CLI/MCP integration test.
- An Outcome publication must not become an invalidation, and pending coordination must not fabricate human authority; owned by Task 4 projection tests.

## File Map

- Modify `crates/cockpit-protocol/src/lib.rs` only for additive identity, action, recovery, and projection fields required by observed-fact validation.
- Modify `crates/cockpit-repository/src/coordination_store.rs` for registration/inspection validation, generation-safe resources, event classification, and append-only recovery.
- Modify `crates/cockpit-repository/src/collaboration.rs` for concrete action admission, pause interception, evidence consumption, and refresh-before-action.
- Modify `crates/cockpit-verification/src/composition.rs` and `crates/cockpit-verification/src/lib.rs` to use the existing bounded executor and common-directory attempt storage.
- Modify `crates/cockpit-repository/src/outcome_render.rs` and the CLI/MCP adapters for shared projection parity.
- Add focused Rust tests beside each owning crate and extend `tests/acceptance/cross_wi_coordination_processes.py` with real linked-worktree process assertions.
- Modify `tests/ci/repository_gate_manifest.json` and its manifest regression test only to register the acceptance script as a canonical gate.
- Update `docs/reference/cross-wi-coordination.{md,zh-CN.md,ja.md}` and `docs/features/cross-wi-coordination.{md,zh-CN.md,ja.md}` with the runtime boundary, action semantics, recovery, and serial single-WI path.

---

### Task 0: Bind the corrective spec and Runtime admission

**Files:**
- Create: `docs/superpowers/specs/2026-09-24-cross-wi-behavioral-closure-design.md`
- Create: `docs/superpowers/plans/2026-09-24-cross-wi-behavioral-closure.md`
- Runtime-managed: `.ai/work-items/active/WI-1030-cross-wi-behavioral-closure.*`

**Interfaces:**
- Consumes: merged PR #993 at `ab1e4be1be766ffac1dea86a3fbec9b8cbd87160`, independent red acceptance review, fixed Runtime identity.
- Produces: an additive Contract amendment with the 9 required scenarios and a current non-red Runtime decision before source changes.

- [x] **Step 1: Create the dedicated branch and linked worktree.**

  Confirm the worktree is attached to `codex/wi-collaboration-behavioral-closure`, starts at `669937a43f680c5ef759b511d30d2a17aceb56e6`, and has no unrelated edits.

- [x] **Step 2: Create the authorized corrective WI through Runtime.**

  Use the repository-bound CLI with `--repo`, `--authority authorized`, and the four evidence classes. Do not hand-edit `.ai` records.

- [x] **Step 3: Append the independent review source and typed scenario coverage before checkpoint.**

  Use `work-item amend` with `sourcesAppend` and `scenarioCoverageAppend`, then confirm the Contract digest changes through the Runtime result.

- [x] **Step 4: Write the corrective spec and implementation plan.**

  Keep the seven blockers, fixed Runtime/candidate boundary, serial single-WI behavior, and pre-release stopping rule explicit.

- [ ] **Step 5: Run Runtime discovery and preflight after the plan is present.**

  Run:

  ```bash
  ai-cockpit inspect --repo "$PWD"
  ai-cockpit status --repo "$PWD"
  ai-cockpit doctor --repo "$PWD"
  ai-cockpit agent doctor --repo "$PWD" --json
  ai-cockpit preflight --repo "$PWD" --contract .ai/work-items/active/WI-1030-cross-wi-behavioral-closure.contract.json
  ```

  Expected: Runtime reports the current Contract and snapshot, no contradictory blocker, and an action explanation for implementation. Preserve any yellow/unknown evidence and stop if the Runtime requires a human decision not already covered by the explicit authorization.

---

### Task 1: Bind coordination writes to observed repository facts

**Files:**
- Modify: `crates/cockpit-protocol/src/lib.rs`
- Modify: `crates/cockpit-repository/src/coordination_store.rs`
- Modify: `crates/cockpit-repository/src/lib.rs`
- Test: `crates/cockpit-repository/tests/coordination_store.rs`
- Test: `crates/cockpit-repository/tests/collaboration_admission.rs`

**Interfaces:**
- Consumes: `GitRepository::topology`, repository `repository_id`, active Contract loader/digest, Runtime capability identity, existing `WorktreeRegistration`, `ConsumedOutcome`, reservation and event types.
- Produces: one validation path used by `register`, `inspect`, `reserve_resources`, `release_resources`, and event/recovery writes; mismatches return typed fail-closed diagnostics before persistence.

- [ ] **Step 1: Write failing identity-binding tests.**

  Build a real temporary repository with an active Contract and two linked worktrees. For each forged input, call `register` and assert the common-directory record count is unchanged:

  ```rust
  for forged in [wrong_repository, wrong_common_dir, wrong_contract_digest,
                 wrong_worktree_path, wrong_branch, wrong_head,
                 missing_required_evidence, wrong_runtime_capability] {
      let error = store.register(forged).expect_err("forged identity must fail");
      assert!(error.is_identity_or_evidence_failure());
      assert!(store.inspect().unwrap().registrations.is_empty());
  }
  ```

- [ ] **Step 2: Run the focused test and observe failure.**

  ```bash
  cargo test --locked -p cockpit-repository --test coordination_store registration_identity -- --nocapture
  ```

  Expected: at least one forged registration is currently persisted because validation only checks field shape/runtime format.

- [ ] **Step 3: Implement observed-fact validation.**

  Resolve canonical common directory and repository identity through Git, load the active Contract for the declared WI, compute its digest using the repository lifecycle helper, and compare actual topology (`worktree_path`, branch, head, common directory). Verify `verification_required` consumers against current evidence. Perform all checks before taking the write lock or writing a registration.

- [ ] **Step 4: Write failing inspection and resource-generation tests.**

  Move/delete a registered worktree and advance the registration generation. Assert `inspect` returns unknown/recovery-required, old `release_resources` fails and leaves the reservation, and a current-generation release succeeds only after current identity validation.

- [ ] **Step 5: Implement generation-safe resource operations.**

  Require the current registration for reserve/release, compare repository and WI identity plus generation, and keep stale reservations visible. Do not use a reservation's self-reported generation as proof of current ownership.

- [ ] **Step 6: Add event-kind and recovery identity types.**

  Preserve `OutcomePublished` as non-invalidating. Add an append-only resolution relation that references the predecessor event and current provider generation/facts; do not require the provider to remain at the predecessor generation.

- [ ] **Step 7: Implement append-only cross-generation recovery.**

  Change `recover_event` to validate the consumer's current generation and the provider's current observed identity, while allowing the predecessor event's generation to be old. Keep the old event in `inspect` output and reject duplicate/forged resolutions.

- [ ] **Step 8: Run the storage and protocol tests.**

  ```bash
  cargo fmt --all -- --check
  cargo test --locked -p cockpit-protocol
  cargo test --locked -p cockpit-repository --test coordination_store -- --nocapture
  ```

- [ ] **Step 9: Commit the protocol/storage layer.**

  ```bash
  git diff --check
  git add crates/cockpit-protocol crates/cockpit-repository/src/coordination_store.rs crates/cockpit-repository/src/lib.rs crates/cockpit-repository/tests/coordination_store.rs crates/cockpit-repository/tests/collaboration_admission.rs
  git commit -m "fix: bind collaboration state to observed repository facts"
  ```

---

### Task 2: Make dependency admission and safe pause executable

**Files:**
- Modify: `crates/cockpit-repository/src/collaboration.rs`
- Modify: `crates/cockpit-repository/src/coordination_store.rs`
- Modify: `crates/cockpit-protocol/src/lib.rs` if the concrete action identity is not already representable
- Test: `crates/cockpit-repository/tests/collaboration_admission.rs`
- Test: `crates/cockpit-repository/tests/coordination_store.rs`

**Interfaces:**
- Consumes: Task 1 observed-fact validator, current request state machine, current event projection, and `verification_required` evidence checks.
- Produces: `admit_collaboration_action(repository, work_item, generation, action, consumer_identity, runtime)` plus refresh-before-action semantics used by composition and CLI/MCP.

- [ ] **Step 1: Write the failing pause-before-spawn test.**

  Register two real linked worktrees. Request, acknowledge, and safely pause WI-A. Invoke the concrete composition action with a marker command and assert the marker does not appear; invoke the unrelated WI-B action and assert it remains admissible. The test must assert the action is rejected while `SafelyPaused`, not only after resume.

  ```rust
  assert!(matches!(admit_collaboration_action(&a, Action::Compose, ...), Err(ActionDenied::SafelyPaused { .. })));
  assert!(!marker.exists());
  assert!(admit_collaboration_action(&b, Action::Compose, ...).is_ok());
  ```

- [ ] **Step 2: Run the focused test and observe failure.**

  ```bash
  cargo test --locked -p cockpit-repository --test collaboration_admission safely_paused -- --nocapture
  ```

  Expected: the current implementation admits WI-A because it checks only registrations and projection blockers.

- [ ] **Step 3: Add the concrete action and consumer/result identity.**

  Keep ordinary single-WI calls serial and declaration-free. For collaboration-aware actions, require the action kind, target generation, and affected consumer/result identity so an unrelated action can continue without bypassing an affected edge.

- [ ] **Step 4: Implement refresh-before-admission.**

  Re-observe Git topology, Contract digest, current registration, evidence required by consumed outcomes, resources, invalidation events, and request state. Reject stale/unknown facts; do not repair or consume them from a read.

- [ ] **Step 5: Enforce pause and resume transitions.**

  Intercept `SafelyPaused` before `run_admitted_composition` can invoke verification. Bind requests to WI and execution generation, reject expired/duplicate old requests, and make resume append a transition followed by a fresh admission check.

- [ ] **Step 6: Separate impact from publication.**

  Ensure `OutcomePublished` does not create a consumer blocker. Only identity-changing event kinds create affected edges; unrelated WIs continue. Add a test that publishes an outcome, then reports a head/Contract invalidation and observes only the dependent action blocked.

- [ ] **Step 7: Run admission and real-process tests.**

  ```bash
  cargo test --locked -p cockpit-repository --test collaboration_admission -- --nocapture
  cargo test --locked -p cockpit-repository --test coordination_store -- --nocapture
  ```

- [ ] **Step 8: Commit the admission/control layer.**

  ```bash
  git diff --check
  git add crates/cockpit-repository/src/collaboration.rs crates/cockpit-repository/src/coordination_store.rs crates/cockpit-protocol/src/lib.rs crates/cockpit-repository/tests/collaboration_admission.rs crates/cockpit-repository/tests/coordination_store.rs
  git commit -m "fix: enforce collaboration admission and safe pause"
  ```

---

### Task 3: Reuse the bounded executor and make composition reuse real

**Files:**
- Modify: `crates/cockpit-verification/src/composition.rs`
- Modify: `crates/cockpit-verification/src/lib.rs`
- Modify: `crates/cockpit-repository/src/collaboration.rs`
- Test: `crates/cockpit-verification/tests/composition.rs`
- Test: `crates/cockpit-repository/tests/collaboration_admission.rs`

**Interfaces:**
- Consumes: Task 1/2 identity and admission services, `VerificationCommand`, `execute_bounded_at`, existing reuse/evidence types, and Git temporary-worktree helpers.
- Produces: composition attempts stored under the shared common-directory coordination area, one durable node record per execution, and process-count-visible `Reuse`/`Execute` decisions.

- [ ] **Step 1: Write the failing precondition tests.**

  Add tests for empty commands, a missing required check, forged `satisfied: true`, paused admission, and mismatched participant identity. Assert the command marker count is zero and the attempt is a readable failure.

- [ ] **Step 2: Run the precondition tests and observe failure.**

  ```bash
  cargo test --locked -p cockpit-verification --test composition empty_commands -- --nocapture
  cargo test --locked -p cockpit-verification --test composition required_checks -- --nocapture
  ```

  Expected: the current empty command list can be marked passed and caller-supplied preconditions bypass Runtime calculation.

- [ ] **Step 3: Implement Runtime-built composition identity and coverage.**

  Construct participant WI/head/Contract/declaration identities from validated registrations. Derive required checks from the Contract/declaration, reject empty or incomplete coverage, and ignore forged caller booleans. Call the concrete admission service before creating the temporary composition worktree.

- [ ] **Step 4: Write failing timeout/interruption/cleanup tests.**

  Run a sleeping node with a bounded timeout and a node interrupted at a safe boundary. Assert the persisted node record includes `timed_out`, interruption/unknown-exit state, bounded stdout/stderr, and cleanup success/failure. Verify the record exists before the next node or terminal error.

- [ ] **Step 5: Adapt nodes to the existing bounded executor.**

  Convert each composition node to `VerificationCommand` and call the existing bounded execution API with its timeout, current directory, dependency identity, and reuse candidate. Do not call `Command::output()` from composition. Persist the returned node record immediately in the shared store before proceeding.

- [ ] **Step 6: Write failing actual-reuse tests.**

  Execute the same exact composition twice through the action path and compare a process sentinel/counter. Then change one node's identity and assert only that node and its transitive dependents spawn. Keep the old receipt immutable and reference it from the new attempt.

  ```rust
  let first = run_composition(request.clone())?;
  let second = run_composition(request.clone())?;
  assert!(second.processes_spawned < first.processes_spawned);
  assert!(second.nodes.iter().all(|node| node.reused || node.ran));
  ```

- [ ] **Step 7: Connect `classify_reuse` to execution.**

  Load the previous shared attempt, compare source/transitive dependency/interface/config/toolchain/lockfile/generated-input/environment/verifier identities, and pass only exact candidates to the executor. Record every node's decision and predecessor receipt reference.

- [ ] **Step 8: Persist explicit cleanup results.**

  Make temporary worktree cleanup return a typed result; record it in the attempt even when cleanup fails. Do not discard cleanup errors or convert an unknown process result into success.

- [ ] **Step 9: Run focused composition tests.**

  ```bash
  cargo fmt --all -- --check
  cargo test --locked -p cockpit-verification --test composition -- --nocapture
  cargo test --locked -p cockpit-repository --test collaboration_admission -- --nocapture
  ```

- [ ] **Step 10: Commit the execution/reuse layer.**

  ```bash
  git diff --check
  git add crates/cockpit-verification crates/cockpit-repository/src/collaboration.rs crates/cockpit-verification/tests/composition.rs crates/cockpit-repository/tests/collaboration_admission.rs
  git commit -m "fix: persist bounded composition attempts and reuse nodes"
  ```

---

### Task 4: Project real facts, add the canonical multi-process gate, and document the boundary

**Files:**
- Modify: `crates/cockpit-repository/src/outcome_render.rs`
- Modify: `crates/cockpit-agent/src/lib.rs`
- Modify: `crates/cockpit-cli/src/main.rs`
- Modify: `crates/cockpit-mcp/src/lib.rs`
- Test: `crates/cockpit-cli/tests/collaboration_cli.rs`
- Test: `crates/cockpit-mcp/tests/collaboration_rpc.rs`
- Test: `crates/cockpit-cli/tests/collaboration_outcome_parity.rs`
- Modify: `tests/acceptance/cross_wi_coordination_processes.py`
- Modify: `tests/ci/repository_gate_manifest.json`
- Test: `tests/ci/repository_gate_manifest_test.py`
- Modify: `docs/reference/cross-wi-coordination.md`, `docs/reference/cross-wi-coordination.zh-CN.md`, `docs/reference/cross-wi-coordination.ja.md`
- Modify: `docs/features/cross-wi-coordination.md`, `docs/features/cross-wi-coordination.zh-CN.md`, `docs/features/cross-wi-coordination.ja.md`

**Interfaces:**
- Consumes: Task 2 admission projection and Task 3 shared composition attempts/reuse facts.
- Produces: CLI/MCP parity, real shared Outcome fields, canonical gate coverage, and explicit single-WI/runtime compatibility documentation.

- [ ] **Step 1: Write failing projection tests.**

  Record composition, merge-target, cleanup, and per-node reuse facts through explicit writes. Assert summary/full Outcome has those facts, distinguishes unknown from false, and leaves `human_decision_required` false for a pending coordination request without a human-decision boundary.

- [ ] **Step 2: Implement shared Outcome projection.**

  Read the common-directory attempt store and coordination projection through the same repository service used by CLI/MCP. Remove fixed `not_observed`/empty placeholders when facts exist, preserve unknown when facts do not, and keep human rendering pure.

- [ ] **Step 3: Write CLI/MCP action parity tests.**

  Assert `inspect` and status do not write or consume records. Assert register/report/request/acknowledge/resume/recover/composition return the same typed result through CLI and MCP, and both reject fixed `0.2.105` for candidate-only actions before mutation.

- [ ] **Step 4: Implement adapters without domain duplication.**

  Keep CLI/MCP argument parsing thin; call the repository service with the concrete action identity and candidate Runtime binding. Preserve ordinary serial single-WI execution when the collaboration declaration is absent.

- [ ] **Step 5: Strengthen the real acceptance script.**

  Replace fixed fake repository/Contract/head identities with facts read from the temporary Git repository and actual active Contracts. Run concurrent registration/report processes, safe-pause sentinel execution, old-generation resource/recovery cases, and two composition executions with process-count assertions across at least two linked worktrees.

- [ ] **Step 6: Register the acceptance script in the canonical gate manifest.**

  Use the prebuilt CI binary path already produced by `.github/workflows/ci.yml`:

  ```json
  {"category":"conformance","command":["python3","tests/acceptance/cross_wi_coordination_processes.py","--binary","target/release/ai-cockpit"],"dependsOn":["docs_governance_integrity"],"id":"conformance_cross_wi_coordination_processes","minimumProfile":"standard"}
  ```

  Keep gate IDs sorted and add a manifest regression assertion that the exact acceptance command is present.

- [ ] **Step 7: Update all three language documents.**

  Document observed identity binding, event publication versus invalidation, cross-generation recovery, safe pause before spawn, bounded executor/reuse, read/write separation, Runtime `0.2.105` compatibility, candidate capability, unsupported topology, and the normal single-WI serial route.

- [ ] **Step 8: Run projection, acceptance, and gate tests.**

  ```bash
  cargo test --locked -p cockpit-cli --test collaboration_cli -- --nocapture
  cargo test --locked -p cockpit-mcp --test collaboration_rpc -- --nocapture
  cargo test --locked -p cockpit-cli --test collaboration_outcome_parity -- --nocapture
  python3 tests/ci/repository_gate_manifest_test.py
  python3 tests/acceptance/cross_wi_coordination_processes.py --binary target/release/ai-cockpit
  ```

- [ ] **Step 9: Commit projection, gate, and documentation.**

  ```bash
  git diff --check
  git add crates/cockpit-repository/src/outcome_render.rs crates/cockpit-agent/src/lib.rs crates/cockpit-cli crates/cockpit-mcp tests/acceptance/cross_wi_coordination_processes.py tests/ci/repository_gate_manifest.json tests/ci/repository_gate_manifest_test.py docs/reference/cross-wi-coordination* docs/features/cross-wi-coordination*
  git commit -m "fix: project executable collaboration facts and acceptance gate"
  ```

---

### Task 5: Fresh verification, Runtime finish, and pre-release handoff

**Files:**
- Runtime-managed: `.ai/work-items/active/WI-1030-cross-wi-behavioral-closure.*`, `.ai/evidence/*`, `.ai/decisions/*`
- Generated verification output: `target/repository-gates.json`, `target/quality-route.json`, and gate receipts

**Interfaces:**
- Consumes: four implementation commits, real multi-process evidence, current candidate binary, fixed Runtime lifecycle state.
- Produces: independently reviewable PR handoff before release; no tag, release, merge, or publication claim.

- [ ] **Step 1: Re-query Runtime and repository identity.**

  Run the four discovery commands, inspect the active Contract/Summary, verify branch/worktree/common directory, and confirm no unrelated changes.

- [ ] **Step 2: Run the canonical repository route.**

  Because no Makefile exists, use the Runtime CLI plus:

  ```bash
  cargo fmt --all -- --check
  cargo test --locked --workspace
  cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
  python3 tests/ci/run_repository_gates.py --repo "$PWD" --manifest tests/ci/repository_gate_manifest.json --profile standard --report target/repository-gates.json
  ```

  Use a release-built candidate binary for the acceptance gate, and preserve every failure/unknown receipt rather than rerunning blindly.

- [ ] **Step 3: Record Runtime verification and inspect the Outcome.**

  Record the declared verification evidence through the repository-bound Runtime entrypoint, re-query status, and verify that every high-risk scenario has concrete evidence. Do not mark reuse, cleanup, merge, or host display true from a log line alone.

- [ ] **Step 4: Finish/archive only if Runtime admits the action.**

  Use Runtime `finish` and `archive` with current evidence. Do not hand-edit generated records. If a failure is found, follow the failure-recovery route and keep the red/yellow evidence visible.

- [ ] **Step 5: Prepare the review handoff.**

  Re-check the four ordered commit boundaries, run `git diff --check`, inspect the exact PR diff/CI status, and report separately: implementation, verification, host/PR state, release state, unknowns, and the explicit requirement for independent review before any release.

## Plan self-review

- Spec coverage: identity binding and resources are Task 1; admission/pause/publication/recovery are Task 2; bounded execution and actual reuse are Task 3; projection/CLI/MCP/gate/docs are Task 4; Runtime evidence and pre-release handoff are Task 5.
- Placeholder scan: no task relies on a future unspecified command or a caller-provided success boolean; every task names files, tests, commands, and expected outcomes.
- Type consistency: the concrete action admission service is introduced in Task 2 and consumed by composition in Task 3 and adapters in Task 4; shared attempts are introduced in Task 3 and consumed by Outcome in Task 4.
- Review focus coverage: each failure mode listed above has a named failing test and owning task.
