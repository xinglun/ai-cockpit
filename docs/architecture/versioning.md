---
author: AI Cockpit maintainers
title: "Versioning"
description: "Runtime and Repository Protocol version identity and migration boundary."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - versioning
---

# Versioning

## Adjacent migration chain

Repository schema migration is an explicit chain of reviewed adjacent edges.
The Runtime resolves the next edge from the current schema and refuses an
unknown source, a future schema, or a direct jump over an unreviewed
intermediate version. Each approved step writes a Runtime-bound receipt with
the step identity, chain length, preserved historical-evidence digest, and
Runtime version/digest. Historical evidence, decisions, knowledge, and
archived Work Items are byte-preserved; they are never rewritten by migration.

Runtime version, Repository Protocol version, and the repository schema version
are independent identities.

```text
ai-cockpit --version
0.2.21

repository:
protocol_version = 1
repository_schema_version = 2
```

The CLI version identifies the executable package. Protocol version identifies
the repository storage contract. Runtime version, runtime digest, and protocol
version are exposed together on identity-bearing surfaces such as `inspect`,
`doctor`, MCP `initialize`, and verification evidence; `--version` alone is a
short package-version command and does not promise the full identity envelope.

A Runtime-only upgrade keeps the repository's `.ai/` bytes unchanged when the
compatibility report is `COMPATIBLE`. Runtime identity is recorded in new
verification and migration receipts, but the Runtime has no global active
repository or Work Item state.

The current Repository Protocol remains Protocol 1 and the attached repository
schema target is 2. An older schema is not silently rewritten. Inspect the
boundary first:

```bash
ai-cockpit compatibility --repo /path/to/repository
ai-cockpit migrate plan --repo /path/to/repository
ai-cockpit migrate apply --repo /path/to/repository --approved
```

`COMPATIBLE` permits normal lifecycle commands. `MIGRATION_REQUIRED` permits
inspection and a read-only plan, but lifecycle, Agent, MCP, and verification
operations stop until a human reviews and approves the explicit migration.
`INCOMPATIBLE` is a fail-closed stop requiring a Runtime that understands the
stored schema. A migration receipt binds the from/to schema, before/after
digests, runtime version, and runtime digest. Work Items, evidence, decisions,
knowledge, and archived history are never rewritten by this migration.

Historical Work Items retain the Project Profile digest and protocol version
used at their decision boundary. A major migration is a separately reviewed
Work Item that preserves old evidence.
