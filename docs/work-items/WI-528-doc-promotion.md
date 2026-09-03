---
author: AI Cockpit maintainers
title: "WI-528 — WI-526 terminal documentation promotion"
description: "Promote the release Work Item documentation projections without rewriting Runtime evidence."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-528-doc-promotion
lastVerifiedBy: WI-528-doc-promotion
terminalArchive: .ai/work-items/archive/WI-528-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-528-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-528-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-528-doc-promotion.close.json
---

[简体中文](WI-528-doc-promotion.zh-CN.md) · [日本語](WI-528-doc-promotion.ja.md)

## Goal

Keep the three-language WI-526 release pages and parity projections synchronized
with the immutable WI-526 archive, verification, finalization, and close records.

## Scope

- Promote the WI-526 reader-facing pages and reference-parity rows.
- Preserve all Runtime-generated records and object repositories.

## Acceptance

- The WI-526 pages and parity rows bind the exact terminal evidence paths.
- Documentation and governance integrity checks pass.

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
bash tests/docs/documentation_acceptance.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
```
