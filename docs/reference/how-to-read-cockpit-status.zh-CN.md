---
title: 如何阅读 Cockpit 状态
description: 把生成的 status 和 Outcome 转换为有边界的人工决定。
author: AI Cockpit 维护者
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: translation
canonical: docs/reference/how-to-read-cockpit-status.md
lastVerifiedBy: WI-346-reference-governance-profiles-status
capabilityClaims:
  - human_outcome_handoff
---

# 如何阅读 Cockpit 状态

[English](how-to-read-cockpit-status.md) · [简体中文](how-to-read-cockpit-status.zh-CN.md) · [日本語](how-to-read-cockpit-status.ja.md)

本页面向所有 Work Item 评审者，包括非技术批准人。它说明如何把生成的事实转成有边界的决定，
不会代替人做决定。

## 先看面向人的交接结果

先读取仓库绑定的 status，再重放面向人的交接：

```sh
ai-cockpit status --repo <repository> --id <work-item>
ai-cockpit work-item outcome --repo <repository> --id <work-item>
```

第二条命令会以 `Outcome: 🔴`、`Outcome: 🟡` 或 `Outcome: 🟢` 开头。CLI 和仓库绑定的 MCP
`work_item_outcome` 提供同一份面向人的投影。宿主可以决定如何显示消息，但折叠的日志或原始
`work_item_get` 响应不能替代这份交接结果。只有机器需要稳定的 `OutcomeV2` 对象时才使用 `--json`。

## 建议阅读顺序

1. **任务结果和标记**——确认 Work Item 及其决定信号。
2. **已完成内容**——阅读 Runtime 摘要和已交付变更声明。
3. **发现的问题和触发的停止**——确认失败 Gate 或暂停原因。
4. **已解决的问题和风险**——区分已记录的解决方案、剩余风险和警告。
5. **未知项**——每个 unknown 都是尚未解决的问题，不是隐藏的通过。
6. **人工决定**——如果存在，确认 actor、授权来源、理由、证据/策略引用、时间和恢复条件。
7. **验证与证据**——在采取行动前检查列出的仓库绑定回执及其新鲜度。
8. **影响和下一步**——未声明的用户收益仍然是 unknown；下一步必须遵循恢复或评审条件。
9. **验收标准（Contract 原文）**——阅读负责人原始 Contract 文本；为保证可审计性，文本保持原样。

参考源把部分字段称为 `Key Conclusion`、`Recommendation`、`Decision Drivers`、`Evidence` 和
`Scenario Coverage`。Rust Runtime 保留相同的阅读目的，但使用类型化 Outcome 章节和独立的
status 投影；这是语义对齐，不是源 JSON wire contract。

## 颜色的含义

| 标记 | 含义 | 安全的下一步 |
| --- | --- | --- |
| 🟢 Green | 当前的、绑定身份的证据足以进行评审。 | 阅读列出的证据并按组织流程取得所需决定；这不是 merge 或 release 授权。 |
| 🟡 Yellow | 证据不完整、部分有效、属于历史，或仍需人工决定。 | 调查、补齐证据或记录明确决定；保持 Work Item 的安全状态。 |
| 🔴 Red | 必需控制失败、权限/范围无效，或证据矛盾/损坏。 | 停止并遵循恢复条件；不要猜测或手工编辑生成记录。 |
| `unknown` 字段 | 事实或投影无法信任，或尚未声明。 | 请求澄清或新的绑定回执；不会静默变成 green。 |

颜色是语义信号，不是分数。绿色 Outcome 只表示可以审阅当前证据，不代表授权 merge、release、
发布、安全声明或企业 assurance。黄色或红色也不能通过重跑无关命令修复。

## 停止条件与证据边界

过期、损坏、符号链接、跨 Work Item、跨仓库或 snapshot 不匹配的 status/证据，必须通过 Runtime
重新生成。不要编辑生成的 Contract、Summary、Outcome、receipt、archive 或 decision 来改变颜色。
历史证据保持不可变；需要当前结果时，必须使用当前 Runtime 重新验证。

本地验证、Hosted CI、provider attestations、SBOM/provenance 和企业批准属于不同证据边界。报告必须
显示每个回执由哪个边界产生；本地 green 检查不能改称 provider 或 enterprise assurance。

## 语言与 adopter 继承

Runtime 生成的标题、标记、状态、摘要、unknown code 和恢复提示遵循 `AI_COCKPIT_LANGUAGE`（或 adapter
选择的语言）。Contract 的 intent、scope 和 acceptance criteria 保持原语言，自动翻译不能改变治理事实。
Agent 对话应使用用户语言呈现交接，同时保留 Contract 原文。

每个 adopter 仓库都通过显式 `--repo` 使用相同路线。共享 Runtime 没有 current project 或全局 Work Item，
因此一个仓库的 status 不能授权或描述另一个仓库的工作。

## 下一步参考

- [治理配置级别](governance-profiles.zh-CN.md)说明与风险相称的质量路由。
- [面向人的 Outcome](outcome-report.zh-CN.md)定义完整交接和机器边界。
- [命令参考](commands.zh-CN.md)列出生命周期命令和显式绑定。
- [排查与恢复](troubleshooting.zh-CN.md)说明停止后的恢复。
