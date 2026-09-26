# WI-1031: Cross-WI acceptance corrections

## Purpose and boundary

This specification narrows the follow-up to defects in the independent acceptance of WI-1030. Existing collaboration records and successful behavior remain compatible where their facts can be verified. Historical receipts and events remain append-only. This work stops before release: it does not create or move a tag, publish artifacts, or declare release acceptance.

The installed repository-bound Runtime is 0.2.113. Runtime remains authoritative for lifecycle and action admission; this change does not replace its governance model or add an additional confirmation gate.

## Required behavior

### Composition identity and reuse

- Runtime derives source identity from the resolved target tree and verifies each participant against current registration and Contract facts.
- Runtime observes command identity and the executable/toolchain and execution environment it actually launches. Caller-supplied digests are hints, never sufficient authority for reuse.
- Any execution-affecting input that cannot be observed or normalized makes that node non-reusable. A changed observed input with unchanged input JSON must execute again or fail closed; it must never report successful reuse.
- Node reuse is scoped to that node's direct inputs and dependency receipts. A change to one node must not invalidate unrelated nodes, while a changed upstream node invalidates its transitive dependents.

### Composition coverage and target resolution

- Resolve the declared target ref from repository facts rather than requiring it to equal the caller's current checkout. A consumer feature worktree must be able to compose against the actual `main` target SHA.
- Validate participants and dependency closure against current Runtime registrations, consumed outcomes, and composition responsibility.
- Derive the required scenario/check set from declarations, including required scenarios, compatibility constraints, composition order, and required participants. The command list must cover that set; omitting one of two required nodes must fail before spawn.
- Preconditions must be derived or verified from declared facts; caller-supplied `satisfied: true` is not proof.

### Verification evidence and merge facts

- Keep descriptive references distinct from verification evidence.
- A verification dependency is satisfied only by a supported, successful, current receipt bound to provider Work Item, generation, head, Contract digest, and required verification identity. Empty JSON, arbitrary files, stale receipts, or mismatched receipts do not satisfy it.
- A Runtime write operation publishes a selected outcome for the current registration generation. The append-only `OutcomePublished` event binds the exact referenced evidence bytes; for a `verification_required` consumer, it must reference the current typed verification envelope. CLI and MCP expose the same explicit `publish-outcome` action. Inspection stays read-only, and changing the evidence after publication invalidates the binding.
- `MergedTarget` requires a verifiable merge fact on the declared target, supported by the local Git graph and/or an accepted provider receipt; a temporary composition worktree alone is not a merge.

### Impact consistency and action-level admission

- Registration identity changes and their invalidation event must have a recoverable write order. Repeating the same registration after interruption must reconcile the event idempotently; consumers must fail closed during any inconsistent intermediate state.
- Admission refreshes dependencies before each action and filters invalidated dependencies by that action's declared outcome IDs. In one Work Item, an action consuming an unaffected outcome can proceed while an action consuming an affected outcome waits.
- Impact propagates through at least a three-level dependency chain. Recovery appends a resolution for the relevant event/consumer generation without deleting or rewriting historical events.

### Human-facing projection

- Report temporary composition state separately from actual target merge state.
- Preserve historical successful attempts, but report current applicability as stale when target/head/Contract identity has moved.
- Compute Outcome action state from the same refreshed admission result used by execution. A safely paused current generation is not `allowed`.
- Preserve the distinction between an agent-to-agent coordination request and a human decision.

## Focused regression scenarios

Every scenario begins unverified in the Contract and becomes verified only after its test or process-level evidence is recorded:

1. Actual execution environment/toolchain changes while composition JSON keeps its old digest: reuse is denied and the affected command is re-executed.
2. Two checks are required but only one is supplied: composition is denied before any process starts.
3. A consumer feature worktree resolves and composes against the actual `main` branch/SHA.
4. Empty, malformed, stale, or identity-mismatched files cannot satisfy a verification dependency; a matching successful receipt published in the current generation can, while an older-generation event or evidence mutation cannot.
5. A temporary merge without target integration never projects as merged; an actual target commit that contains the participant is required.
6. A prior composition remains in history but is projected stale after target advancement.
7. A safely paused generation is blocked in both action admission and Outcome; a different Work Item remains runnable.
8. Interruption between registration/event writes followed by an identical registration retry leaves one valid impact event and no admission gap.
9. Two actions in one consumer Work Item, each naming a different outcome, admit only the action whose dependency is unaffected.
10. An invalidation at level one propagates through levels two and three; resolving one consumer does not erase the provider event or authorize an unrelated generation.
11. An ordinary single-Work-Item workflow with no collaboration declarations or coordination state continues to run serially through the existing lifecycle and canonical verification route.

## Compatibility and verification

The 0.2.105 lifecycle executor remains responsible for its existing lifecycle commands and record interpretation. Candidate composition and coordination behavior is verified with Runtime 0.2.113 and current contracts. No claim of backward write compatibility is inferred from old read compatibility. Where an older Runtime cannot preserve or enforce new collaboration constraints, the operation must fail closed or be routed through a compatible candidate Runtime; it must not silently ignore fields.

Verification uses the repository's canonical Runtime CLI + Cargo/CI gate (not Make), plus real concurrent processes and multiple linked worktrees. Query paths remain read-only; durable impact, recovery, and coordination changes go through explicit write APIs.

The `publish-outcome` write path is part of the evidence contract, not a query side effect: it resolves the current registration and outcome, validates any verification receipt required by registered consumers, and appends a non-invalidating event with evidence digests. The CLI and MCP route to this same repository service.
