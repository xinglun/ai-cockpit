# AI Cockpit Collaboration, Observation, and Performance Design

Status: approved for implementation by the current user authorization.

## Goal

Make every collaboration Outcome state and next action derive from the same
typed, repository-bound facts; make Outcome assembly fail closed when any
fact dependency changes during a request; prove those semantics through real
CLI and MCP execution; and measure performance with operation-scoped traces
before retaining any optimization.

The delivery starts at `c72254c2ae6bc712139fc755698694653ae004ba`, preserves
the published `v0.2.90` identity, and discovers the next version only after
the reviewed implementation is merged.

## Non-negotiable boundaries

- Historical Contracts, receipts, archive bytes, and decision records remain
  immutable.
- Existence, mtime, size, and cache-hit signals are never evidence that a
  governance fact is valid or unchanged.
- Rendering consumes an already assembled input and performs no filesystem,
  Git, Runtime, or subprocess I/O.
- CLI and MCP call the same repository Outcome assembly and action projection.
- A successful verification, a resource cleanup, and human authorization are
  separate facts; none is inferred from another.
- A trace is diagnostic metadata only and is excluded from evidence identity
  digests.
- Every retained performance change has an isolated commit, correctness
  evidence, before/after samples, and a measured benefit or is reverted.
- No persistent cache or resident process is part of the default delivery;
  it requires a new measured bottleneck decision.

## Shared semantic model

Introduce one typed finalization observation model in the protocol/repository
boundary. It must distinguish at least:

`NotRequired`, `ReceiptMissing`, `VerifiedRetained`, `VerifiedDeleted`,
`VerifiedAbandoned`, `RecordCorrupt`, `IdentityMismatch`, `CleanupPending`,
and `Unknown`.

The model also carries a stable action identifier, an optional Runtime command
shape, whether the action is observational or mutating, whether authorization
is still required, and the reason/evidence references that made the action
safe. Existing wire string fields remain compatible; additive typed fields
are preferred over a breaking rename.

Action construction is exhaustive over the typed state and error enum. In
particular:

- missing receipts lead to re-observation and the actual receipt-recording or
  recovery route, followed by verification; they never point to a
  verification-only command as if it recorded a receipt;
- retained resources explicitly produce a no-delete action;
- deleted/abandoned resources only produce the required close-recording path;
- corrupt or identity-mismatched records enter recovery/binding inspection;
- unmet cleanup postconditions require re-observation and do not authorize
  cleanup;
- unknown facts remain blocked and expose a real inspection/recovery route.

Human localization renders this action object. Machine JSON serializes the
same object. No endpoint or language-specific renderer may infer an action
from an error string or invent a command.

## Request-scoped observation boundary

Add a request-local observation ledger owned by Outcome assembly. Each
registered dependency records its repository-relative path, kind, existence,
regular-file/type status, content digest when read, parsed fact identity, and
the reason it participates in the decision. Directory dependencies additionally
record the complete candidate member set and each member's type/content
identity.

The ledger covers every fact read by Outcome, governance, human decision,
archive verification, recovery/successor resolution, and finalization. This
includes active and archived Contract/Summary/Outcome, close and preflight
decisions, finalization receipts and transitions, task Outcome events,
archive-manifest references, historical artifacts, and the candidate sets of
the decisions/archive directories.

Consumers reuse the same read and parse result inside one assembly attempt.
The final boundary rechecks all registered dependencies and directory member
sets independently of the reuse counters. A byte, type, identity, addition,
deletion, or candidate-set change causes one bounded retry. If the second
attempt is not stable, the assembly returns an explicit unknown/error and
never a mixed-fact success.

The ledger is not exposed to pure rendering and does not claim a filesystem
transaction. It only proves the bounded observation guarantee described by
the Outcome metadata.

## Real collaboration acceptance

Extend the scenario matrix with structured facts, expected state, blockers,
action, forbidden inferences, and verification commands. The matrix runner
builds controlled repository fixtures through production lifecycle commands,
then invokes both the CLI subprocess and MCP handler. It covers all required
finalization states, mutation/drift cases, blocked and stale authorization,
no-chat-history handoff, and the en/zh-CN/ja × summary/full product.

Tests must fail when a matrix expectation is changed without changing the
implementation. Key recovery actions execute their follow-up Runtime command
and assert the resulting state, not merely a phrase in rendered text.

## Performance measurement model

Every measured command gets an `operationId`, `scenarioId`, `measurementId`,
Runtime identity, repository identity, and raw sample sequence. Phase spans
record parent/span relationships, wall versus internal time, and explicit
overlap/nesting. Counts are incremented at the real Git, file read, digest,
parse, and Runtime child-process boundaries.

External CLI process startup, benchmark probes, warmups, benchmark metadata
Git calls, and the measured command's internal trace are separate metrics.
Each report references only its own operation trace. Unavailable metrics carry
an explicit reason. The benchmark retains the first sample separately from at
least 100 valid warm samples, preserves raw order, and reports p50/p95 only
when their validity floors are met. It must state that a first sample is not a
cold-cache result unless the operating-system cache was actually controlled.

## Optimization gate and order

After the corrected baseline is accepted, evaluate one change at a time:

1. reuse immutable parsed facts within one request;
2. load only the target Work Item's required dependencies;
3. share decision/archive/finalization candidate indexes within the request;
4. reuse one Git snapshot and related topology observations;
5. reduce JSON/value cloning and duplicate serialization;
6. add bounded parallel reads only where independence and benefit are measured.

Cross-request caches, persistent indexes, and resident processes are deferred
unless the preceding evidence identifies a remaining bottleneck that justifies
their invalidation and lifecycle cost. A target hot path should pursue the
instruction's p95 improvement goal, but correctness and measurement validity
always remain gates; a repeatable regression above 5% requires repair or an
explicit accepted reason.

## Parallel delivery boundaries

The integration owner controls shared `crates/cockpit-repository/src/lib.rs`
interfaces and final release state. Agent A owns typed action/error semantics
and localized projection. Agent B owns the observation ledger and dependency
boundary. Agent C owns the real scenario runner and matrix expectations.
Agent D owns trace schema, counters, benchmark scope, and baseline. A/B
interface decisions precede their implementation; C consumes the stable A/B
projection; D coordinates with B on actual read/hash counters. E is serial
after D's baseline and is split into independently reviewable commits.

No two agents write the same checkout or generated `.ai` evidence. Each
parallel stream is reconciled against current `origin/main` before its
deliverable is integrated.

## Acceptance evidence

The final Work Item must bind focused Rust tests, real CLI/MCP scenario
results, mutation and bounded-retry tests, corrected benchmark raw samples
and summaries, optimization before/after evidence, full workspace checks,
documentation/governance checks, reviewed PR/merge state, the new published
version, downloaded artifact installation, N-1 upgrade history preservation,
and exact post-close cleanup. Any missing or contradictory evidence remains
unknown rather than being promoted to green.
