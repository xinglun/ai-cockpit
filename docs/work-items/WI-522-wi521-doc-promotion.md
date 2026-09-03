---
author: AI Cockpit maintainers
title: "WI-522 — WI-521 terminal documentation promotion"
description: "Promote WI-521 documentation projections after its verified close without rewriting immutable Runtime records."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-522-wi521-doc-promotion
lastVerifiedBy: WI-522-wi521-doc-promotion
---

[简体中文](WI-522-wi521-doc-promotion.zh-CN.md) · [日本語](WI-522-wi521-doc-promotion.ja.md)

## Goal

Promote the closed WI-521 reader-facing pages and parity rows to the exact
terminal truth already recorded by the Runtime.

## Scope

- The three WI-521 Work Item pages.
- The three `docs/reference/reference-parity` projections.
- These three tri-language WI-522 records.

Runtime source, the reference implementation, object repositories, release
publication, global Agent/MCP configuration, and generated WI-521 records are
out of scope.

## Acceptance

- `promote_closed_work_item.py --check-all` reports no stale WI-521 projection.
- All WI-521 pages and parity rows show implemented status and exact terminal
  archive, verification, finalization, and close evidence links.
- Documentation, parity, status-consistency, and governance-integrity checks
  pass without changing immutable Runtime records.

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/docs/work_item_status_consistency.py --repo <repo>
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
git diff --check
```
