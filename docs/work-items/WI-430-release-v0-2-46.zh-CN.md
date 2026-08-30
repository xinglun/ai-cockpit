---
author: AI Cockpit maintainers
title: WI-430——v0.2.46 发布
description: 将 WI-429 历史恢复修复作为不可变 Runtime Release 发布。
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
workItemId: WI-430-release-v0-2-46
---

# WI-430——v0.2.46 发布

[English](WI-430-release-v0-2-46.md) · [日本語](WI-430-release-v0-2-46.ja.md)

## 意图

将经过审查的 WI-429 recovery-history 修复发布为 v0.2.46，使采用者可以从不可变的公开制品安装已修正的 Runtime。

## 边界

本 Work Item 只推进一个 patch release 并同步发布文档。不改变治理语义，不复制 reference/V1 runtime，不修改全局 Agent 或 MCP 配置，也不修改 adopter 业务工程。

## 验收

- Cargo metadata 和 lockfile 从 v0.2.45 准确推进到 v0.2.46，不复用已有 tag 或 Release。
- 发布流程绑定准确的 reviewed commit、五个 target archive、SBOM、manifest、Formula、校验和、provenance 与 Release identity。
- 发布、安装、版本和 parity 文档在英语、简体中文、日语之间同步；v0.2.45 保留为历史记录。
- 发布后验收只能使用不可变的公开 v0.2.46 制品，并拒绝源码、workspace 或本地 binary fallback。
- merge、finalize、close、同步和精确清理完成后，`main` 为 `ready_on_base`。

## 验证边界

发布前使用 Contract 声明的 release gates。公开 adopter 验收是独立的发布后 Work Item，必须保留 Runtime identity、制品 digest、隔离 manifest 和清理证明。
