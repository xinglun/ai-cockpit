# Cross-Work-Item Behavioral Closure Design

## Status

Approved corrective design for `WI-1030-cross-wi-behavioral-closure`. The
existing collaboration model, commands, and tests remain in place where they
are correct; this design closes the missing behavior at the execution and
evidence boundaries identified by the independent red acceptance review.

## Goal

Make cross-Work-Item collaboration executable and evidence-bound: a
registration describes observed repository facts, affected actions are
admitted only after a fresh dependency check, safe pause actually prevents the
target action, composition uses the bounded executor and durable attempts,
reuse changes process execution, and the shared Outcome reports real facts.

## Non-negotiable boundaries

- Keep one active WI, branch, linked worktree, and repository context for this
  correction.
- Use the fixed `0.2.105` Runtime for lifecycle responsibility. The candidate
  Runtime built by this branch owns collaboration capability and composition
  behavior verification. This is not a bidirectional compatibility claim.
- A fixed Runtime may continue ordinary single-WI lifecycle operations, but it
  must not silently consume candidate collaboration records. Candidate
  collaboration actions fail closed before writing when the capability or
  runtime binding is absent or mismatched.
- Query operations are read-only. Registration, impact reporting,
  coordination, recovery, and composition are explicit write/action paths.
- Do not infer repository identity, Contract identity, branch, head, or
  worktree topology from caller declarations. Validate them against Git and
  Runtime-owned records before persistence or admission.
- Do not turn “request confirmation” into another governance gate. Existing
  user authorization and Runtime action admission remain the authority; a
  coordination request is an execution-state fact, not a human decision.
- Preserve ordinary single-WI serial execution when no collaboration
  declaration is present.
- No release, tag, provider mutation, or merge is part of this corrective
  implementation. The handoff may produce a reviewable PR, but release stays
  blocked until independent review and post-merge acceptance.

## Corrective behavior

### 1. Registration and identity

Before writing a registration, the Runtime resolves the caller's Git common
directory and repository identity, loads the active Contract for the Work
Item, computes its digest, and observes the linked worktree topology,
branch, and `HEAD`. The supplied registration must match every observed
identity dimension, the candidate Runtime binding, and the current generation.
Required `ConsumedOutcome` evidence is checked against actual current evidence;
the `verification_required` flag is not merely copied into a projection.

Inspection revalidates registrations against current facts and classifies
missing, moved, stale, corrupt, or mismatched records as unknown or
recovery-required. It never repairs state.

### 2. Admission, pause, and resources

The action-admission API accepts a concrete action and, where applicable, the
consumer/result identity. It refreshes Git, Contract, registration, event,
resource, and coordination-request state before deciding.

`SafelyPaused` rejects that target action before any composition or verification
process is spawned. An unrelated Work Item remains admissible. Resume appends a
state transition and forces a fresh dependency evaluation.

Reservations and releases validate the current registration generation and
repository identity. An old owner cannot release a reservation after a new
generation is registered; the reservation remains until a current-generation
operation resolves it through the explicit recovery path.

### 3. Composition and bounded execution

The composition entrypoint obtains a Runtime-built identity binding and
computed preconditions. Caller-supplied `satisfied` values are not sufficient.
Empty commands, missing required checks, identity mismatches, and blocked
dependencies fail before an expensive process starts.

Composition adapts nodes to the existing bounded verification executor. Each
node has a durable attempt record before/after execution, bounded output,
timeout and interruption state, and an explicit cleanup result. A failed or
interrupted attempt is immutable and remains available for recovery; temporary
worktree cleanup errors are not discarded.

### 4. Actual reuse

The execution path loads the previous immutable node attempts, calls the reuse
classifier, and skips only nodes whose full identity and dependency proof
matches. A second identical CLI or MCP request must start fewer processes. A
local identity change must rerun the changed node and only its transitive
dependents. A classification unit test without a process-count assertion is
not sufficient evidence.

### 5. Impact and recovery

Publication and invalidation are distinct event semantics. Publishing an
Outcome does not automatically block consumers; a change to a Contract,
head, interface, resource, execution, or verification identity can.

Recovery is append-only and cross-generation. An old-generation invalidation
remains in history, while a current-generation resolution record references
the predecessor event and proves the current provider facts. Recovery does not
require the provider to still be on the event's old generation and never
deletes the old event.

Before a dependent action, the Runtime recomputes the current provider head,
Contract digest, declaration digest, and required evidence. A stale event is
not cleared by deletion or by a caller-provided boolean.

### 6. Shared Outcome and delivery

The shared projection reads composition attempts from the common directory and
reports the latest identity-bound composition, merge-target observation,
cleanup result, and per-node reuse. It distinguishes unknown/not observed from
false or passed. A pending coordination request does not directly set
`human_decision_required`; that field is reserved for a real human decision
boundary.

CLI and MCP use the same repository service and projection. Their read paths
remain side-effect free and their action paths return the same typed facts.

### 7. Acceptance and canonical gates

The acceptance suite uses real concurrent processes and multiple linked
worktrees. It covers identity rejection, pause-before-spawn, generation-safe
resources, empty/incomplete composition, timeout/interruption persistence,
actual CLI/MCP reuse, cross-generation recovery, shared Outcome facts, and
single-WI serial execution.

The multi-process acceptance script is directly listed in
`tests/ci/repository_gate_manifest.json`; its existence alone is not gate
coverage. The repository has no Makefile, so the canonical route is Runtime
CLI plus Cargo/CI gates. A missing Makefile is not itself a failure.

## Compatibility and persistence

Existing Contracts without collaboration fields remain readable. Existing
Runtime-generated lifecycle records remain immutable and are changed only by
Runtime entrypoints. Coordination records are repository-local observations
and actions, not a second Contract authority. All writes are atomic and
deduplicated by stable identity. Unknown or partial state fails closed.

## Acceptance evidence

The Work Item must contain fresh evidence for:

1. registration, dependency, pause, resource, event, and recovery behavior;
2. durable bounded composition attempts and cleanup outcomes;
3. real process-count reduction for CLI and MCP reuse, including partial
   invalidation;
4. multi-process linked-worktree acceptance through the canonical gate;
5. CLI/MCP/Outcome projection parity and all three documented languages.

The final pre-release handoff must separately state implementation status,
verification status, PR/host status, release status, unknowns, and the fact
that independent review remains required.
