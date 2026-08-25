---
author: AI Cockpit maintainers
title: CI Contract-aware quality gates
description: The dynamic CI route and its Rust-native Contract gate.
audience:
  - contributor
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-291-ci-contract-aware-gates
---

# CI Contract-aware quality gates

AI Cockpit uses a resource-aware quality route. The route is dynamic, not a
promise that every change always runs the most expensive profile:

- `light` is the focused route for documentation-only changes;
- `standard` adds source/test and workspace checks;
- `strict` is required for governance, workflow, release-owned, high-risk, and
  unknown surfaces.

The stage floor and risk escalation can raise a profile. A requested profile
can only raise the automatic result; it can never lower it. The canonical
manifest remains the list of commands, while the route receipt binds the
manifest, Git base/head, changed paths, Contract path/digest, and ordered gate
IDs.

## Rust authority and Python shadow

For a standard or strict pull request with an active Contract, CI runs the
read-only Rust gate before repository commands:

```text
Python route/manifest plan
        ↓
Rust Contract gate (authority, no .ai writes)
        ↓
Python gate runner and Cargo/static checks (shadow comparison)
```

The Rust gate validates the regular Contract file, repository identity, base
revision, current snapshot, typed Contract invariants, policy-bound
intent/scenario/operation/stage route, and the current Agent-Risk/preflight
projection. It emits `repository_contract_quality_gate` JSON with a stable
receipt digest, decision state, verification tier, and evidence assurance.
Yellow or red output exits non-zero and cannot authorize repository commands.

The Python route and runner remain during this convergence phase. A future
batch may remove duplicated policy only after hosted shadow comparisons prove
semantic agreement. This gate does not implement the reference source's full
workflow matrix, dependency planner, or release-preflight sequence.

## Evidence and release boundary

The CI gate is a source-built check for the reviewed change. Its Runtime
identity is recorded for diagnosis, but it is not a public Release artifact.
Release and adopter acceptance still require an immutable downloaded tag,
archive/binary checksums, SBOM/provenance, and the published-artifact harness.
The CI gate never writes `.ai/` Contract, Summary, checkpoint, verification,
or decision records; lifecycle commands remain the only authority for those
mutable records.

The same boundary applies to an object project: the shared Runtime is external,
every request carries an explicit `--repo`, and repository evidence is isolated.
