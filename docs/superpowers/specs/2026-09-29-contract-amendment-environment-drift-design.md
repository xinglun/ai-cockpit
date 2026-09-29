# Contract Amendment and Environment Drift Design

## Problem

An active Work Item Contract is an implementation plan, not an immutable ideal. Real work and parallel execution can invalidate assumptions. The current `work-item amend` command only appends a small allowlist of fields, so an agent cannot correct a known-wrong command, replace an acceptance criterion, remove obsolete scope, or reorder a plan without leaving the governed path. Environment changes are also not first-class, durable facts: a request-local observation cache cannot notify other processes or linked worktrees.

The feature must let an owner adapt a plan at any active lifecycle point while preserving identity, authorization, evidence history, and fresh verification. It must publish observed environment changes through durable shared state and make each affected action refresh that state before execution.

## Goals and non-goals

Goals:

- Support schema-aware `add`, `set`, `clear`, `remove`, `replace`, and `reorder` operations over human-owned Contract plan fields.
- Require a reason and expected Contract digest; validate the full resulting Contract before one amendment commits.
- Retain an append-only, integrity-verifiable amendment history, including old/new values, evidence invalidation, snapshot binding, and idempotent retry identity.
- Keep Runtime-owned identity, observed facts, lifecycle transitions, and historical receipts outside generic amendment.
- Record runtime-observed environment drift in the existing common-directory coordination store; refresh dependency admission before affected actions and allow unaffected actions to proceed.
- Preserve a serial single-Work-Item lifecycle and default execution. Concurrency is exercised in acceptance tests, not used to parallelize implementation.
- Expose matching CLI and MCP read/write operations and document the supported behavior in English, Japanese, and Simplified Chinese.

Non-goals:

- Release publication, tag mutation, Task 9 script migration, or changes to WI-1040/WI-1041.
- Rewriting archived Contract or evidence bytes, changing Runtime-observed facts through Contract input, or adding a second collaboration store.
- Automatically terminating an already-running external process. Drift affects the next safe action boundary; any in-flight cancellation uses an existing explicit process-control capability if available.

## Contract amendment protocol

Add a typed `ContractAmendmentRequest` in `cockpit-protocol` with `schemaVersion: 1`, unique `changeId`, `expectedContractDigest`, non-empty `reason`, and ordered `changes`. Each change has a canonical JSON Pointer `path`, an `operation`, and an optional JSON `value`; only the operation-specific operand is accepted. `add` appends one typed collection element, `set` assigns an existing typed field, `clear` takes no value and clears only an optional/clearable field, `remove` takes the exact existing collection element, `replace` takes the replacement for an existing path, and `reorder` takes the full sequence as an exact permutation of current elements. The field registry supplies stable identities for keyed collections; index paths are range-checked against the value at that point in the batch. Missing, duplicate, ambiguous, unknown, or protected paths fail closed. A batch is ordered and atomic: later operations see earlier operations in the same batch.

The field registry is explicit and derived from the typed Contract schema, not an open-ended JSON patch. It classifies fields as:

1. **Plan-editable:** human-authored intent/goal, scope, out-of-scope, risk assessment, acceptance, evidence requirements, sources, verification, guidelines, rollback, unknowns, scenario plans, and other typed plan declarations.
2. **Sensitive plan-editable:** authority, approval/gate requirements, risk level, verification strength, and concurrency constraints. These may change only through normal policy evaluation; weakening or authority changes invalidate prior admission and trigger the existing required review/reauthorization gates. A reason is rationale, never authority.
3. **Runtime-protected:** repository/Work Item identity, base and snapshot identity, branch/worktree facts, lifecycle state, timestamps, generated checkpoints, resource identity, lineage, and immutable receipts. Generic amendment rejects these paths.

`clear` is legal only where the schema allows absence; it is not an alias for deleting required values. Every prospective Contract is deserialized into the current typed schema and passes all cross-field, policy, and lifecycle validation before persistence. The Runtime recomputes aliases/projections such as `acceptance`/`acceptanceCriteria`; callers cannot submit contradictory mirrors.

Under a per-Work-Item lock, the Runtime compares the expected digest with the current canonical Contract digest, computes the whole prospective Contract, and evaluates policy impact. A mismatch returns a conflict containing the current digest, never silently rebases. Reusing a `changeId` with identical request bytes returns the committed receipt; reusing it with different bytes is an error.

Commit through a recoverable write-ahead transaction. Append a prepared journal record, atomically replace the Contract, update only Runtime-owned derived projections, then append a committed record linked to the prior audit digest. Readers either recover/observe a complete transaction or fail closed on unresolved transaction state. A committed receipt binds repository ID, Work Item ID, sequence, change ID, request digest, reason, changed paths and canonical old/new values, previous/new Contract digests, repository/environment observation digests, invalidated checks/evidence, policy review requirements, and the previous/current journal digest. History is append-only and moves into the archive bundle without rewriting prior entries. A crash/retry test must cover every persistence boundary.

Amendments remain available through active lifecycle states before the immutable finish checkpoint. After archival, the predecessor remains immutable and the established successor/recovery path is used. Any amendment invalidates affected verification and governance projections; fresh preflight and required verification are mandatory before finish/archive.

## Environment drift and coordination

Reuse `RepositoryExecutionContext` to capture execution facts and `CoordinationStore` for cross-process persistence. `ObservationLedger` remains request-scoped and read-only; it is not used as an event bus. An explicit write operation records an environment-drift event only from Runtime-observed facts: repository/common-directory identity, provider Work Item and generation, old/new environment digest, affected outcome identities, and observed source fields. The event has a stable idempotency key and is immutable. Unknown/unobservable relevant inputs disable reuse and remain explicit unknowns rather than being represented by caller-supplied digests.

Store the event under the existing Git common-directory coordination root, protected by its inter-process lock. Registration refresh/retry must reconcile a committed identity change with its event so interruption cannot leave an effective new registration without its impact record. Published outcomes are not invalidation events. Recovery appends a resolution relation to the historical event; it never deletes or rewrites it, and may resolve a prior generation after the provider advances only when the current outcome identity is freshly verified.

Before verification, composition, reservation/release, or another declared consumer action, refresh the shared event/registration state and evaluate only the action's consumed outcomes. Relevant stale generations and receipts are denied before process/resource side effects. Unaffected outcomes and unrelated Work Items continue on their own fresh admission. Preserve the existing pause request/acknowledgement/resume protocol; pause state denies execution through the same admission result, and resume re-evaluates current dependencies. Read-only inspection never records or consumes events.

## Interfaces and compatibility

Extend `work-item amend` with request-file input and add a read-only amendment-history query. Add equivalent MCP tools/schemas for amendment writes and audit reads. Add an explicit environment-drift record action to the existing coordination CLI/MCP interface; existing inspect/query actions remain read-only. Runtime action admission, not CLI/MCP prose, is authoritative.

Declare an explicit Runtime capability for this amendment and environment-drift protocol. Each amended Contract records the required Runtime capabilities; an older strict-schema Runtime such as `0.2.113` must reject the added field during Contract parsing, before it can admit an action. A Runtime that can parse a Contract but lacks the amendment journal, shared event refresh, or protected-field semantics is likewise incompatible for actions that depend on them. Probe the exact predecessor directly in compatibility tests: Contract readability alone is not evidence of bidirectional support, and unknown capability/version state fails closed for the affected action. The new Runtime continues to support the ordinary serial single-Work-Item path.

## Verification and acceptance

- Unit tests cover every operation over scalar, optional, nested, and collection fields; invalid types, protected/unknown fields, digest conflicts, policy weakening, aliases, and full Contract invariants.
- Crash/retry tests interrupt each amendment commit step and prove no lost, duplicated, or partially visible committed state.
- Runtime tests prove observed environment changes produce persistent, deduplicated shared events; publishing an outcome does not invalidate; stale generations cannot act; recovery is append-only across provider generations; and outcome-level impact allows unrelated actions in the same Work Item.
- CLI/MCP parity tests prove identical validation, receipts, errors, and read-only inspection behavior.
- A real acceptance test starts independent Runtime processes in multiple linked worktrees over one Git common directory. It changes the observed execution environment without changing the submitted Contract JSON, then proves affected action denial before process spawn, unrelated action continuation, recovery, and stale-receipt rejection. A separate serial lifecycle test proves default one-worker behavior still works.
- Run the repository canonical Runtime/Cargo/CI gates and the required multilingual documentation acceptance. Register the process acceptance test in the canonical gate manifest.

## Risks and mitigations

- **Concurrent edits:** expected-digest compare-and-swap plus a common per-item lock prevents lost updates; callers re-read and make a new reasoned amendment.
- **Partial persistence:** write-ahead states and deterministic recovery make interruption visible and recoverable; unresolved state blocks admission.
- **Gate weakening disguised as plan maintenance:** classify sensitive fields and re-run policy/authorization rather than treating a reason as approval.
- **Old Runtime silently ignoring collaboration constraints:** capability compatibility rejects affected actions unless all required semantics are present.
- **Overblocking after drift:** bind impact to consumed outcomes and action identity, not the whole Work Item; unknown impact blocks only the action whose required facts cannot be proved.
- **False environment claims:** derive digests from observed execution context; if the Runtime cannot observe an input, do not enable reuse that depends on it.
