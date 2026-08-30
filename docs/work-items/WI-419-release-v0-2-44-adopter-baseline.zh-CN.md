---
author: AI Cockpit maintainers
title: WI-419 — v0.2.44 发布后 adopter 基线
description: 固化公开 v0.2.44 adopter 验收 receipt 与 Runtime 身份。
workItemId: WI-419-release-v0-2-44-adopter-baseline
audience: [adopter, contributor, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-419-release-v0-2-44-adopter-baseline
terminalArchive: .ai/work-items/archive/WI-419-release-v0-2-44-adopter-baseline.contract.json
terminalVerification: .ai/evidence/WI-419-release-v0-2-44-adopter-baseline.verification.json
terminalFinalization: .ai/decisions/WI-419-release-v0-2-44-adopter-baseline.finalize.5e69364aa22b2a2fa6dafd2af75cd5eef1cc6b31b01bd41c09f4cdad956e9a08.json
terminalDecision: .ai/decisions/WI-419-release-v0-2-44-adopter-baseline.close.json
---

# WI-419 — v0.2.44 发布后 adopter 基线

[English](WI-419-release-v0-2-44-adopter-baseline.md) · [日本語](WI-419-release-v0-2-44-adopter-baseline.ja.md)

## 意图

为公开 v0.2.44 Release 持久化可重复的发布后 adopter 验收基线。receipt 将下载的
archive 和 binary 绑定到 Release 身份、adopter repository 身份、生命周期、隔离、证据复用和临时运行目录清理。

## 证据边界

完整的公开 binary harness 输出保存在
`.ai/evidence/WI-419-release-v0-2-44-adopter-acceptance/`，包括 `runtime.json`（archive
和 binary SHA-256）、`repository.json`、attach、profile、Agent doctor、保持
`state: not_ready` 的 `first-adopter-smoke`、证据复用、完整 Work Item 生命周期记录、隔离 manifest、
`cleanup.json`、`acceptance.json` 和 `SHA256SUMS`。本次只下载不可变的公开 v0.2.44
`aarch64-apple-darwin` archive，并验证临时运行根目录已删除。

## 验收

- `acceptance.json` 报告 `releasePublished: true` 和 `adopterAcceptance: passed`，所有步骤及 cleanup 验证通过。
- Runtime 版本为 `0.2.44`，binary digest 为
  `sha256:69d28c970c2b89534e63cb685c6cc02a2f135d3067b6a84feaabce2adce1d5e5`；adopter
  repository identity 为 `sha256:26301b33fabbb72aaacb48c8f9ccac335be8ca5aa42b9e98941324d2108a8df1`。
- 生命周期 verification evidence 为 schema 2；复用证据时没有新进程 spawn，并记录了结构化 close decision。
  持久化 checksum 文件验证所有保留 artifact。

## 非声明

这是公开 artifact 的验收记录，不是新的 Runtime 治理权威、源码构建、V1 fixture 或第二技术栈验收。
历史 receipt 保持不可变。
