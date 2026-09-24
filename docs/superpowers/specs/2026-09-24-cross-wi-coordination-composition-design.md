# Cross-Work-Item Coordination and Pre-Merge Composition Design

## Status

Proposed for the authorized implementation Work Item. This document is the
design interpretation of the user-supplied development instruction; it is not
an acceptance result for the current implementation.

## Goal

Extend the existing Contract concurrency boundary, dependency/conflict
sidecars, repository-local parallel slots, observation ledger, and CLI/MCP
projections into one evidence-bound collaboration loop:

```text
declare dependency -> observe result -> report impact -> coordinate execution
-> verify an exact composition -> hand off the integration result
```

The implementation must expose cross-Work-Item effects early enough to avoid
late integration failures while preserving fail-closed authorization,
ordinary single-Work-Item behavior, and immutable historical evidence.

## Scope

### In scope

- Same-machine coordination across linked worktrees in one Git common
  directory.
- Explicitly participating Work Items, their Contract identity, worktree
  identity, current head, declared outputs, dependencies, resources, and
  verification constraints.
- Repository-local coordination records under the resolved Git common
  directory at `.ai-cockpit/coordination/v1/`.
- Atomic resource reservations, generation/lease validation, event identity,
  duplicate suppression, recovery diagnostics, and cycle detection.
- Stage-aware dependency and impact evaluation without requiring a provider
  Work Item to be closed before its composable head can be consumed.
- Bounded coordination intents: wait for dependency, request safe pause,
  adjust integration order, and resume/re-evaluate.
- Exact composition verification in an isolated temporary worktree, including
  identity binding, cheap preconditions, text-conflict checks, affected
  verification, execution persistence, and evidence-reuse decisions.
- A shared read-only domain projection consumed by CLI, MCP, and human
  Outcome delivery.
- Real Git linked-worktree, concurrent-process, failure-recovery, and
  cross-WI integration tests; focused performance and governance-cost
  measurements.
- English, Simplified Chinese, and Japanese documentation for the supported
  behavior and explicit unsupported topology boundaries.

### Out of scope

- Cross-machine coordination, independent-clone global locking, daemon or
  resident service, and a general scheduler.
- A standalone subtask-progress API, hand-maintained ledger, global skill
  modification, unrelated history cleanup, or automatic provider resource
  takeover.
- Making collaboration declarations mandatory for ordinary single-WI work.
- Inferring semantic compatibility from paths, file size, timestamps, or a
  digest without a direct machine-checkable rule or composition evidence.
- Automatic push, merge, release, tag mutation, cleanup takeover, or release
  publication.

## Existing foundations to preserve

The implementation extends, rather than replaces:

- `ConcurrencyBoundary` and `ParallelSlotLease` in
  `crates/cockpit-protocol`.
- Contract-owned dependency/conflict intelligence and repository-local slot
  operations in `crates/cockpit-repository`.
- Request-scoped fact observation in `execution_context.rs` and
  `observation_ledger.rs`.
- Shared Outcome assembly and pure human rendering in
  `outcome_render.rs`.
- Existing CLI/MCP Work Item and parallel-operation adapters.
- Existing collaboration scenario matrix, CLI/MCP parity tests, and gate
  manifest conventions.

No second authority is introduced in a sidecar or coordination directory.
The Contract remains the human-owned declaration; Git remains the source of
commit/worktree facts; Runtime remains authoritative for admission and
evidence validity; coordination records are repository-local observations and
events only.

## Runtime compatibility boundary

The installed Runtime is fixed for this Work Item at version `0.2.105` with
digest `sha256:43a8ed731ac3187868e8d0c021be2a1a1b72986885d13bf3373f2ed5a0296b04`.
It remains responsible for the existing Protocol 1 / repository schema 2
Work Item lifecycle and is not treated as a reader of the new collaboration
records. The candidate Runtime built from this branch is responsible for
collaboration capability discovery, coordination writes/reads, impact
admission, and composition verification.

The existing Contract files remain compatible with `0.2.105`; the new
coordination record has its own strict schema and binds the repository,
Work Item, Contract digest, worktree/head identity, candidate Runtime version,
candidate Runtime digest, and collaboration capability identifier. This is
not a claim of bidirectional wire compatibility. The candidate collaboration
entrypoints must reject an installed/selected Runtime that does not advertise
the capability and must not partially write a registration, event, request,
or receipt. A `0.2.105` query cannot claim collaboration state; it reports
only its existing lifecycle projection, while the candidate surface reports
`unsupported_runtime_capability` with the required candidate identity.

Verification will exercise both sides explicitly: prove ordinary lifecycle
commands still work with `0.2.105`, prove the old Runtime rejects or does not
consume candidate collaboration records rather than silently treating them as
valid, and prove candidate commands fail closed before any write when the
runtime binding/capability is absent or mismatched.

## Domain model

### Work Item collaboration declaration

Additive typed fields describe:

- `providedOutcomes`: stable outcome identity, interface/schema/behavior
  contract, compatibility constraints, and exact published head/evidence.
- `consumedOutcomes`: provider Work Item, required outcome identity, minimum
  stage, and verification requirement.
- `resourceClaims`: exclusive paths, shared generated outputs, and serial
  requirements.
- `integrationResponsibility`: responsible Work Item/role, target branch,
  selected composition order, and rationale.
- `compositionVerification`: machine-checkable compatibility constraints,
  required scenarios, and permitted reusable nodes.

Stages are explicit and independent of lifecycle closure:

`interface_stable`, `composable_head`, and `merged_target`.

The declaration, implementation progress, and verification conclusion remain
separate facts. An Agent's completion message never satisfies a dependency or
creates a receipt.

### Coordination identity and storage

Resolve the Git common directory using Git, never by assuming `.git` is a
directory. Records are partitioned by repository identity and include:

- worktree registration and actual path/branch/HEAD identity;
- Work Item and Contract digest;
- published outcome identity and exact source commit;
- resource reservation and owner generation;
- append-only event identity, source, execution generation, declaration or
  outcome reference, and evidence references;
- coordination request, target WI, target generation, condition, state, and
  acknowledgement.

Writes use temporary files plus atomic rename and an exclusive coordination
lock. Readers validate the complete record against current Git/worktree/
Contract facts. Partial, corrupt, stale, missing, moved, or deleted records
are `unknown`/`recovery_required`, never `free` or `available`.

Resource acquisition orders canonical resource identities, rolls back a
partial multi-resource acquisition, and records the result. Generation
changes invalidate old owners and reject late writes. Timeout only reports an
expired observation; it never grants takeover.

### Events and impact

Events are created when a provider outcome, interface/semantic declaration,
resource ownership, execution status, or verification identity changes.
The domain projection coalesces equivalent consumer invalidations while
retaining the original evidence references. Events are idempotent by stable
identity and cannot overwrite a newer generation merely because their wall
clock time is later.

When cross-process ordering is uncertain, the reader rechecks current facts
and emits an explicit ordering/identity uncertainty instead of applying the
late event.

### Coordination intents

Only these initial intents are supported:

- `wait_for_dependency`: do not start the dependent action, while allowing
  unrelated work to continue;
- `request_safe_pause`: request a pause at a safe boundary and distinguish
  requested, acknowledged, paused, unavailable, and expired states;
- `adjust_integration_order`: update the selected composition order and
  rationale within the integration owner's scope;
- `resume_re_evaluate`: revalidate current facts and admission before
  continuing.

Running tests/writes finish at a safe boundary unless the executor explicitly
supports safe cancellation. A request is bound to the target WI and execution
generation; duplicate or expired requests do not repeat an operation.

Dependency invalidation blocks only affected actions by default. Cycle
diagnostics identify the cycle and the actionable edge rather than allowing
indefinite mutual waiting.

The candidate CLI and MCP expose these as explicit domain operations rather
than implicit status side effects. Read-only operations are `inspect` and
status projection. Mutating operations are registration, impact report,
coordination request, request acknowledgement/safe-pause transition, resume
and re-evaluate, and recovery consumption. Every mutating operation writes a
typed record with atomic identity/deduplication and returns the updated
projection; a read never repairs, consumes, acknowledges, or refreshes a
lease. Before a dependent action or composition attempt, the executor calls
the same admission service to refresh current declarations, events, resource
ownership, and execution generation.

## Composition verification

Each attempt binds:

- target SHA and target branch identity;
- participating WI/head SHAs and fixed order;
- Contract and declaration digests;
- toolchain, command, configuration, lockfile, generated-input, and
  verifier identities;
- actual composition result, conflict result, execution records, and reuse
  decisions.

The verifier uses an isolated temporary worktree and the repository's actual
integration strategy. It performs identity completeness, dependency/resource
and direct interface checks, composition construction, text-conflict checks,
cheap checks, and only then required affected verification. If a prerequisite
fails, the number of expensive verification processes must be zero.

Failures, timeouts, and interruptions are persisted before any retry. A
successful receipt is created only after exit status, logs, and required
outputs are unambiguous. Temporary resources have explicit identities and
recoverable cleanup records.

An old composition result is never reused solely because a file or command is
unchanged. Reuse requires a proof covering source and transitive dependencies,
interfaces, configuration, toolchain, lockfile, generated inputs, relevant
environment, and verifier semantics. Reuse references the original immutable
receipt and never rewrites it.

## Unified projection and human handoff

CLI and MCP call the same repository-domain projection. The projection
contains:

- providers, consumers, waiting edges, and stage reasons;
- new impact and invalidated identities;
- unhandled coordination requests and acknowledgement state;
- integration owner and selected composition order;
- latest exact composition identity and result;
- checks requiring revalidation versus checks proven reusable;
- separate implementation, composition-pass, target-merged, and cleanup
  states;
- blocker, unknown, human-decision, verification, impact, and next-action
  facts.

The human Outcome compresses these facts into a decision-ready handoff and
does not expose an event log as a substitute for interpretation. Read
operations have no side effects; execution operations recalculate admission.

## Compatibility and safety

- Existing Contracts without collaboration fields remain readable.
- Existing single-WI lifecycle commands remain usable with the fixed
  `0.2.105` Runtime and do not scan coordination state; collaboration-aware
  actions require the candidate capability explicitly.
- Ordinary single-WI operations do not scan coordination state or require
  collaboration declarations unless they explicitly opt in.
- Unsupported independent-clone and cross-machine topologies report a bounded
  limitation rather than pretending to coordinate.
- Coordination success, declaration validity, and composition verification do
  not grant push, merge, cleanup, release, or resource-takeover authority.
- Historical Contracts, receipts, archives, and decision records remain
  immutable.

## Verification and acceptance

Acceptance must use real Git operations and multiple linked worktrees, not
same-directory mocks. Required scenario groups are:

1. cross-worktree observation and stage-aware dependency consumption;
2. concurrent conflicting resource acquisition with no double ownership;
3. corrupt/stale/moved/deleted records and crash recovery;
4. generation rejection, duplicate/late events, and request idempotency;
5. local continuation, safe-pause state transitions, and cycle diagnostics;
6. interface errors found by exact composition before merge;
7. target/head/declaration changes invalidating old composition results;
8. safe reuse and conservative revalidation under unknown impact;
9. precondition failure with zero expensive verification processes;
10. persisted failure/timeout/interruption and recoverable cleanup;
11. CLI/MCP parity, repeated read-only queries, Outcome semantics, and
    ordinary-WI compatibility;
12. explicit unsupported-topology reporting.

The measurement report separates cold/warm runs and records first sample,
valid warm samples, elapsed phases, verification process count, file/Git
reads, coordination reads/writes, temporary disk use, and governance cost.
No user benefit is claimed without a measured comparison; missing historical
baselines remain `not_measured`.

## Delivery boundary

The Work Item proceeds through Contract, preflight, implementation, focused
and full verification, finish, archive, commit, push, PR creation, and hosted
CI handoff. It stops before merge/release unless separately authorized. No
release tag, provider Release, publication, or release asset is created by
this design.
