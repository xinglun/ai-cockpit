---
author: AI Cockpit maintainers
title: "参考源对齐"
description: "Rust runtime 与参考 AI Cockpit template 的有证据比较。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - reference_parity
---

# 参考源对齐

本页记录 Rust runtime 与参考 AI Cockpit 产品的功能边界比较，供采用者和审查者
理解当前能力；实现历史不属于读者路线。

## 对齐矩阵

| 参考关注点 | Rust runtime 状态 | 证据与边界 |
| --- | --- | --- |
| 面向读者的入口和语言切换 | 已实现 | 根 README 互相链接，阅读路线区分 adopter 与 maintainer 内容。 |
| 目的、问题、架构和功能概览 | 已实现 | `docs/philosophy*`、`docs/architecture*`、`docs/capabilities*` 描述 Rust runtime 及外部责任。 |
| 共享 Runtime 与 request-scoped repository context | 已实现 | `docs/architecture/runtime-topology*`、所有 CLI 显式 `--repo`、repository isolation tests。 |
| Repository attach 和最小 scaffold | 已实现 | `attach`、`.ai/cockpit.toml`、`.ai/project.json`、`.ai/agent-interface.json` 及 attach tests。 |
| 显式 Agent Discovery / Adapter 层 | 已实现 | `agent list/install/doctor/repair/detach`、受 ownership 保护的 managed section 和 `.ai/adapters/<provider>.json`。`attach` 不修改 Agent 文件。 |
| Work Item 生命周期和治理决定 | 已实现 | Contract、preflight、verification evidence、archive、close 和 human decision records。 |
| 有界验证与 fail-closed evidence reuse | 已实现 | Runtime identity、snapshot/toolchain/environment binding、receipt store 和 workspace verification suite。 |
| MCP repository binding | 已实现 | repository-bound stdio MCP service 与 CLI/MCP parity tests。 |
| 公开 Release 与新 adopter 验收 | 已实现 | 已提供公开 binary harness、Release 证据和发布后 CI job。 |
| Runtime-only upgrade 与 repository migration | 已实现 | `compatibility`、`migrate plan` 和批准后的 `migrate apply` 保留历史 evidence 并绑定 Runtime identity。 |
| N-1 旧 adopter 升级验收 | 已提供公开 artifact harness | harness 覆盖旧 schema 检测、批准门控、历史保持与继续运行；每个 Release 是否自动启用该门禁必须由 workflow 明确声明。 |
| 安装和 provider 配置 | 外部边界 | Rust 项目分发一份共享 binary，并把安装/provider 配置与 repository state 分离。 |

## 已完成内容

Rust 实现覆盖当前用户边界：一份 Runtime 可以治理多个相互独立 attach 的
repository；repository state 隔离；Agent discovery 显式且有 ownership；决定绑定
证据；公开 Release 验收可重复执行。矩阵中标为部分实现或可用的能力仍是明确的
后续边界，不应被理解为隐藏的完整对齐声明。

当前项目有意保留 `cockpit.toml` 为 TOML。参考 template 的 JSON
project/profile records 在适用位置由 Rust Protocol files 表达；不把
`cockpit.toml` 改成 JSON。

## 当前边界

Reader 路线、Runtime migration 边界和公开 artifact 验收 harness 已经实现并写入文档。后续变化必须
保持共享 Runtime 升级、显式 repository migration 与 repository-local evidence 的分离。
