---
author: AI Cockpit maintainers
title: "WI-578：WI-577 终态文档晋级"
description: "在不重写不可变记录的前提下晋级已关闭 WI-577 的文档投影。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-578-wi577-doc-promotion
lastVerifiedBy: WI-578-wi577-doc-promotion
terminalArchive: .ai/work-items/archive/WI-578-wi577-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-578-wi577-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-578-wi577-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-578-wi577-doc-promotion.close.json
---

[English](WI-578-wi577-doc-promotion.md) · [日本語](WI-578-wi577-doc-promotion.ja.md)

# WI-578：WI-577 终态文档晋级

## 目标

晋级已关闭 WI-577 的 Work Item 页面和 parity 登记，使文档投影真实，并通过仓库的关闭后
文档门禁。

## 边界

范围仅包括三个 WI-577 页面、三份 parity 行和本三语晋级记录。WI-577 archive/evidence/
decision bytes、Runtime 行为、对象工程、全局配置及历史文字保持不可变或在范围外。

## 验收

- 三个 WI-577 页面标为 `implemented`，并链接终态 archive、verification、finalization、close 证据。
- 每个 parity 页面将 WI-577 记录为已实现，并说明有界元数据检查；不增加语义比较声明。
- 三语文档验收、状态一致性和晋级 `--check-all` 通过。
- 不重写任何不可变治理记录。

## 验证

详见 active Contract 与 `tests/docs/promote_closed_work_item.py`。
