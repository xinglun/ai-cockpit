# Cross-Work-Item coordination

This capability is candidate-Runtime-only. Runtime `0.2.105` remains the
lifecycle owner and is not a reader of the coordination records. The candidate
Runtime owns collaboration capability discovery, repository-local coordination
writes/reads, impact admission, and exact composition verification. This is not
a bidirectional compatibility promise.

## Supported boundary

The store is resolved from Git's common directory, so linked worktrees in one
repository can coordinate. Independent clones and cross-machine coordination
are unsupported. Records live under `.ai-cockpit/coordination/v1/` and bind the
repository, Work Item, Contract, worktree/head, Runtime capability, and
execution generation.

Registration and inspection re-observe those bindings from the canonical Git
topology and active Contract; a caller-provided repository id, branch, head,
Contract digest, or evidence path is not authoritative by itself. A head or
Contract/declaration change on a later registration appends a deduplicated
Impact event. Published outcomes are informational and do not invalidate a
consumer; invalidation recovery records the current provider generation/head
and Contract digest, so an old event can be resolved after the provider has
advanced without deleting history.

Ordinary single-WI lifecycle commands remain the fast path. They do not scan or
write the collaboration store unless the Work Item explicitly participates.

## Commands and write boundary

Read-only inspection:

```text
ai-cockpit work-item coordination inspect --repo <path>
```

Explicit writes use JSON records and return the updated projection:

```text
ai-cockpit work-item coordination register --repo <path> --input registration.json
ai-cockpit work-item coordination report-impact --repo <path> --input event.json
ai-cockpit work-item coordination publish-outcome --repo <path> --id <wi> --generation <n> --outcome-id <outcome>
ai-cockpit work-item coordination request-pause --repo <path> --input request.json
ai-cockpit work-item coordination acknowledge --repo <path> --request-id <id> --state acknowledged
ai-cockpit work-item coordination resume --repo <path> --id <wi> --generation <n>
ai-cockpit work-item coordination recover --repo <path> --event-id <id> --consumer-work-item-id <wi> --consumer-generation <n>
ai-cockpit work-item composition --repo <path> --id <wi> --generation <n> --input composition.json
```

Inspection never repairs, consumes, acknowledges, refreshes a lease, or
creates the coordination directory. Recovery consumption is a write: it is
idempotent for the exact event/provider generation/consumer generation tuple,
and rejects stale generations.

Impact events are deduplicated by event identity. An impact blocks only
affected consumers; unrelated Work Items continue. Pause requests are distinct
from acknowledgement, safe pause, unavailable/expired, and resume. A request
from an older execution generation cannot control a newer one.

Publishing an outcome is an explicit write for the current registration
generation and a declared outcome. Runtime validates any verification receipt
required by current consumers and appends an `OutcomePublished` event bound to
the exact referenced evidence bytes. A read-only query never publishes, repairs,
or consumes an event; changing the evidence after publication breaks the byte
binding and cannot satisfy a verification dependency.

### MCP identity fields

The `work_item_coordination` schema constrains fields by action. `publish-outcome`
uses `providerWorkItemId`, `providerGeneration`, and `outcomeId`; the older
`workItemId`/`generation` pair remains accepted only as a legacy alias for that
action. `resume` uses `workItemId` and `generation` for the Work Item being
resumed. `recover` uses `eventId` plus `consumerWorkItemId` and
`consumerGeneration`; the immutable event identifies its provider. Extra or
mixed identity fields are rejected.

```json
{"action":"publish-outcome","providerWorkItemId":"WI-PROVIDER","providerGeneration":3,"outcomeId":"api"}
{"action":"recover","eventId":"impact-1","consumerWorkItemId":"WI-CONSUMER","consumerGeneration":2}
```

## Composition verification

Composition first refreshes dependency admission and rejects a safely paused
target before any verification process starts. The Runtime then verifies the
target topology, every registered participant head/Contract, required check
coverage, and preconditions before building the declared participant order in a
temporary linked worktree. It uses the shared bounded verifier for finite
timeouts and bounded output, persists an in-progress attempt before spawning,
persists every node, and records timeout/interruption-safe state and cleanup
results. Caller-supplied identity digests do not authorize reuse: Runtime
observes the target tree, lock and configuration files, resolved executables,
effective environment, and declared input files. A node is reusable only when
its observed executable, command, effective environment, declared input-file
bytes, and upstream receipts match a successful predecessor; missing or
unobservable node inputs disable reuse. An exact repeat can therefore spawn
zero processes, while a changed declared input reruns its node and dependent
nodes.

The CLI and MCP expose the same repository service. Outcome output keeps
implementation, temporary composition, current composition applicability,
actual target merge, and cleanup separate; historical passes remain visible
but become stale when their bound target or participant facts move. An
unproven benefit or missing comparison remains explicit rather than becoming a
performance claim.
