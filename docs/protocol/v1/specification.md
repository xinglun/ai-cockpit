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
└── knowledge/
```

`cockpit.toml` contains the protocol version and repository identity. `project.json`
is the current Living Project Profile. Work Item files contain scoped intent and
outcome. Evidence files are content-addressed receipts or references to delegated
provider evidence. Knowledge is a deterministic projection and never a second
fact source.

## Required identity

Every protocol-bound record includes `protocolVersion`, `repositoryId`,
`repositorySnapshotDigest`, and a `createdAt` timestamp. Runtime-produced evidence
also includes `runtimeVersion` and `runtimeDigest`. Historical records retain the
Project Profile digest used at their decision boundary.

All digests use `sha256:<64 lowercase hexadecimal characters>`. Canonical JSON is
used for digest inputs; map keys are sorted, arrays retain semantic order, and
timestamps are UTC RFC 3339 values.

## Contract envelope

A Contract authorizes an intent and an effect boundary. It records scope,
out-of-scope, risk, authority, acceptance, required evidence, base revision,
project profile digest, and repository snapshot digest. It does not freeze the
number of tests, helper files, class names, or other intermediate implementation
details.

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

An implementation that does not support protocol major version 1 must stop Red.
Runtime upgrades may continue to support protocol 1 without changing repository
files. A protocol-major migration is an explicit, separately reviewed operation.

