---
author: AI Cockpit maintainers
title: "AI Cockpit"
description: "面向 AI 辅助工程的、以证据为基础的 repository 治理。"
audience:
  - adopter
  - contributor
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - repository_governance_layer
---

# AI Cockpit

[English](README.md) | [日本語](README.ja.md)

AI Cockpit 是面向 AI 辅助工程的 repository 治理 runtime。它把 repository
事实、声明的范围、验证结果和人的选择转化为可复查的有界决定。

## 它解决什么问题

AI 辅助修改可能超出范围、削弱测试、跳过验证，或让审查者缺少证据。AI Cockpit
明确记录预期修改、实际 repository 状态、必需检查、未知项和人类决定。

## 它如何工作

人和工具通过 CLI 或本地 MCP adapter 使用它；repository 状态通过 Repository
Protocol v1 保存，Rust 治理核心与应用代码保持独立。典型流程是：

`inspect → attach → start → preflight → checkpoint → verify → finish → archive → close`

`start` 记录由人负责的 Contract，`preflight` 判断是否可以开始，`checkpoint` 是
实现继续之前的串行门。`verify` 记录新鲜 evidence；`finish` 绑定结果，`archive`
保存不可变的 Work Item bundle，`close` 记录明确的人类决定。

## 30 秒开始

Runtime 只安装一份，然后 attach 当前要治理的 repository：

```bash
ai-cockpit attach --repo /path/to/repository
ai-cockpit status --repo /path/to/repository
```

先读[功能与边界](docs/capabilities.zh-CN.md)了解第一个受治理 Work Item，
再读[发布与分发](docs/release/distribution.zh-CN.md)了解安装和验证。

## 一个经过验证的完整案例

真实的、范围明确的完整交接案例见[WI-663 Outcome](.ai/work-items/archive/WI-663-wi659-outcome-trust-replacement.outcome.json)。
它是本 repository 治理记录的证据，不是关于普遍安全性或产品性能的声明。

- **结果：** 归档记录报告 `state=finish_ready`、`decisionState=green` 和
  `verification.status=verified`。[关闭决定](.ai/decisions/WI-663-wi659-outcome-trust-replacement.close.json)
  单独记录了 repository owner 的批准；验证通过和获得批准不是同一事实。
- **关键变化：** 输入是有明确 base 和边界范围的 Outcome 展示层修复。记录的发现保留了
  验证、生命周期和人工决定的区别，也保留历史、过期、缺失和已替代证据的区别。
- **证据边界：** [验证证据](.ai/evidence/WI-663-wi659-outcome-trust-replacement.verification.json)
  支持已声明的检查以及 repository/Work Item 绑定；[收尾 receipt](.ai/decisions/WI-663-wi659-outcome-trust-replacement.finalize.json)
  支持记录的合并和 cleanup 事实。两者都不能证明已经发布、普遍安全或产生了用户可见收益。
- **剩余不确定性：** `user_visible_benefit_not_declared` 保持显式存在。通过当前 Runtime
  查看历史记录时，还可能显示历史证据尚未重新验证；这是新鲜度限制，不是当前测试失败。
- **人的下一步：** 归档验证为 green 不会产生新的授权。如果要依据这些证据作出当前决定，
  应在当前 Runtime 下重新验证，并由人明确作出决定。

从 checkout 重复执行只读交接查询时，将占位路径替换为实际 repository 路径：

```bash
repo=/path/to/ai-cockpit
ai-cockpit work-item outcome --repo "$repo" \
  --id WI-663-wi659-outcome-trust-replacement
```

[首个 Work Item 路线](docs/getting-started/first-work-item.zh-CN.md)进一步把同一案例从输入、
范围映射到证据、Outcome、人工决定和 cleanup。

## 共享 Runtime，隔离 repository

分别 attach 每个目标 repository：

```text
ai-cockpit attach --repo /project-a
ai-cockpit attach --repo /project-b
```

binary 可以共享，但每个 repository 都有自己的 `.ai/` Contract、Evidence 和
Knowledge。所有 repository-bound command 都必须带 `--repo`；Runtime 不保存全局
current repository 或 active Work Item。

`attach` 只创建最小 repository scaffold（`cockpit.toml`、`project.json`、
`agent-interface.json`、Work Item 目录、evidence、decisions 和 knowledge），不会安装
Agent provider instruction。需要治理骨架时显式运行：

```bash
ai-cockpit work-item new --repo /project-a \
  --id payment-refund-guard --mode code
```

命令会列出已确定推导的事实和仍需人类填写的 `intent`、`scope`、`acceptanceCriteria`、
`authority`。结果状态是 `not_ready`，脚手架不会声称 approved 或 verified。类似地，
`profile propose --repo /project-a` 只输出候选 amendment，不改变正式 profile。

如果要让选定的 Agent 宿主发现该 repository，请显式使用 repository-local adapter：

```bash
ai-cockpit agent list --repo /project-a
ai-cockpit agent install --repo /project-a --provider codex
ai-cockpit agent doctor --repo /project-a --json
```

这只会在选定的 repository surface 和 `.ai/adapters/` 写入受 ownership 保护的内容，
不会修改全局 Agent/MCP 设置。Discovery、adapter 安装、连接、验证和合规仍是不同状态。

## 三种决定状态

- `green`：已有证据支持当前有边界的下一步动作；
- `yellow`：证据缺失、过期、矛盾或需要人工确认；
- `red`：控制失败或权限缺失，操作必须停止。

## 从这里开始

- [文档导航](docs/README.zh-CN.md)——选择采用者、贡献者、审查者、MCP 或维护者路径。
- [功能与边界](docs/capabilities.zh-CN.md)——查看当前命令能力和外部责任。
- [发布与分发](docs/release/distribution.zh-CN.md)——安装、验证、回滚和 MCP 配置。

在源码检出中，贡献者可用 `cargo run -p cockpit-cli -- --help` 查看命令面。公开
Release 和 Homebrew 是否可用属于独立的发布证据，不能由当前源码检出推断。

## 仍由外部负责

外部 identity、branch protection、生产隔离、provider Release 和 provenance 仍属于
外部证据或采用者责任。AI Cockpit 提供有界的 repository 治理，不替代人工 review、
组织自身的安全系统或合规体系。
