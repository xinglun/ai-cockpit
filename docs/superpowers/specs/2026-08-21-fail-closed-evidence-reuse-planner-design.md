# Fail-Closed Evidence Reuse Planner Design

## Context

AI Cockpit already names content-, diff-, and environment-bound evidence, but
the current implementation compares every binding class with one content
digest. The verification executor also accepts a caller-controlled `reuse:
bool` and skips a process without validating a receipt. Dependency edges are
available in the graph model but are discarded before command execution.

That boundary is unsafe: a stale, malformed, failed, or self-declared reuse
candidate can suppress a real verification command. WI-32 replaces it with an
executable, fail-closed decision and planning boundary aligned with the locked
V1 Oracle's semantics without installing or running V1 as the product.

## Decision

### Composite evidence identity

`cockpit-evidence` will expose one serializable `EvidenceContext` containing:

- content digest;
- base commit, head commit, and normalized changed-path digest;
- environment, command, scope, governance, toolchain, and policy digests;
- stage and runner identity.

Every reusable receipt also binds its schema version, receipt id, node id,
passed result, output digest, creation time, and expiry time. The receipt id is
SHA-256 over a deterministic serialized body that excludes the id itself.
Digest fields are validated as typed `sha256:<64 lowercase hex>` identities;
Git object ids are full lowercase hexadecimal identities, empty textual
identities and invalid time windows are rejected, and serde rejects unknown
schema fields instead of silently discarding unbound input.

### Fail-closed states

The pure evidence classifier returns `Fresh`, `Stale`, or `Unknown` plus an
explicit `Reuse` or `Execute` action and reason code.

- Only a non-protected, passed, non-expired, internally valid receipt whose
  node and complete context exactly match is `Fresh/Reuse`.
- Binding or expiry mismatch is `Stale/Execute`; a receipt naming another node
  is invalid evidence and therefore `Unknown/Execute`.
- Missing, malformed, tampered, failed, future-dated, or otherwise invalid
  evidence is `Unknown/Execute`.
- A protected node always executes regardless of candidate freshness.

### Plan before execution

`cockpit-verification` will replace `with_reuse(bool)` with a required typed
policy: `Protected(class)`, `Reusable`, or `NeverReuse`. There is no default
boolean builder to omit. A typed reuse candidate is considered only for the
`Reusable` policy. The command digest is recomputed from the actual program,
arguments, and working directory at the verification boundary; caller-supplied
command identity cannot authorize a skip.

The planner validates unique command ids and dependency topology, evaluates
each candidate once, and produces an ordered plan without spawning processes.
If any dependency is planned for execution, a downstream otherwise-fresh
candidate is changed to `Stale/Execute` with a dependency-rerun reason. The
bounded executor then schedules only `Execute` entries. A dependent becomes
runnable only after all executed dependencies finish; independent ready nodes
remain bounded-parallel. A reused command is never spawned.

Commands without a reuse candidate remain `NotApplicable/Execute` for
backward-compatible protected and ordinary verification. `NotApplicable` is
executor telemetry, not a fourth evidence-classifier state.

### Metrics

The execution receipt retains existing counts and adds stale reruns, unknown
reruns, protected nodes executed, protected nodes skipped, spawn failures, and
per-node action/state/reason/receipt/satisfaction results. A passing protected
boundary requires `protectedNodesSkipped == 0`. Tests prove actual process-call
reduction by selecting commands whose execution would fail.

## Component Boundaries

### `cockpit-evidence`

Owns receipt/context schemas, receipt identity construction and validation,
digest validation, state/action/reason types, and the pure reuse decision. It
does not know process commands, dependency graphs, repositories, or storage.

### `cockpit-verification`

Owns actual command identity derivation, typed reuse candidates, dependency
planning, bounded process execution, and execution metrics. It consumes the
pure evidence decision and never accepts an unverified skip flag.

### CLI and MCP

Existing CLI/MCP project commands are protected and continue to execute. This
Work Item does not add a receipt file format, profile policy, or repository
storage adapter. Those cross-process integrations require a later Work Item.

## Error Handling

Duplicate command ids, missing dependencies, dependency cycles, zero workers,
or poisoned worker synchronization are explicit errors. Evidence uncertainty
is not an execution error: it becomes `Unknown/Execute`. Command spawn or
non-zero exit marks the execution receipt failed.

## Rejected Approaches

- Retaining `reuse: bool` cannot prove provenance, freshness, or identity.
- Checking only content ignores source range, change set, environment, policy,
  and invocation drift.
- Treating malformed evidence as stale or reusable obscures trust failure;
  malformed evidence is unknown and executes.
- Letting a fresh child skip after a dependency reruns can combine evidence
  from incompatible executions.
- Adding persistence in the same Work Item would mix the pure safety boundary
  with repository/profile policy and weaken reviewability.

## Verification

Evidence tests cover an exact fresh receipt, mutation of every binding,
expiry, missing/failed/future/malformed/tampered receipts, and protected nodes.
Verification tests prove that fresh evidence suppresses an actually failing
command, stale/unknown/protected commands execute, and a dependency rerun
forces downstream execution. A mutation check must demonstrate that weakening
one mismatch rule fails the suite. Full workspace, Clippy, rustfmt, diff,
workflow, multilingual, and locked V1 Oracle gates remain required.

## Scope Boundary

WI-32 does not create `.ai/`, attach this repository, commit, push, publish,
sign, or claim hosted CI. It closes the in-process planner/executor trust
boundary. Cross-process receipt persistence and profile-driven CLI/MCP reuse
will be opened only after this boundary is accepted.
