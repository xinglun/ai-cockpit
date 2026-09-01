---
author: AI Cockpit maintainers
title: "WI-468 — reference ledger parity promotion"
description: "Redeliver WI-467's manifest-derived ledger projection with the required tri-language parity registration."
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-468-reference-ledger-parity-promotion
predecessorWorkItemId: WI-467-reference-ledger-projection
status: recovered
authority: authorized
lastVerifiedBy: WI-468-reference-ledger-parity-promotion
---

# WI-468 — reference ledger parity promotion

## Intent and boundary

WI-468 is the explicit successor to immutable WI-467. The predecessor's
Contract, evidence, Outcome, archive, and recovery receipts remain unchanged.
This Work Item promotes the same bounded manifest-derived current snapshot and
registers it in the English, Chinese, and Japanese reference-parity ledgers so
the repository governance gate can verify documentation truth before merge.

## Scope and acceptance

- Keep the three comparison pages derived from the canonical inventory manifest.
- Add one matching WI-468 row to all three reference-parity pages.
- Keep historical sections and predecessor bytes immutable.
- Make the documentation gate fail closed when the current counts or parity
  registration diverge.

The source checkout remains a local semantic reference, not a runtime or wire
format dependency. Generated archive, evidence, and decision records are owned
by the Runtime and are never hand-edited.

## Verification

Use the installed Runtime with an explicit repository path, then run the
documentation, conformance, and workspace gates declared by the Contract.

## Links

[简体中文](WI-468-reference-ledger-parity-promotion.zh-CN.md) ·
[日本語](WI-468-reference-ledger-parity-promotion.ja.md)
