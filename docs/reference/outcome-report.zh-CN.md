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

绿色只表示 Runtime 已验证一份完整、未过期且绑定当前 Work Item 与 repository 的
`evidenceSchemaVersion=2` 验证证据。证据缺失或快照过期显示为黄色；证据被篡改、
格式错误、身份不匹配或摘要不一致显示为红色。`finish`、`archive`、`close` 在
相同校验失败时会 fail closed，不会因为证据文件存在就宣称成功。旧版证据不会被
自动改写为绿色，必须重新验证生成新版证据。

验收标准、intent、scope 等字段是 Work Item owner 写入的治理原文，报告保留原文并
标注“验收标准（Contract 原文）”，不会擅自翻译或改变 Contract bytes。只有 Runtime
生成的固定标题、摘要、状态、未知项和恢复提示会按对话语言显示。

CLI 直接输出优先使用 `AI_COCKPIT_LANGUAGE`，其次使用进程 locale。Agent 对话应
使用用户当前语言。JSON 字段名和枚举值在不同语言之间保持稳定。
