---
author: AI Cockpit maintainers
title: "WI-204——v0.2.25 发布与 transition 兼容性恢复"
description: "在 verification 与 archive 之前绑定资源上下文，重新完成 v0.2.25 发布边界。"
audience:
  - maintainer
  - adopter
workItemId: WI-204-release-v0-2-25
status: in_progress
authority: canonical
lastVerifiedBy: WI-204-release-v0-2-25
---

# WI-204——v0.2.25 发布与 transition 兼容性恢复

WI-204 是 WI-203 的明确 successor。WI-203 在 `finalize-plan` 绑定真实
branch、worktree 和 pull request 上下文之前已归档，因此保持为不可变历史。
本 Work Item 在 verification 与 archive 之前绑定该上下文，重新完成同一
v0.2.25 发布边界。

公开验收只使用不可变的 v0.2.25 Release 资产，记录 Release identity、下载的
adopter/N-1 回执、隔离与清理证据、append-only transition 和终态 human decision。
v0.2.24 tag 仍是发布前失败历史，不会复用。

文档入口：[English](WI-204-release-v0-2-25.md) · [日本語](WI-204-release-v0-2-25.ja.md)

## 验收边界

1. 版本、分发文档和所有 parity 行一致指向 v0.2.25。
2. 不可变公开 Release 提供完整 manifest、checksum、archive、SBOM、Formula
   和 provenance evidence。
3. 下载的 v0.2.25 在无源码 fallback 的隔离环境中通过 adopter 与
   v0.2.23→v0.2.25 N-1 验收。
4. 安装的 v0.2.25 接受并记录 append-only finalization transition。
5. WI-203 recovery、Runtime identity、evidence reuse、isolation、cleanup 和
   三语面向人的 Outcome 保持可审计。
