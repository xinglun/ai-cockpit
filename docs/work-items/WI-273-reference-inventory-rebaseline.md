---
author: AI Cockpit maintainers
title: "WI-273 — reference inventory rebaseline"
workItemId: WI-273-reference-inventory-rebaseline
description: "Rebind the file-level reference comparison ledger to the reviewed current default-branch commit without changing Runtime behavior."
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-273-reference-inventory-rebaseline
terminalArchive: .ai/work-items/archive/WI-273-reference-inventory-rebaseline.contract.json
terminalVerification: .ai/evidence/WI-273-reference-inventory-rebaseline.verification.json
terminalFinalization: .ai/decisions/WI-273-reference-inventory-rebaseline.finalize.json
terminalDecision: .ai/decisions/WI-273-reference-inventory-rebaseline.close.json
authority: canonical
---

# WI-273 — reference inventory rebaseline

## Intent

Rebaseline the file-by-file reference comparison ledger and its reader-facing
documentation to the reviewed `origin/main` commit `487f019` before starting
the next semantic comparison batch. This is metadata and documentation truth,
not a Runtime feature change.

## Scope

- Update the inventory target commit and derived tracked/working-tree metadata.
- Preserve all existing classifications, including WI-270/WI-272 records and
  the four explicit capability/profile migrate gaps.
- Keep deferred paths deferred; a metadata refresh must not close semantic work.
- Synchronize the English, Simplified Chinese, and Japanese comparison and
  parity documentation.
- Keep historical `docs/work-items/**` and generated evidence immutable.

## Boundary

This Work Item does not modify Rust Runtime behavior, CI workflow behavior,
Agent/MCP global configuration, or the reference source. It does not promote
any deferred path and does not rewrite archived Work Item evidence. Generated
archive, verification, and decision records are produced by the installed
Runtime rather than hand-edited.

## Acceptance

- Inventory target commit, tracked/working-tree counts, and path digests match
  clean `origin/main` `487f01970c49e2b85d17b0cb0536f9d60c8f05e0`.
- The ledger contains 5,119 records: 4,262 generated-history, 163
  implemented-different-by-design, one implemented-equivalent, 689
  deferred-next-batch, and four migrate-gap records.
- Generator and regression checks reject a stale target revision and validate
  the current metadata.
- All three comparison documents and parity ledgers use the same baseline and
  counts.
- Documentation, inventory, governance, and full required checks pass without
  a Runtime business-behavior change.

## Verification

- Installed Runtime with an explicit `--repo` for every repository-bound call.
- `bash tests/conformance/reference_file_inventory_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo . --report <report>`
- Full workspace quality and hosted checks required by the Contract.

## Terminal evidence

The installed Runtime owns the terminal paths recorded in the Contract:

- Archive: `.ai/work-items/archive/WI-273-reference-inventory-rebaseline.contract.json`
- Verification: `.ai/evidence/WI-273-reference-inventory-rebaseline.verification.json`
- Finalization: `.ai/decisions/WI-273-reference-inventory-rebaseline.finalize.json`
- Close: `.ai/decisions/WI-273-reference-inventory-rebaseline.close.json`
