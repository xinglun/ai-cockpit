---
author: AI Cockpit maintainers
title: "WI-765 — WI-764 closed documentation promotion"
description: "Promote the closed WI-764 release-boundary documentation without changing its historical governance evidence."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human:repository-owner
workItemId: WI-765-wi764-doc-promotion
lastVerifiedBy: WI-765-wi764-doc-promotion
terminalArchive: .ai/work-items/archive/WI-765-wi764-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-765-wi764-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-765-wi764-doc-promotion.finalize.60a5e15353209ac22e995c11894712c8181b817a706fad9322d5a1712fec21d5.json
terminalDecision: .ai/decisions/WI-765-wi764-doc-promotion.close.json
---

[简体中文](WI-765-wi764-doc-promotion.zh-CN.md) · [日本語](WI-765-wi764-doc-promotion.ja.md)

# WI-765 — WI-764 closed documentation promotion

## Intent

Promote the verified closed WI-764 release-boundary projections to their
terminal documentation state while preserving the failed-release tag and all
historical governance evidence.

## Boundary

This Work Item changes only the WI-764 documentation projections, the
reference-parity ledger, and its own governance documentation. It does not
change Runtime behavior, release workflow behavior, version metadata, or any
historical WI-764 archive, evidence, finalization, cleanup, or close bytes.

## Verification

The Runtime verification evidence and
`python3 tests/docs/promote_closed_work_item.py --check-all` must pass. The
pre-archive parity rows are registered before hosted verification; terminal
links are projected only after this Work Item is verified and closed.
