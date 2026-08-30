---
author: AI Cockpit maintainers
title: WI-422——v0.2.45 发布
description: 在 mixed-monorepo 参考批次之后发布已审查 Runtime。
workItemId: WI-422-release-v0-2-45
audience: [adopter, maintainer, reviewer]
status: release-preparation
authority: human-authorized
lastVerifiedBy: WI-422-release-v0-2-45
---

# WI-422——v0.2.45 发布

[English](WI-422-release-v0-2-45.md) · [日本語](WI-422-release-v0-2-45.ja.md)

## 意图

在 mixed-monorepo 参考逐文件比对批次完成后，将已审查的 `main` 发布为 v0.2.45，
并同步发布身份、安装指引和三语 parity 记录。

## 边界

本 Work Item 只推进一个补丁版本并验证现有严格发布路径。不改变 Runtime 治理语义，
不复制参考源或 V1 Runtime/安装器代码，不修改全局 Agent/MCP 配置，也不包含 adopter
应用源码。公开 artifact 的 adopter 验收由发布后的独立 Work Item 负责，并且只能使用
不可变的 v0.2.45 制品。

## 验收

- Cargo metadata 与 lockfile 从 v0.2.44 精确推进一个补丁版本至 v0.2.45；不复用已有 tag 或 Release。
- 已审查 workflow 绑定精确 reviewed commit、目标 archive、SBOM、manifest、Formula、checksum、
  provenance 以及不可变 tag/Release identity。
- 发布、安装、版本和 parity 指引在英文、简体中文和日文中同步；在新的公开 adopter 基线验收前，
  v0.2.44 保留为历史 evidence。
- 后续隔离 Work Item 只能使用不可变公开 v0.2.45 制品进行发布后验收；禁止源码、workspace 或本地 binary fallback。
- 审查合并、finalization、close、默认分支同步和精确分支/worktree 清理后，`main` 保持 `ready_on_base`。

## 验证边界

发布前使用 Contract 声明的严格源码与发布 gate。staged candidate 和源码构建不是公开 adopter evidence。
发布后验收 receipt 必须保留 Runtime identity、制品 digest、隔离 manifest 和 cleanup proof，且不能改写 Release truth。
