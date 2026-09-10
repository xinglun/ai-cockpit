---
author: AI Cockpit maintainers
title: "WI-774——WI-773 终态文档 promotion"
description: "将已关闭的 WI-773 实验纳入所需的三语文档与 parity 边界。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: authorized
workItemId: WI-774-wi773-doc-promotion
lastVerifiedBy: WI-774-wi773-doc-promotion
---

[English](WI-774-wi773-doc-promotion.md) · [日本語](WI-774-wi773-doc-promotion.ja.md)

# WI-774——WI-773 终态文档 promotion

## 意图与边界

WI-774 作为不可变的失败文档交付被保留。其 hosted PR #757 因 parity 注册与验证证据在同一
提交中产生而未通过 `docs_governance_integrity`。WI-775 作为 recovered predecessor 保留，
WI-776 是从最新默认分支完成 archive-evidence 重新交付的有界 successor。WI-773 的不可变
Contract、benchmark evidence、Outcome、finalization、close 和候选拒绝判断保持不变。

## 范围

- 保留 WI-774 的 archive、verification evidence、recovery decision 和 hosted failure 绑定，
  不重写历史字节。
- 由 WI-775 → WI-776 负责有序重新交付，并确保 parity 注册先于新鲜 verification evidence。
- 不修改 Runtime 生成记录、历史证据、治理规则或其他 Work Item。

## 不可变 recovery 绑定

- archive：`.ai/work-items/archive/WI-774-wi773-doc-promotion.archive.json`
- verification：`.ai/evidence/WI-774-wi773-doc-promotion.verification.json`
- recovery：`.ai/decisions/WI-774-wi773-doc-promotion.recovery.json`
- 失败交付：[PR #757](https://github.com/xinglun/ai-cockpit/pull/757)
- successor chain：WI-775 parity-order recovery → WI-776 archive-evidence recovery

## 验收与验证边界

- `promote_closed_work_item.py --check` 报告 WI-773 为 current。
- 三语 parity、documentation acceptance 和 Work Item status consistency 检查通过。
- Runtime verification 在 finish、archive、review、merge、finalization 和 close 前绑定当前
  Contract 与声明的文档检查。
