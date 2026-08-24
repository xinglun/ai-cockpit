---
author: AI Cockpit maintainers
title: "WI-203——v0.2.25 发布与 transition 兼容性"
description: "恢复 v0.2.24 发布失败，并建立新的不可变 v0.2.25 发布基线。"
audience:
  - maintainer
  - adopter
workItemId: WI-203-release-v0-2-25
status: recovered
authority: canonical
lastVerifiedBy: WI-203-release-v0-2-25
---

# WI-203——v0.2.25 发布与 transition 兼容性

本 Work Item 是 WI-202 的明确 successor。v0.2.24 tag 与发布前失败的
workflow 保留为不可变历史，不会复用；当前基线严格递增一个 patch 到
v0.2.25。

范围仅包括版本/分发文档、parity 与治理记录、公开 Release evidence、
下载后的 adopter 验收，以及安装版 Runtime finalization transition。
Runtime 源码和 CI workflow 实现不在范围内。

公开验收只使用不可变的 v0.2.25 Release 资产，必须记录 manifest、archive
和 binary digest、adopter 与 N-1 回执、隔离 root manifest、清理证明、
transition receipt 和终态 Human Decision。源码检出或 workspace binary
不得作为 fallback。

文档入口：[English](WI-203-release-v0-2-25.md) · [日本語](WI-203-release-v0-2-25.ja.md)

## 验收边界

1. 版本、当前分发文档和三语 parity 在 verification 前一致指向 v0.2.25。
2. 公开 Release 稳定且不可变，并具有完整 manifest、checksum、archive、
   SBOM、Formula 和 provenance evidence。
3. 下载的 v0.2.25 在无源码 fallback 的隔离环境中通过 adopter 与
   v0.2.23→v0.2.25 N-1 验收。
4. 安装的 v0.2.25 能接受并记录 append-only finalization transition。
5. WI-202 recovery、Release identity、evidence reuse、isolation、cleanup
   和三语面向人的 Outcome 可审计。
