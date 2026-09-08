---
author: AI Cockpit maintainers
title: "WI-681——WI-674 终态文档晋级"
description: "将已关闭的 WI-674 Repository 拆分 Work Item 投影到受治理的中、日、英文档。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-681-wi674-doc-promotion
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-681-wi674-doc-promotion
---

[English](WI-681-wi674-doc-promotion.md) · [日本語](WI-681-wi674-doc-promotion.ja.md)

# WI-681——WI-674 终态文档晋级

## 意图

使 WI-674 的中、日、英 Work Item 页面与 reference-parity 行和不可变的
archive、verification、finalization、close 记录保持一致。

## 边界

这是仅限文档的投影。唯一预期变更是 6 个 WI-674 投影文件和 3 个 WI-681
self-registration 页面。不可变的 `.ai` 生命周期记录仅作为只读输入；不改变
Runtime 行为、治理规则、历史证据或其他 agent 的工作树。

## 验收

- WI-674 的三语言页面在验证关闭后显示终态 `Implemented` 与准确的终态证据路径。
- WI-674 的三语言 parity 行显示一致的终态及证据。
- WI-681 自身登记在三语言页面和 parity 行中，使 closed-work-item promotion
  检查保持有界且可重复。
- 在准确的 reviewed head 上通过文档、parity、状态一致性和 closed-work-item
  promotion 检查。
- 保留 `user_visible_benefit_not_declared`；本投影不声明用户可见、性能或认知收益。

WI-674 终态证据：

- archive：`.ai/work-items/archive/WI-674-p1-repository-split.contract.json`
- verification：`.ai/evidence/WI-674-p1-repository-split.verification.json`
- finalization：`.ai/decisions/WI-674-p1-repository-split.finalize.json`
- close：`.ai/decisions/WI-674-p1-repository-split.close.json`
