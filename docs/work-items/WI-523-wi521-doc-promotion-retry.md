---
author: AI Cockpit maintainers
title: "WI-523 — WI-521 documentation promotion retry"
description: "Redeliver the bounded WI-521 documentation projection after the predecessor pre-merge receipt became stale."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-523-wi521-doc-promotion-retry
lastVerifiedBy: WI-523-wi521-doc-promotion-retry
---

[简体中文](WI-523-wi521-doc-promotion-retry.zh-CN.md) · [日本語](WI-523-wi521-doc-promotion-retry.ja.md)

## Goal

Redeliver the WI-521 terminal documentation projection from the latest reviewed
default branch, while preserving WI-522's immutable archive and recovery record.

## Scope

- Mark the immutable WI-522 predecessor as recovered and link its recovery and successor.
- Promote the WI-521 and WI-523 reader-facing pages and all three parity projections.
- Keep Runtime-generated evidence, predecessor bytes, object repositories, and global configuration unchanged.

## Acceptance

- WI-522 remains explicitly recovered; its stale finalization is not presented as a success.
- The three WI-523 pages and parity rows bind exact archive, verification, finalization, and close evidence after terminalization.
- Documentation, parity, status-consistency, and governance-integrity checks pass on the final archive commit.
- The pre-merge finalization receipt is created only after archive and its head equals the reviewed PR head.
- No predecessor evidence or object-repository file is modified.

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/docs/work_item_status_consistency.py --repo <repo>
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
git diff --check
```
