---
author: AI Cockpit maintainers
title: "WI-597 — WI-596 terminal documentation promotion"
description: "Promote the closed WI-596 release facts into the tri-language documentation projections."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-597-doc-promotion-wi596
lastVerifiedBy: WI-597-doc-promotion-wi596
---

[简体中文](WI-597-doc-promotion-wi596.zh-CN.md) · [日本語](WI-597-doc-promotion-wi596.ja.md)

## Objective

Promote the already closed WI-596 release and parity facts without changing immutable governance records or Runtime behavior.

## Boundary

This Work Item changes only documentation projections and the pending parity bridge. It does not modify Runtime code, release bytes, object repositories, or historical evidence.

## Acceptance

- The WI-596 English, Chinese, and Japanese pages and parity rows link the exact archive, verification, finalization, and close evidence.
- WI-597 has matching reader pages and an auditable in-progress parity bridge until its own reviewed close.
- Documentation, parity, and governance-integrity checks pass.

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --repo . --work-item WI-596-release-v0-2-78
bash tests/docs/documentation_acceptance.sh
python3 tests/ci/governance_integrity_gate.py --repo . --report target/wi597-governance-report.json
```
