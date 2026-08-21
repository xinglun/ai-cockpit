---
author: AI Cockpit maintainers
title: "Repository Protocol v1"
description: "Normative repository-owned storage, identity, receipt, and decision contract."
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - protocol_v1
---

# Repository Protocol v1

Repository Protocol v1 is the stable, repository-owned storage boundary between
an application repository and an external AI Cockpit runtime. It stores facts,
decisions, evidence, and generated knowledge; it does not install the runtime.

## Layout

```text
.ai/
├── cockpit.toml
├── project.json
├── work-items/
│   ├── active/
│   └── archive/
├── decisions/
├── evidence/
│   ├── <work-item>.verification.json
│   └── reuse/
│       ├── index.lock
│       ├── index.json
│       └── receipts/<64-lowercase-hex>.json
└── knowledge/
```

`cockpit.toml` contains the protocol version and repository identity. `project.json`
is the attached Living Project Profile. Work Item files contain scoped intent,
contract, summary, and outcome. Verification evidence is recorded under
`.ai/evidence`; cross-process reusable receipts are content-addressed under the
`reuse` store. Knowledge is a deterministic projection and never a second fact source.

The reuse index is schema version 1 and binds a `repositoryId`, a `profileDigest`,
and a map of `nodeId` to receipt ID. The receipt filename uses the lowercase hex
part of its canonical `sha256:<64 hex>` ID so it is portable across platforms.
The index is committed through `index.pending` while writers hold `index.lock`;
readers reject an uncertain, malformed, oversized, symlinked, or inconsistent store.
Runtime-managed store files are not hand-edited by adopters.

## Identity-bearing records

Identity fields are required where a Contract, verification evidence, archive
manifest, or reusable receipt needs to bind a decision to repository state. A
Contract records:

| Field | Meaning |
| --- | --- |
| `protocolVersion` | Protocol major understood by the runtime. |
| `repositoryId` | Stable identity derived for the target repository. |
| `repositorySnapshotDigest` | The observed repository state used for the decision. |
| `baseRevision` / `headCommit` | The source range used by the decision when available. |
| `projectProfileDigest` | The attached/calibrated profile used for authorization. |
| `createdAt` | UTC RFC 3339 creation time. |

Runtime-produced verification evidence additionally records runtime version and
digest, command results, output identity, reuse metrics, and the final snapshot.
Other records, such as knowledge projections and human decision receipts, have
their own schemas and do not implicitly contain every field in this table.

All digests use `sha256:<64 lowercase hexadecimal characters>`. Canonical JSON is
used for digest inputs: map keys are sorted, arrays retain semantic order, and
timestamps are UTC RFC 3339 values.

## Reusable receipt schema

Reusable receipts use schema version 2 and reject unknown fields. The stable fields are
`receiptId`, `nodeId`, `passed`, `outputDigest`, creation/expiry epoch seconds, and
an `EvidenceContext`. The context binds content, base/head and changed-path digests,
environment, command, scope, governance, toolchain, policy, profile, stage, and runner.
The receipt ID is the digest of the canonical receipt body; tampering, a failed or
expired receipt, a future timestamp, or any binding mismatch makes the candidate
`unknown` and causes execution.

The store bounds index reads to 8 MiB and reusable receipt reads to 1 MiB. These
limits are part of the fail-closed resource boundary, not a promise that arbitrary
large output is retained.

## Contract envelope

A Contract authorizes an intent and an effect boundary. It records scope,
out-of-scope, risk, authority, acceptance, required evidence, base revision,
project profile digest, and repository snapshot digest. It does not freeze the
number of tests, helper files, class names, or other intermediate implementation details.

## Decision states

- `green`: required evidence supports the bounded next action;
- `yellow`: evidence or capability needs investigation or human confirmation;
- `red`: a required control failed, authority is absent, or the state is invalid.

`unknown` evidence is never interpreted as a pass. Human decisions are recorded
as decisions and do not replace independent verification evidence.

## Evolution

- L0 content evolution is automatically absorbed.
- L1 verification evolution expands the existing verification graph.
- L2 capability evolution creates a Yellow candidate and a Profile proposal.
- L3 governance evolution requires a human decision and never becomes mandatory
  without explicit confirmation.

## Compatibility

The current runtime accepts protocol major version 1 and rejects malformed or
unsupported versions before executing repository material. Required fields are
validated by the operation that consumes the record. Optional capabilities are
not silently upgraded or converted into a pass; an unsupported request remains an
explicit error, unknown, or stop condition. A protocol-major migration is a
separately reviewed operation that preserves old evidence.
