---
author: AI Cockpit maintainers
title: WI-418——v0.2.44 发布
description: 发布包含 lockfile-aware Cargo 验证命令选择的已审查 Runtime。
workItemId: WI-418-release-v0-2-44
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-418-release-v0-2-44
---

# WI-418——v0.2.44 发布

[English](WI-418-release-v0-2-44.md) · [日本語](WI-418-release-v0-2-44.ja.md)

## 意图

在 lockfile-aware Cargo 验证命令选择修复之后，将已审查的 `main` 发布为 v0.2.44，
并保持发布身份与三语文档同步。

## 边界

本 Work Item 只推进补丁版本并验证严格发布源码路径，不改变治理语义、不复制参考源/V1
Runtime 或安装器、不修改全局 Agent/MCP 配置，也不修改 adopter 应用源码。公开 artifact
adopter 验收仍由发布后的独立 Work Item 负责。

## 验收

- Cargo metadata 与 lockfile 从 v0.2.43 精确推进一个补丁版本至 v0.2.44，不复用 tag 或 Release。
- 已审查 workflow 绑定精确 commit、目标 archive、SBOM、manifest、Formula、checksum、
  provenance 以及不可变 tag/Release 身份。
- 发布、安装、版本和 parity 指引在英文、简体中文和日文中保持同步。
- 后续隔离 Work Item 只能使用不可变公开 v0.2.44 artifact 做 adopter 验收。
- 审查合并、finalization、close、同步和精确清理后，`main` 保持 `ready_on_base`。

## 验证边界

发布前使用声明的严格源码与发布 gate。staged candidate 或源码构建不会被当作公开 adopter
证据。
