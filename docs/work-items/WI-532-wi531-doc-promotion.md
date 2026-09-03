---
author: AI Cockpit maintainers
title: "WI-532 — WI-531 terminal documentation promotion"
description: "Promote WI-531 reader documentation after its verified close."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-532-wi531-doc-promotion
lastVerifiedBy: WI-532-wi531-doc-promotion
---

[简体中文](WI-532-wi531-doc-promotion.zh-CN.md) · [日本語](WI-532-wi531-doc-promotion.ja.md)

## Goal

Synchronize the three-language WI-531 pages and parity rows with immutable
archive, verification, finalization, and close evidence.

## Scope

- Run the official closed-Work-Item promotion helper for WI-531.
- Keep Runtime records, source behavior, and object repositories unchanged.

## Acceptance

- WI-531 pages and parity rows bind exact terminal evidence.
- Documentation and governance integrity checks pass.

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-531-historical-direct-merge-apply
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/ci/governance_integrity_gate.py --repo <repo>
```
