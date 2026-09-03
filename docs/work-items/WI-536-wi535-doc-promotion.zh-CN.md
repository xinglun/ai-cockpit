---
author: AI Cockpit maintainers
title: "WI-536——WI-535 终态文档晋级"
description: "晋级 WI-535 读者文档，并在归档前登记本 Work Item 自身。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-536-wi535-doc-promotion
lastVerifiedBy: WI-536-wi535-doc-promotion
---

[English](WI-536-wi535-doc-promotion.md) · [日本語](WI-536-wi535-doc-promotion.ja.md)

## 目标

使 WI-535 三语页面与不可变的终态 evidence 同步，并在验证和归档前将本 Work
Item 登记到全部 Parity 台账。

## 范围与边界

- WI-535 和 WI-536 的三语读者页面。
- 英语、日语和简体中文 Parity 台账。
- Runtime 行为、生成的 `.ai` 记录、发布 artifact 和对象工程不在本 Work Item 范围内。

## 验收

- WI-535 页面和 Parity 行绑定准确的终态 evidence。
- WI-536 在验证和归档前登记到三份 Parity 台账。
- 文档、Parity 与治理完整性检查通过。

## 验证

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-535-mcp-fixture-cleanup
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
tests/docs/documentation_acceptance.sh
tests/docs/parity_status_check.sh
tests/ci/governance_integrity_gate.py --repo <repo>
```
