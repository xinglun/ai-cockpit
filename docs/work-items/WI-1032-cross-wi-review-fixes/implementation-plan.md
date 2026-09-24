# WI-1032 Cross-WI Review Corrections Implementation Plan

> **For agentic workers:** Execute these tasks serially in this WI worktree. Every behavior fix follows test-first RED → GREEN. Keep each task in a separately reviewable commit.

**Goal:** Close every Important review finding against the admitted Runtime implementation while preserving safe composition reuse and cross-WI serial execution.

**Architecture:** Keep collaboration policy in the repository admission service, durable coordination semantics in the shared store, and candidate execution identity in the bounded composition runner. Derive trust from registered repository/Contract facts; when a read-set or check identity cannot be observed, fail closed or execute without reuse. Keep CLI/MCP schemas mechanically aligned with the protocol.

**Tech Stack:** Rust workspace, Cargo, Python process-acceptance harness, installed AI Cockpit Runtime CLI 0.2.113, Git linked worktrees.

**Spec:** `docs/work-items/WI-1032-cross-wi-review-fixes/spec.md`

## Global Constraints

- Use exactly one active Work Item, branch, worktree, and repository context at a time.
- Preserve WI-1031 archive, verification, recovery, and event bytes; continue only through the selected WI-1032 successor lineage.
- Keep Sentinel unchanged during WI-1032; its candidate CLI compatibility check is read-only. Actual Sentinel implementation is a later, separate, serial WI.
- Use installed lifecycle Runtime 0.2.113 and this candidate tree for collaboration behavior tests; do not claim old Runtime 0.2.105 enforces the new collaboration protocol.
- Use Runtime CLI plus Cargo/CI canonical gates. This repository has no supported Make entrypoint.
- Do not create or move release tags, prepare/publish a release, or publish public artifacts.
- Do not weaken the CI gate manifest or replace real process/worktree evidence with fixtures.

## Review Focus

1. An arbitrary command can read files beyond caller-declared `inputPaths`; stale success must never be reused when the complete read-set is not bound to trusted Contract facts.
2. A relative executable may resolve differently between Runtime CWD and the isolated composition worktree; identity must cover the executable actually launched.
3. Caller labels and a non-owner invocation must not satisfy registered required checks or start any process.
4. A killed lock owner must not leave a permanent lock or allow a retry to publish registration facts without its impact event.
5. Duplicate `outcomeId` values from different providers must remain distinct through CLI, MCP, projection, and recovery selection.

---

### Task 1: Contain outcome evidence beneath the registered worktree

**Files:**
- Modify: `crates/cockpit-repository/src/collaboration.rs`
- Modify if it reads the same references: `crates/cockpit-repository/src/coordination_store.rs`
- Test: `crates/cockpit-repository/tests/collaboration_admission.rs`
- Test if store-level validation changes: `crates/cockpit-repository/tests/coordination_store.rs`

**Interfaces:**
- Consumes: the registration's canonical worktree path and relative `ProvidedOutcome.evidence_refs`.
- Produces: one shared path-resolution rule that rejects absolute paths, parent traversal, final symlinks, and symlinked parent components before reading any evidence bytes.

- [ ] Add a real filesystem test with `.ai/evidence` (an intermediate path component) symlinked outside the registered worktree. Attempt publication and assert it fails and appends no event.
- [ ] Run `cargo test --locked -p cockpit-repository --test collaboration_admission outcome_publication_rejects_parent_symlink_escape`; expected: failure because the current final-component-only check accepts the outside file.
- [ ] Resolve each component without following an untrusted symlink; canonicalize and verify the final regular file remains beneath the canonical registered worktree. Reuse the same helper at every evidence read boundary in this scope.
- [ ] Re-run the focused test and the repository package tests; expected: the escape is rejected and existing valid in-tree evidence cases pass.
- [ ] Commit this storage-boundary change separately.

### Task 2: Recover coordination locking after a real process kill

**Files:**
- Modify: `crates/cockpit-repository/src/coordination_store.rs`
- Test: `crates/cockpit-repository/tests/coordination_store.rs` and/or a unit-test module in the same source file when private lock access is required.

**Interfaces:**
- Consumes: `CoordinationStore::register`, deterministic `auto-impact-<workItem>-<generation>` event identity, and existing atomic record writes.
- Produces: crash-released, cross-process exclusive locking whose retry is idempotent and which never removes a live owner's lock.

- [ ] Add a child-process test that pauses after the deterministic impact event is durable while the production store lock is held. Wait for a synchronization marker, kill the child process, then retry the same next-generation registration in a fresh process.
- [ ] Assert the retry succeeds, exactly one impact event exists, the new registration is authoritative only after reconciliation, and a competing live owner is never evicted.
- [ ] Run the new test before changing the lock implementation; expected: retry fails with `coordination lock did not become available` after the child is killed.
- [ ] Replace create-new/delete lockfile ownership with a portable OS-backed advisory lock held by an open file handle; retain fail-closed timeout behavior for a live owner and rely on OS lock release after process death. Do not delete/recreate a lock path while another process may hold its inode.
- [ ] Re-run the crash test plus `cargo test --locked -p cockpit-repository --test coordination_store`; expected: crash retry reconciles once and all store tests pass.
- [ ] Commit the lock recovery independently from Task 1.

### Task 3: Scope selected action dependencies by provider and outcome

**Files:**
- Modify: `crates/cockpit-protocol/src/lib.rs`
- Modify: `crates/cockpit-repository/src/collaboration.rs`
- Test: `crates/cockpit-repository/tests/collaboration_admission.rs`
- Update the CLI/MCP transport in Task 6 if the public action field must carry provider identity.

**Interfaces:**
- Consumes: `ConsumedOutcome.provider_work_item_id`, `ConsumedOutcome.outcome_id`, and explicit selected action dependencies.
- Produces: a typed provider/outcome key used for matching, invalidation filtering, and blocker calculation; a bare outcome ID cannot select or satisfy a dependency.

- [ ] Add two provider registrations that both publish `api`, and a consumer that declares both pairs. Select only `(WI-PROVIDER-A, api)` and assert only A's impact contributes a blocker; select B and assert A cannot cross-block it.
- [ ] Run `cargo test --locked -p cockpit-repository --test collaboration_admission action_dependency_selection_is_provider_scoped`; expected: the old outcome-id-only selection produces a cross-provider mismatch.
- [ ] Change the action selection type and all internal comparisons to the `(providerWorkItemId, outcomeId)` pair. Preserve conservative behavior when no selection is supplied: all declared dependencies remain relevant.
- [ ] Re-run focused pair tests and the repository package suite; expected: identical outcome strings remain isolated by provider.
- [ ] Commit dependency-key semantics separately.

### Task 4: Enforce integration ownership and authoritative required checks

**Files:**
- Modify: `crates/cockpit-repository/src/collaboration.rs`
- Modify protocol types only if a typed check identity is needed: `crates/cockpit-protocol/src/lib.rs`
- Test: `crates/cockpit-repository/tests/collaboration_admission.rs`

**Interfaces:**
- Consumes: the actual registered integration owner, digest-matched participant Contracts, Contract verification declarations, and the ordered composition command list.
- Produces: an admitted command set whose exact executable/argv identities cover every required registered check and whose caller is the declared integration owner.

- [ ] Add one test where a registered non-owner invokes composition with a command that writes a marker; assert admission rejects it and the marker does not exist.
- [ ] Add one test where `sh -c true` claims all required scenario/constraint labels while the registered Contract requires two different checks; assert missing/forged coverage is rejected before spawn. Also assert that providing only one of two exact required checks is rejected.
- [ ] Run the focused tests before the change; expected: the owner case reaches execution and the label-only command is accepted by the current validation.
- [ ] Derive required check identities from each real digest-matched participant Contract and bind labels to those identities. Compare the complete required set, not non-emptiness or caller labels. Reject missing, duplicate, extra-ambiguous, or unresolvable check identities before execution. Never treat arbitrary `CompositionPrecondition.satisfied` values as authority.
- [ ] Re-run focused tests and `cargo test --locked -p cockpit-repository --test collaboration_admission`; expected: no command starts on either invalid case and valid complete sets still execute.
- [ ] Commit admission and check-identity enforcement separately from node hashing.

### Task 5: Bind reuse to observed source, environment, and launched executable

**Files:**
- Modify: `crates/cockpit-verification/src/composition.rs`
- Test: `crates/cockpit-verification/tests/composition.rs`
- Extend real-process acceptance: `tests/acceptance/cross_wi_coordination_processes.py`

**Interfaces:**
- Consumes: the exact isolated composition worktree, complete trusted per-node read-set, actual inherited/overlaid environment, executable path resolved as the runner resolves it, and upstream execution receipts.
- Produces: a node reuse identity based only on Runtime-observed facts. If any relevant read-set or executable cannot be proven, the node executes and is not reused.

- [ ] Add a regression that runs the same composition JSON twice, changes an actual source input readable by the command without editing the JSON, and asserts the affected command is spawned again. Include an independent node and assert reuse only when its trusted read-set is unchanged.
- [ ] Add a regression for `./tools/check.sh` where the executable changes in the isolated composition worktree while the JSON remains identical; assert its identity changes and it runs again.
- [ ] Add/retain a real CLI process sequence: first run spawns one process, an unchanged second run spawns zero, and a third run with changed inherited environment but byte-identical composition JSON spawns the affected command.
- [ ] Run these tests against the current code before changing it; expected: an unlisted readable source change or relative executable replacement can incorrectly reuse a prior passing record.
- [ ] Resolve relative executables from the exact isolated worktree. Derive the complete reusable read-set from identity-bound registered Contract facts; when a command's reads cannot be bounded deterministically, mark its node identity unknown and execute rather than reuse. Hash current bytes and effective environment on every attempt, not caller-provided digest fields.
- [ ] Re-run focused composition tests and the real CLI acceptance; expected: exact unchanged inputs reuse, any changed observed input executes, and only independently proven unchanged nodes may be reused.
- [ ] Commit the runner and acceptance behavior in the composition/reuse layer.

### Task 6: Make MCP identity semantics unambiguous and protect them in the canonical gate

**Files:**
- Modify: `crates/cockpit-mcp/src/lib.rs`
- Update interface descriptions/reference output if generated from the source of truth: `crates/cockpit-interface/src/interface_description.rs`, `crates/cockpit-protocol/tests/interface_description.rs`, and associated reference files.
- Add or update the required Work Item projection pages: `docs/work-items/WI-1032-cross-wi-review-fixes.md`, `.zh-CN.md`, and `.ja.md`.
- Add the exact Work Item row to `docs/reference/reference-parity.md`, `.zh-CN.md`, and `.ja.md`.
- Test: `crates/cockpit-mcp/tests/collaboration_rpc.rs` and relevant interface-description tests.
- Modify `tests/ci/repository_gate_manifest.json` only if inspection shows the real process acceptance is not already registered.

**Interfaces:**
- Consumes: typed provider/consumer identities from Task 3 and the collaboration action's canonical MCP schema.
- Produces: exactly one meaning for every identity field per action, with CLI/MCP parity and gate-manifest coverage.

- [ ] Add a schema regression that asserts the serialized `work_item_coordination` input object has no duplicate property and that `publish-outcome`, action selection, and recovery expose the correct provider/consumer identity fields.
- [ ] Run `cargo test --locked -p cockpit-mcp --test collaboration_rpc` before the change; expected: the schema reveals the duplicate `workItemId` and its overwritten description.
- [ ] Define action-specific identity fields once in the canonical interface description, generate MCP properties from them, and update CLI/MCP parity assertions. Preserve provider identity for publish and consumer/provider tuples for actions that select dependencies.
- [ ] Verify `tests/acceptance/cross_wi_coordination_processes.py` is named in the canonical gate manifest; register it and add manifest coverage only if absent.
- [ ] Update the collaboration reference/docs in all maintained locales only where the public field semantics changed, then run the documentation and interface-generation checks.
- [ ] Runtime currently rejects verification until the three Work Item projection pages and parity rows exist. Keep their claims explicitly in-progress until evidence is available; create/update them before Runtime-bound verification, then refresh the outcome projection after verification without claiming unproven benefits.
- [ ] Run focused MCP/interface tests and the manifest test; expected: schema has unambiguous fields and the real process test remains a required gate.
- [ ] Commit schema, parity tests, and docs as the projection/documentation layer.

### Task 7: Prove query and durable-write boundaries

**Files:**
- Test: `crates/cockpit-cli/tests/collaboration_cli.rs` and relevant status/outcome query tests.
- Test: `crates/cockpit-mcp/tests/collaboration_rpc.rs`.
- Test: `crates/cockpit-repository/tests/coordination_store.rs` where store-open behavior needs direct coverage.
- Update the collaboration reference only if the public query/write distinction is unclear after the tests.

**Interfaces:**
- Consumes: read-only `inspect` and projection routes, explicit durable write actions, and request-scoped observation ledgers.
- Produces: regression evidence that reads never mutate durable coordination state and that request-local consistency is not described as cross-process event transport.

- [ ] Add subprocess-backed CLI and MCP tests that snapshot the coordination directory before and after `inspect`/projection queries, including an absent store; assert no file is created, rewritten, or consumed.
- [ ] Assert each durable mutation remains behind its explicit action and that a fresh process observes only records actually persisted by those actions; request-scoped ledger state alone must not appear as a delivered event.
- [ ] Run the focused tests before any change; retain existing correct query boundaries and repair only a demonstrated mutation or misleading projection/documentation.
- [ ] Re-run CLI/MCP/repository focused suites and the real multi-process acceptance; keep query assertions separate from writes so an inspect call cannot mask a missing explicit report or recovery.
- [ ] Commit the query/write-boundary proof with the final projection/documentation layer.

## Final verification and handoff

Run the Contract-declared focused tests during each task, then all of:

```sh
cargo fmt --all -- --check
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
python3 tests/ci/repository_gate_manifest_test.py
cargo build --locked --release -p cockpit-cli --bin ai-cockpit
python3 tests/acceptance/cross_wi_coordination_processes.py --binary target/release/ai-cockpit
python3 scripts/documentation_acceptance.sh
```

Run the read-only candidate CLI compatibility check against Sentinel and compare its repository tree, Contract/evidence state, lifecycle Runtime, and coordination store before and after. Then follow Runtime verification, finish, archive, PR review, hosted checks, merge, and exact cleanup. Do not enter release preparation or publish.
