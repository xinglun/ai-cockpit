---
workItemId: WI-138-post-release-adoption-recovery
status: complete
lastVerifiedBy: WI-138-close
author: AI Cockpit maintainers
title: "发布后 adopter 验收与 stale 状态恢复"
description: "公开 v0.2.11 adopter 证据与 fail-closed stale 状态恢复边界。"
audience:
  - maintainer
  - adopter
authority: canonical
---

# WI-138——发布后 adopter 验收与 stale 状态恢复

## 目的

本 WI 记录使用公开 `v0.2.11` Runtime 完成的第一 adopter 验收，并记录发布准备阶段发现的安全恢复边界。

WI-137 在 `v0.2.11` 发布 commit 合并前完成 verify，因此它的 verification receipt 绑定的是更早的 repository snapshot。合并后 Runtime 正确把该 receipt 判定为 stale/foreign；这不允许手工修改 receipt 或降低校验级别。

## 恢复规则

如果 Work Item 已经处于 `finish_ready`，但 archive 前 repository 发生了变化，不得编辑 `.ai/work-items/**`、替换 `repositorySnapshotDigest` 或复用旧 verification receipt。必须保留历史 bytes，并基于当前 repository snapshot 创建一个新的、明确授权的 Work Item；新 WI 使用当前安装 Runtime 完整执行标准生命周期。这样同时保留失败的恢复边界与后续有效证据。

## 公开验收证据

- Release：[v0.2.11](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.11)
- Release workflow：[run 32578324451](https://github.com/xinglun/ai-cockpit/actions/runs/32578324451)
- Fresh adopter receipt：[artifact 9477249990](https://github.com/xinglun/ai-cockpit/actions/runs/32578324451/artifacts/9477249990)
- N-1 upgrade receipt：[artifact 9477256331](https://github.com/xinglun/ai-cockpit/actions/runs/32578324451/artifacts/9477256331)
- 仓库内验收摘要：`.ai/evidence/WI-138-release-adopter-acceptance.json`

公开 receipt 固定记录 release identity、repository ID、Runtime digest、`first-adopter-smoke = not_ready`、evidence reuse、完整 Work Item lifecycle、隔离 manifest 与 cleanup 状态。

## 验收边界

公开 fresh-adopter 与 N-1 job 运行在 Linux release target。当前 macOS ARM 安装则独立从公开 Release 下载，并依据 `release-manifest.json` 完成 checksum 校验，再使用显式 `--repo` 执行 `inspect`、`status`、`doctor` 与 `agent doctor`。

本验收不包含源码构建、本地 workspace binary、历史 evidence 重写或全局 Agent/MCP 配置改动。
