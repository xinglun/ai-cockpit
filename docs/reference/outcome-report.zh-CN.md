---
author: AI Cockpit maintainers
title: "面向人的 Outcome"
description: "Work Item Outcome 的面向人交接结果。"
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: outcome-dialog-acceptance
capabilityClaims:
  - human_outcome_handoff
---

# 面向人的 Outcome

`ai-cockpit work-item outcome --repo <repository> --id <work-item>` 默认输出
面向人的交接结果。机器需要稳定的 `OutcomeV2` 对象时使用 `--json`。

输出顺序为：结果和状态、已完成内容、发现的问题、触发的停止、已解决的问题、
避免的风险、剩余风险、未知项、人工决定、验证与证据、影响、下一步。

状态标记是决策信号，不是发布授权：

- `🟢` 已有验证证据；继续前先审阅证据。
- `🟡` 部分完成、未就绪或未知；需要修复或调查。
- `🔴` 必需控制失败，或权限/范围无效；必须停止并恢复。

空章节会明确显示为 `无`。报告不会通过推断补全治理决定；绿色结果也不授权
合并、发布、公开或安全性声明。

CLI 直接输出优先使用 `AI_COCKPIT_LANGUAGE`，其次使用进程 locale。Agent 对话应
使用用户当前语言。JSON 字段名和枚举值在不同语言之间保持稳定。
