---
author: AI Cockpit maintainers
title: "WI-525 — WI-524 terminal documentation promotion"
description: "Promote the closed WI-524 documentation projections with exact terminal evidence bindings."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-525-wi524-doc-promotion
lastVerifiedBy: WI-525-wi524-doc-promotion
---

[简体中文](WI-525-wi524-doc-promotion.zh-CN.md) · [日本語](WI-525-wi524-doc-promotion.ja.md)

## Goal

Synchronize the three-language WI-524 Work Item pages and parity rows with the
immutable archive, verification, resource-finalization, and close evidence.

## Scope

- Promote the WI-524 pages and all three reference-parity projections.
- Keep historical evidence bytes, Runtime behavior, object repositories, and
  global Agent/MCP configuration unchanged.
- Keep this projection auditable after terminal close.

## Acceptance

- Every WI-524 page and parity row binds the exact terminal evidence paths.
- The closed Work Item promotion, documentation, and governance gates pass.
- No object-repository state or historical evidence is modified.

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/docs/work_item_status_consistency.py --repo <repo>
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
git diff --check
```
