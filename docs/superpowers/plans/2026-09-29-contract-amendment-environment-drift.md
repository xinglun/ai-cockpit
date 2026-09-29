# Contract Amendment and Environment Drift Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let owners safely revise any human-authored active Contract plan and let Runtime-observed environment changes propagate across linked worktrees without silently admitting stale or unrelated work.

**Architecture:** Define a typed amendment request and explicit field mutability registry in `cockpit-protocol`, then apply it in a recoverable, digest-checked repository transaction with an append-only journal. Extend the existing common-directory coordination store and action-admission path for environment drift; provide CLI/MCP parity, real-process coverage, and multilingual guidance without parallelizing the implementation.

**Tech Stack:** Rust workspace, serde/serde_json, existing Git and Runtime identity types, existing inter-process coordination lock/store, clap CLI, MCP JSON schemas, Cargo and Runtime CI gates.

**Spec:** [2026-09-29-contract-amendment-environment-drift-design.md](../specs/2026-09-29-contract-amendment-environment-drift-design.md)

## Global Constraints

- A mutation requires a non-empty reason, unique `changeId`, and exact `expectedContractDigest`; it never silently rebases a conflict.
- Unknown or protected Contract paths fail closed; policy-sensitive edits re-run authorization/review and verification admission.
- Amendment history and environment-drift events are append-only; request-scoped observation state is not cross-process persistence.
- Persist shared collaboration state under the existing Git common-directory coordination root and its inter-process lock.
- Preserve the ordinary one-Work-Item serial path and default one-worker verification; parallel execution is not used for implementation.
- Run the actual process/worktree acceptance as a registered canonical gate; no release/tag mutation or Task 9 migration is in scope.

## Review Focus

- Two writers amend the same Contract digest: only one commits; the other gets a conflict, while an exact `changeId` retry returns the original receipt.
- A batch clears a required field or weakens a gate: the whole batch is rejected or routed to the existing review boundary, with no partial Contract write.
- A process stops between journal preparation, Contract replacement, projection update, and commit marker: readers recover the exact transaction or deny action; no duplicate/lost amendment results.
- Environment variables/toolchain change while submitted Contract JSON remains unchanged: a real child Runtime observes and persists drift, denies only affected actions before side effects, and lets unrelated outcomes proceed.
- An old Runtime can parse a new Contract but lacks the journal/drift capability: compatibility/admission denies affected work rather than ignoring the constraint.

---

### Task 1: Typed amendment protocol and field registry

**Files:**
- Modify: `crates/cockpit-protocol/src/lib.rs`
- Create: `crates/cockpit-repository/src/contract_amendment.rs`
- Modify: `crates/cockpit-repository/src/lib.rs`
- Modify: `crates/cockpit-repository/src/lifecycle.rs`
- Test: `crates/cockpit-repository/tests/contract_amendment.rs`
- Test: `crates/cockpit-protocol/tests/contract_amendment.rs`

**Interfaces:**
- Produces `ContractAmendmentRequest { schema_version: u32, change_id: String, expected_contract_digest: Digest, reason: String, changes: Vec<ContractAmendmentChange> }`.
- Produces `ContractAmendmentChange { path: String, operation: ContractAmendmentOperation, value: Option<serde_json::Value> }` and operations `Add`, `Set`, `Clear`, `Remove`, `Replace`, `Reorder`.
- Produces `apply_contract_amendment(contract: &Contract, request: &ContractAmendmentRequest) -> Result<Contract, Vec<ContractAmendmentError>>`.
- `path` is a canonical JSON Pointer restricted by an explicit schema registry. `add` appends one valid collection element; `set` assigns an existing typed field; `clear` removes only optional/clearable values; `remove` removes an exact collection element; `replace` changes an existing value; `reorder` requires an exact permutation keyed by that field's typed stable identity.

- [ ] First add `typed_request_replaces_goal_without_rewriting_identity` in the repository integration test. It calls the existing amendment API with a JSON `changes` request and expects the active Contract's goal to be replaced.
- [ ] Run `cargo test --locked -p cockpit-repository --test contract_amendment typed_request_replaces_goal_without_rewriting_identity`; expected initial failure is the current `unsupported Contract amendment field changes` result.
- [ ] Implement the typed request, explicit field classification, operation semantics, and full Contract re-deserialization/validation in `cockpit-protocol`; add protocol tests for scalar, optional, nested, collection, batch-order, and protected-path behavior.
- [ ] Implement the vertical-slice repository adapter in `contract_amendment.rs` and route both typed requests and legacy append input through it.
- [ ] Run `cargo test --locked -p cockpit-protocol --test contract_amendment` and `cargo test --locked -p cockpit-repository --test contract_amendment`; expected: all typed-operation and public API cases pass.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Commit protocol and first amendment path as `feat(protocol): define reasoned contract amendment requests`.

### Task 2: Recoverable amendment persistence and audit

**Files:**
- Create: `crates/cockpit-repository/src/contract_amendment.rs`
- Modify: `crates/cockpit-repository/src/lib.rs`
- Modify: `crates/cockpit-repository/src/lifecycle.rs`
- Test: `crates/cockpit-repository/tests/contract_amendment.rs`

**Interfaces:**
- Produces `apply_work_item_contract_amendment(root: &Path, work_item_id: &str, request: &ContractAmendmentRequest, runtime: &RuntimeContext) -> Result<ContractAmendmentReceipt, ObserverError>`.
- Produces `read_work_item_contract_amendments(root: &Path, work_item_id: &str) -> Result<Vec<ContractAmendmentReceipt>, ObserverError>`; this read does not write or consume records.
- Existing additive `amend_work_item_contract` remains a compatibility wrapper that translates legacy append input plus `--reason` into the same transaction path.

- [ ] Add repository tests for exact-digest conflict, `changeId` idempotency, audit-chain validation, invalidated-check listing, and sensitive-field policy evaluation; expect them to fail against the Task 1 vertical slice.
- [ ] Add crash-injection tests at each write boundary: prepared journal, Contract replace, Summary projection, and commit marker; assert atomic visibility or fail-closed recovery.
- [ ] Run `cargo test --locked -p cockpit-repository --test contract_amendment`; expected: new transaction/audit assertions fail before persistent transaction support.
- [ ] Implement per-Work-Item locking, expected-digest comparison, prospective Contract validation, recoverable transaction states, append-only old/new audit entries, and verification/projection invalidation.
- [ ] Implement the read-only amendment-history query and archive preservation validation without rewriting archived bytes.
- [ ] Run `cargo test --locked -p cockpit-repository --test contract_amendment`; expected: recovery, conflict, and audit-chain cases pass.
- [ ] Commit repository amendment persistence as `feat(runtime): persist contract amendments atomically`.

### Task 3: Runtime-observed environment drift and selective admission

**Files:**
- Modify: `crates/cockpit-protocol/src/lib.rs`
- Modify: `crates/cockpit-repository/src/execution_context.rs`
- Modify: `crates/cockpit-repository/src/coordination_store.rs`
- Modify: `crates/cockpit-repository/src/collaboration.rs`
- Test: `crates/cockpit-repository/tests/environment_drift.rs`
- Test: `crates/cockpit-repository/tests/collaboration_admission.rs`

**Interfaces:**
- Produces `EnvironmentDriftRequest { work_item_id: String, expected_generation: u64 }`; it carries no caller-supplied environment digest.
- Produces a Runtime-created `EnvironmentDriftBinding` containing repository/common-directory identity, provider generation, prior/current observed environment digests, observed input identities, affected outcome IDs, and event identity.
- Extends `CoordinationEvent` with an optional versioned environment binding; legacy event semantics remain unchanged and unsupported versions fail closed.
- Extends `CoordinationStore` with `record_environment_drift(request, observed_context) -> Result<CoordinationEvent, CoordinationError>` and makes action admission refresh this durable state immediately before evaluating the selected outcomes.

- [ ] First add a protocol test that deserializes an event carrying a versioned `environmentChange` binding and rejects caller-supplied digest data in `EnvironmentDriftRequest`; run it and confirm the current strict coordination schema rejects the unknown binding.
- [ ] Add repository tests proving event IDs are idempotent, outcome publication does not invalidate, and recovery appends a resolution across provider generations.
- [ ] Run the focused protocol/repository tests; expected initial failure is absence of the typed environment binding/Runtime observer.
- [ ] Add tests proving stale affected generation/receipt is denied before execution/reservation, unrelated outcomes in the same Work Item remain admissible, and `SafelyPaused` continues to deny through the shared admission result.
- [ ] Implement runtime-derived environment identity from the current execution context; if a relevant input cannot be observed, mark it unknown and deny reuse that depends on it.
- [ ] Persist drift events under the existing common-directory store and inter-process lock; make registration refresh/event commit recoverable and action-scoped.
- [ ] Run `cargo test --locked -p cockpit-repository --test environment_drift` and `cargo test --locked -p cockpit-repository --test collaboration_admission`.
- [ ] Commit shared environment-drift admission as `feat(coordination): persist observed environment drift`.

### Task 4: CLI/MCP capability and real concurrent acceptance

**Files:**
- Modify: `crates/cockpit-cli/src/main.rs`
- Modify: `crates/cockpit-mcp/src/lib.rs`
- Modify: `.ai/agent-interface.json`
- Modify: `tests/ci/repository_gate_manifest.json`
- Create: `crates/cockpit-cli/tests/contract_amendment_processes.rs`
- Test: `crates/cockpit-mcp/tests/work_item_amendment_parity.rs`

**Interfaces:**
- CLI extends `work-item amend` to accept typed requests while retaining the legacy additive input adapter; adds read-only `work-item amendments` and an explicit coordination environment-drift write action.
- MCP exposes `work_item_amend`, `work_item_amendments`, and the matching environment-drift coordination action with strict schemas and read/write separation.
- Agent capability metadata declares the amendment and environment-drift capability; older Runtime identity without those semantics is rejected for affected mutations/admissions.

- [ ] First add a CLI help assertion for the typed amend request and MCP tool-list assertions for `work_item_amend` and `work_item_amendments`; run and confirm these surfaces are absent before implementation.
- [ ] Add CLI/MCP parity tests for successful writes, validation/conflict errors, identical receipts, read-only history inspection, and capability incompatibility.
- [ ] Add an OS-process acceptance test that creates multiple linked worktrees, starts independent CLI processes against the common directory, changes observed environment while keeping input JSON unchanged, and proves affected denial before process spawn plus unrelated-action continuation and recovery.
- [ ] Add a serial single-Work-Item CLI lifecycle assertion with default one-worker behavior.
- [ ] Register the process acceptance command in `repository_gate_manifest.json` and assert the manifest test sees the registration.
- [ ] Run `cargo test --locked -p cockpit-cli --test contract_amendment_processes`, the MCP parity test, and `tests/ci/repository_gate_manifest_test.sh`.
- [ ] Commit CLI/MCP capability and process acceptance as `feat(cli): expose contract amendment and drift controls`.

### Task 5: Agent guidance, multilingual references, and canonical verification

**Files:**
- Modify: `.ai/README.md`
- Modify: `agents/skills/README.md`
- Modify: `agents/skills/ordinary-work-item.md`
- Modify: `docs/reference/commands.md`, `docs/reference/commands.ja.md`, `docs/reference/commands.zh-CN.md`
- Modify: `docs/reference/contract-fields.md`, `docs/reference/contract-fields.ja.md`, `docs/reference/contract-fields.zh-CN.md`
- Modify: `docs/reference/agent-workflow.md`, `docs/reference/agent-workflow.ja.md`, `docs/reference/agent-workflow.zh-CN.md`
- Test: `tests/docs/documentation_acceptance.sh` and the registered doc-parity tests

- [ ] Document amend operations, protected/sensitive fields, reason/digest conflicts, idempotent retry, evidence invalidation, environment-drift recording, refresh-before-action, serial fallback, and N-1 compatibility in all three languages.
- [ ] Document that observation/query is read-only and that only explicit Runtime write actions amend or publish drift events; do not describe the request-scoped ledger as cross-process persistence.
- [ ] Run the Contract-declared checks: `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`, and `cargo test --locked --workspace`.
- [ ] Run the Runtime gate/preflight for this Work Item, the canonical Rust/CI gate manifest, and the multilingual documentation acceptance on the same reviewed snapshot; preserve any failed receipt and do not claim the release is published.
- [ ] Re-read this plan and the spec; verify each acceptance criterion maps to a test and each Review Focus case has a negative or process-level regression.
- [ ] Commit the agent guidance and three-language references as `docs: explain contract amendments and environment drift`.
