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

## Composition verification

Composition first refreshes dependency admission and rejects a safely paused
target before any verification process starts. The Runtime then verifies the
target topology, every registered participant head/Contract, unique required
check coverage, and computes the preconditions before building the participant
order in a temporary linked worktree. It uses the shared bounded verifier for
finite timeouts and bounded output, persists an in-progress attempt before
spawning, persists every node, and records timeout/interruption-safe state and
cleanup results. A previous receipt is reusable per node only when all bound
source, dependency, interface, configuration, toolchain, lockfile,
generated-input, environment, verifier, and command identities match and the
predecessor node passed; an exact repeat can therefore spawn zero processes and
a local change reruns only its affected node.

The CLI and MCP expose the same repository service. Outcome output keeps
implementation, composition, target merge, and cleanup states separate; an
unproven benefit or missing comparison remains explicit rather than becoming a
performance claim.
