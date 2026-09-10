---
author: AI Cockpit maintainers
title: "WI-774——WI-773 终态文档 promotion"
description: "将已关闭的 WI-773 实验纳入所需的三语文档与 parity 边界。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-774-wi773-doc-promotion
lastVerifiedBy: WI-774-wi773-doc-promotion
---

[English](WI-774-wi773-doc-promotion.md) · [日本語](WI-774-wi773-doc-promotion.ja.md)

# WI-774——WI-773 终态文档 promotion

## 意图与边界

本 Work Item 将已关闭的 WI-773 性能实验投影到终态三语 Work Item 和
reference-parity 记录。保留 WI-773 不可变的 Contract、benchmark evidence、Outcome、
finalization、close 和候选拒绝判断；不引入生产行为或性能收益声明。

## 范围

- 在 WI-773 已验证关闭后，promotion 其英文、简体中文、日文和 parity 六项投影。
- 登记本 documentation-promotion Work Item 自身的三语页面与 prearchive parity 行，
  保持投影有界且可审计。
- 不修改 Runtime 生成记录、历史证据、治理规则或其他 Work Item。

## 验收与验证边界

- `promote_closed_work_item.py --check` 报告 WI-773 为 current。
- 三语 parity、documentation acceptance 和 Work Item status consistency 检查通过。
- Runtime verification 在 finish、archive、review、merge、finalization 和 close 前绑定当前
  Contract 与声明的文档检查。
