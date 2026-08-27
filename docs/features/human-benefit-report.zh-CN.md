---
author: AI Cockpit maintainers
title: "Human Benefit 报告"
description: "一个 Task Outcome 的简洁、由 evidence 推导的面向人的投影。"
audience:
  - adopter
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-323
capabilityClaims:
  - human_benefit_report
---

# Human Benefit 报告

## 这项能力帮助你做什么

当你需要给人一个简短但有证据支持的答案时使用它：完成了什么、发现了什么问题、
触发了什么停止、解决了什么、还剩哪些风险、哪些内容未知，以及下一步最安全的
行动是什么。它是经过验证的 `OutcomeV2` 的面向人投影，不是第二个授权来源。

## 请求 Agent 提供交接结果

repository-bound CLI 路由是：

```sh
ai-cockpit work-item outcome --repo <repository> --id <work-item-id>
```

使用 MCP 的 Agent 应以 repository context 调用 `work_item_outcome`，并把返回的
`humanHandoff` 内容展示给人。`finish`、`archive` 和 `close` 同样在 stdout 保留
生命周期 JSON，并默认在 stderr 输出同一份 handoff。CLI 无法强制 Cursor 展开聊天面板；
Agent 或 provider adapter 必须展示 handoff，或重放 `work-item outcome`。

Runtime 会本地化生成的标签、状态、unknown、决定和下一步。Contract intent 与
acceptance criteria 保持其 authored language，不会被机器翻译成治理事实。未声明的
用户收益仍是显式 unknown；本页面不宣称参考源专有的
`implementation_approach_report` 能力。

自动化读取稳定机器对象时加上 `--json`，获得机器可读的 `OutcomeV2` 与可选的
`taskOutcomeReport`。该模式只抑制人类 handoff，不改变事实，也不授权生命周期转换。

## 结果包含什么

面向人的 handoff 按决策需要保持以下顺序：

```text
Task Result
Status: Success / Partial / Blocked / Failed

What was completed
Problems found
Stops triggered
Problems resolved
Risks avoided
Remaining risks
Unknowns
Human decisions
Verification
Impact
Next action
```

问题、警告、风险和强制停止的数量是 evidence record 数量，不是生产力、时间、金额、
安全或信任分数。绿色要求当前 Contract/Summary/evidence 绑定和已验证 Runtime 事实；
黄色表示需要调查或确认；红色表示必需控制失败并停止。摘要简短不等于可以跳过评审。

## 使用事例

如果补上了缺失的能力一览链接，合规 handoff 可以写成：

```text
完成：补上了缺失的能力一览链接。
已解决问题：文档入口现在能够到达能力一览。
证据：Contract、变更文件和通过的文档链接检查。
剩余风险：Hosted provider 评审尚未确认。
下一步：先评审 PR，等待 provider 结果后再合并。
```

Runtime 必须把没有证据的收益标成 `unknown` 或 `inference`；Agent 不得把 prose
改写成已完成事实。Contract、evidence 和 decision bytes 由 Runtime 生成，不要手动编辑。

## 报告缺失、过期或无效时

报告缺失、格式错误、过期、属于其他 Work Item、foreign repository、相互矛盾或与归档
Outcome 不一致时，先停止并验证源 Outcome。通过 Runtime 修复 Contract/evidence 后重新
生成投影，不要编辑报告来制造完成感。历史 evidence 仍是历史事实，不能显示成当前验证
通过或失败。

## 生命周期和责任边界

报告来自 repository-local 的 `.ai/` 记录，并遵循显式生命周期：

```text
start → preflight → checkpoint → verify → finish → archive → close
```

`work-item outcome` 是 Agent 或人的官方重放入口。`work_item_get` 是面向机器的
记录查询，不是人类 handoff 的替代。Runtime 负责事实、校验和本地化；provider/Agent 负责
对话展示。PR 创建、Hosted CI、合并、分支清理、平台隔离、企业合规和生产安全，除非有
单独绑定的外部 evidence，都不能由本报告证明。

[Task Outcome 报告](task-outcome-report.zh-CN.md) | [Outcome 参考](../reference/outcome-report.zh-CN.md) |
[English](human-benefit-report.md) | [日本語](human-benefit-report.ja.md)
