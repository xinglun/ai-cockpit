---
author: AI Cockpit maintainers
title: "Human Benefit 报告"
description: "一个 Task Outcome 的简洁、由 evidence 推导的面向人的投影。"
audience:
  - adopter
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-136
capabilityClaims:
  - human_benefit_report
---

# Human Benefit 报告

Human Benefit 报告是已校验 Task Outcome 的面向人投影。它回答完成了什么、发现了
什么、何时停止、解决了什么、剩余哪些风险、哪些未知，以及下一步最安全的行动。

它不是第二个授权来源。每个 claim 都绑定 Task Outcome 的 evidence 引用；未声明的
用户收益必须保持为显式 unknown。Runtime 只本地化生成的标签，Contract 验收原文不变。

使用 `work-item outcome` 或 MCP `work_item_outcome` 获取交接；机器需要稳定的
OutcomeV2 和可选 `taskOutcomeReport` 时使用 `--json`。

[Task Outcome 报告](task-outcome-report.zh-CN.md) | [Outcome 参考](../reference/outcome-report.zh-CN.md) |
[English](human-benefit-report.md) | [日本語](human-benefit-report.ja.md)
