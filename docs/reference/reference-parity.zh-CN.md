---
author: AI Cockpit maintainers
title: "参考源对齐"
description: "Rust runtime 与参考 AI Cockpit template 的有证据比较。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: wi-41-reference-parity
capabilityClaims:
  - reference_parity
---

# 参考源对齐

本页记录 `xinglun/ai-cockpit` 与参考源
`spirex-ds-dev/ai-cockpit-template` 的比较。本次参考快照为
`e5acb67`，Rust runtime 基线为 `031f67d`。

这是边界审计，不是复制参考实现。Rust 项目是独立的 V2 runtime，不安装 V1
Python 模块、Makefile helper 或 V1 repository state。

## 对齐矩阵

| 参考关注点 | Rust runtime 状态 | 证据与边界 |
| --- | --- | --- |
| 面向读者的入口和语言切换 | 部分完成，WI-41 修正 | 根 README 已互相链接；完整阅读路线和内部历史清理由 WI-42 完成。 |
| 目的、问题、架构和功能概览 | 已实现 | `docs/philosophy*`、`docs/architecture*`、`docs/capabilities*` 描述 Rust runtime 及外部责任。 |
| 共享 Runtime 与 request-scoped repository context | 已实现 | `docs/architecture/runtime-topology*`、所有 CLI 显式 `--repo`、repository isolation tests。 |
| Repository attach 和最小 scaffold | 已实现 | `attach`、`.ai/cockpit.toml`、`.ai/project.json`、`.ai/agent-interface.json` 及 attach tests。 |
| 显式 Agent Discovery / Adapter 层 | 已实现 | `agent list/install/doctor/repair/detach`、受 ownership 保护的 managed section 和 `.ai/adapters/<provider>.json`。`attach` 不修改 Agent 文件。 |
| Work Item 生命周期和治理决定 | 已实现 | Contract、preflight、verification evidence、archive、close 和 human decision records。 |
| 有界验证与 fail-closed evidence reuse | 已实现 | Runtime identity、snapshot/toolchain/environment binding、receipt store 和 workspace verification suite。 |
| MCP repository binding | 已实现 | repository-bound stdio MCP service 与 CLI/MCP parity tests。 |
| 公开 Release 与新 adopter 验收 | 已实现 | WI-40 harness、公开 `v0.1.1` 证据和发布后 CI job。 |
| Runtime-only upgrade 与 repository migration | 尚未实现 | 当前 versioning 文档已声明边界，但没有 `compatibility` 或 `migrate` command；由 WI-43 实现。 |
| N-1 旧 adopter 升级验收 | 尚未实现 | 已有新 adopter 验收；旧 adopter 升级矩阵由 WI-44 实现。 |
| 参考 installer、Makefile 和 V1 helper scripts | 有意不复制 | Rust 项目分发 Rust binary，并把安装/provider 配置与 repository state 分离。 |
| 参考源历史 Work Item 和内部进度计划 | 不是产品能力 | WI-42 会从读者入口移除内部历史；归档证据仍在 Git 中可审计。 |

## 已完成内容

Rust 实现已经覆盖参考产品的核心用户边界：一份 Runtime 可以治理多个相互
独立 attach 的 repository；repository state 隔离；Agent discovery 显式且有
ownership；决定绑定证据；公开 Release 验收可重复执行。

当前项目有意保留 `cockpit.toml` 为 TOML。参考 template 的 JSON
project/profile records 在适用位置由 Rust Protocol files 表达；不把
`cockpit.toml` 改成 JSON。

## 后续 Work Item

- WI-42：让文档入口面向读者，删除入口中的进度历史，并规范三种语言路线。
- WI-43：加入显式 Runtime compatibility gate 和版本化 repository migration
  protocol，且不重写历史 evidence。
- WI-44：把 N-1 旧 adopter 升级加入 Release evidence matrix。

这些工作分成独立 Work Item，因为它们分别改变文档导航、runtime/repository
行为和发布验收 authority surface。
