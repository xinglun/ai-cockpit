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

Composition first refreshes dependency admission, then builds the declared
participant order in a temporary linked worktree. It records text conflicts,
interface checks, command output, exit status, and cleanup. A failed
precondition starts zero expensive verification processes. A previous receipt is
reusable only when all bound source, dependency, interface, configuration,
toolchain, lockfile, generated-input, environment, verifier, and command
identities match and the predecessor passed unambiguously.

The CLI and MCP expose the same repository service. Outcome output keeps
implementation, composition, target merge, and cleanup states separate; an
unproven benefit or missing comparison remains explicit rather than becoming a
performance claim.
