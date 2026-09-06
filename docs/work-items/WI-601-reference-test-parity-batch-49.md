---
title: "WI-601 — reference test parity batch 49"
description: "Compare the next ten maintained reference test paths without copying source implementation or wire formats."
author: AI Cockpit maintainers
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
workItemId: WI-601-reference-test-parity-batch-49
lastVerifiedBy: WI-601-reference-test-parity-batch-49
terminalArchive: .ai/work-items/archive/WI-601-reference-test-parity-batch-49.contract.json
terminalVerification: .ai/evidence/WI-601-reference-test-parity-batch-49.verification.json
terminalFinalization: .ai/decisions/WI-601-reference-test-parity-batch-49.finalize.json
terminalDecision: .ai/decisions/WI-601-reference-test-parity-batch-49.close.json
---

# WI-601 — reference test parity batch 49

[简体中文](WI-601-reference-test-parity-batch-49.zh-CN.md) · [日本語](WI-601-reference-test-parity-batch-49.ja.md)

## Intent and boundary

Re-read the next ten maintained files from the pinned local reference
checkout, one file at a time. Carry portable governance semantics to the Rust
Runtime or repository-native gates, while retaining source/provider fixtures,
Dependabot intake, and deprecated-asset registry behavior as bounded
`reference-only` responsibilities.

This is semantic parity, not source command, Python module, or JSON-wire
compatibility. It does not modify the reference checkout, object repositories,
global Agent/MCP configuration, or immutable historical evidence.

## Bounded result

The ten paths are recorded in `tests/conformance/reference_file_inventory.json`
under `WI-601-reference-test-parity-batch-49`:

- Seven are `implemented-different-by-design`, backed by existing typed
  Contract, profile, lifecycle, trust, CI, and documentation boundaries.
- Three are `reference-only`: the source seven-stack long-cycle fixture,
  Dependabot intake, and deprecated-assets registry are source/provider
  boundaries, not missing Runtime controls.

No `migrate-gap` was found. The tri-language ledger, parity pages, metadata
sidecar, regression wrapper, and this record are updated together; the
append-only ledger and source history are not rewritten.

## Acceptance and verification

- Every selected path has exactly one classification, counterpart set, and
  bounded reason.
- Any confirmed portable omission is fixed within this Work Item rather than
  silently deferred or hidden in a successor.
- Inventory, regression scripts, metadata, tri-language comparison/parity
  pages, and this record agree.
- Conformance, documentation, governance-integrity, and locked workspace
  checks pass before finish.

The next comparison batch starts only after reviewed release, exact cleanup,
and visible human Outcome. Attached object/adopter repositories inherit the
shared Runtime and its repository-bound isolation; source Python/Make modules,
provider policy values, stack matrices, and source wire formats do not cross
that boundary.
