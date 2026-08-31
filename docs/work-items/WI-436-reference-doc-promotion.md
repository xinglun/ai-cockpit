---
author: AI Cockpit maintainers
title: "WI-436 — closed documentation projection promotion"
workItemId: WI-436-reference-doc-promotion
description: "Promote the tri-language documentation projections after WI-435 close."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-436-reference-doc-promotion
terminalArchive: .ai/work-items/archive/WI-436-reference-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-436-reference-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-436-reference-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-436-reference-doc-promotion.close.json
---

# WI-436 — closed documentation projection promotion

This documentation-only Work Item applies the repository-owned closed Work
Item promotion helper to WI-435. It keeps the maintained local checkout at
`/Users/sei-rinn/dev/workspace_python/ai-cockpit-template` as the semantic
reference and does not access the public reference repository or change
Runtime behavior.

[简体中文](WI-436-reference-doc-promotion.zh-CN.md) · [日本語](WI-436-reference-doc-promotion.ja.md)

## Scope

- Promote WI-435's three Work Item documents and three reference-parity rows.
- Record only immutable archive, verification, finalization, and close paths.
- Keep all other Work Items and historical bytes unchanged.

## Verification

`tests/docs/promote_closed_work_item.py --work-item WI-435-reference-inventory-rebaseline-local`
and `--check-all`, documentation acceptance, parity status, and diff checks
must all pass.
