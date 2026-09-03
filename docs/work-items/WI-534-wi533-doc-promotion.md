---
author: AI Cockpit maintainers
title: "WI-534 — WI-533 terminal documentation promotion"
description: "Promote WI-533 reader documentation after its verified close."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-534-wi533-doc-promotion
lastVerifiedBy: WI-534-wi533-doc-promotion
---

[简体中文](WI-534-wi533-doc-promotion.zh-CN.md) · [日本語](WI-534-wi533-doc-promotion.ja.md)

## Goal

Synchronize the three-language WI-533 pages and parity rows with immutable
archive, verification, finalization, and close evidence.

## Scope

- Run the official closed-Work-Item promotion helper for WI-533.
- Keep Runtime records, source behavior, release artifacts, and the object
  repository unchanged.

## Acceptance

- WI-533 pages and parity rows bind exact terminal evidence.
- Documentation and governance integrity checks pass.

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-533-release-v0-2-66
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
tests/docs/documentation_acceptance.sh
tests/docs/parity_status_check.sh
```
