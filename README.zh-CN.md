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

`inspect → attach → preflight → verify → finish/archive/close`

## 30 秒开始

Runtime 只安装一份，然后 attach 当前要治理的 repository：

```bash
ai-cockpit attach --repo /path/to/repository
ai-cockpit status --repo /path/to/repository
```

先读[功能与边界](docs/capabilities.zh-CN.md)了解第一个受治理 Work Item，
再读[发布与分发](docs/release/distribution.zh-CN.md)了解安装和验证。

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
