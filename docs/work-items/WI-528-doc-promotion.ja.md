---
author: AI Cockpit maintainers
title: "WI-528 — WI-526 terminal documentation promotion"
description: "Runtime の証拠を改変せず、release Work Item の文書投影を昇格する。"
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

[English](WI-528-doc-promotion.md) · [简体中文](WI-528-doc-promotion.zh-CN.md)

## Goal

WI-526 の release ページと parity 投影を、immutable な archive、verification、
finalization、close 記録に同期します。

## Scope

- WI-526 の reader-facing ページと reference-parity 行を昇格します。
- Runtime が生成した記録と adopter repository は変更しません。

## Acceptance

- WI-526 ページと parity 行が正確な terminal evidence path を参照します。
- documentation と governance integrity のチェックが成功します。

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
bash tests/docs/documentation_acceptance.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
```
