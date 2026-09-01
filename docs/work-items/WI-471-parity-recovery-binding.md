---
author: AI Cockpit maintainers
title: "WI-471 — parity recovery binding"
description: "Bind the authoritative WI-469 recovery receipt in every reference-parity ledger before release."
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-471-parity-recovery-binding
status: recovered
authority: authorized
lastVerifiedBy: WI-471-parity-recovery-binding
---

# WI-471 — parity recovery binding

## Intent and boundary

The post-close governance-integrity gate identified that the WI-469 rows in
the three parity ledgers listed the normal close path but not the validated
digest-suffixed recovery receipt selected by the Runtime as the authoritative
terminal projection. This Work Item makes that binding explicit. It does not
rewrite historical bytes or change Runtime behavior.

## Scope

- Update the English, Simplified Chinese, and Japanese reference-parity rows
  for WI-469 with the exact authoritative recovery receipt path.
- Keep all existing archive, verification, finalization, and close references.
- Record the same boundary in this Work Item's three language pages.

## Acceptance

1. All three WI-469 rows include the validated recovery receipt and all
   terminal lifecycle references.
2. `tests/ci/governance_integrity_gate.py` reports zero findings.
3. Historical archive, evidence, recovery, close, and source bytes are not
   rewritten.
4. The three Work Item pages remain status-consistent after closure.

## Verification

- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `bash tests/docs/parity_status_check.sh .`
- `bash tests/docs/documentation_acceptance.sh`

## Recovery boundary

The recovery receipt is an immutable recovery projection selected by the
Runtime so this Work Item continues through its explicit successor. Listing it
in the parity row does not reclassify or rewrite the predecessor; it makes the
recovery path auditable while WI-472 owns the bounded redelivery.
