---
author: AI Cockpit maintainers
title: "WI-597——WI-596 终态文档晋级"
description: "把已关闭的 WI-596 发布事实晋级到三语文档投影。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-597-doc-promotion-wi596
lastVerifiedBy: WI-597-doc-promotion-wi596
terminalArchive: .ai/work-items/archive/WI-597-doc-promotion-wi596.contract.json
terminalVerification: .ai/evidence/WI-597-doc-promotion-wi596.verification.json
terminalFinalization: .ai/decisions/WI-597-doc-promotion-wi596.finalize.34b2d27066299df9fed65741230bb4bc3bd9285e005610c6348f6dcc09f9f6eb.json
terminalDecision: .ai/decisions/WI-597-doc-promotion-wi596.close.json
---

[English](WI-597-doc-promotion-wi596.md) · [日本語](WI-597-doc-promotion-wi596.ja.md)

## 目标

在不改变不可变治理记录和 Runtime 行为的前提下，将已关闭 WI-596 的发布与 parity 事实晋级到三语文档投影。

## 边界

本 Work Item 只修改文档投影和 pending parity bridge；不修改 Runtime 代码、发布字节、对象工程或历史 evidence。

## 验收

- WI-596 三语页面和 parity 行链接准确的 archive、verification、finalization、close 证据。
- WI-597 在自身审查关闭前拥有三语阅读页面和可审计的进行中 parity bridge。
- 文档、parity 和治理完整性检查通过。

## 验证

```text
python3 tests/docs/promote_closed_work_item.py --repo . --work-item WI-596-release-v0-2-78
bash tests/docs/documentation_acceptance.sh
python3 tests/ci/governance_integrity_gate.py --repo . --report target/wi597-governance-report.json
```
