---
author: AI Cockpit maintainers
title: WI-416 — v0.2.43 发布后 adopter 基线
description: 固化公开 v0.2.43 adopter 验收 receipt 与 Runtime 身份。
workItemId: WI-416-release-v0-2-43-adopter-baseline
audience: [adopter, contributor, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-416-release-v0-2-43-adopter-baseline
terminalArchive: .ai/work-items/archive/WI-416-release-v0-2-43-adopter-baseline.contract.json
terminalVerification: .ai/evidence/WI-416-release-v0-2-43-adopter-baseline.verification.json
terminalFinalization: .ai/decisions/WI-416-release-v0-2-43-adopter-baseline.finalize.json
terminalDecision: .ai/decisions/WI-416-release-v0-2-43-adopter-baseline.close.json
---

# WI-416 — v0.2.43 发布后 adopter 基线

[English](WI-416-release-v0-2-43-adopter-baseline.md) · [日本語](WI-416-release-v0-2-43-adopter-baseline.ja.md)

## 意图

为公开 v0.2.43 Release 持久化可重复的发布后 adopter 验收基线。证据将下载的
archive 和 binary 绑定到 Release 身份、adopter repository 身份、生命周期、隔离
以及临时运行目录清理证明。

## 证据边界

完整 harness 输出保存在
`.ai/evidence/WI-416-release-v0-2-43-adopter-acceptance/`，包括
`runtime.json`（archive digest 与 binary SHA-256）、`repository.json`、attach、
profile、Agent doctor、保持 `state: not_ready` 的 `first-adopter-smoke`、证据复用、
生命周期记录、隔离 manifest、`cleanup.json`、`acceptance.json` 和 `SHA256SUMS`。
harness 只使用公开且不可变的 v0.2.43 archive，并已删除临时运行根目录。

## 验收

- `acceptance.json` 报告 `releasePublished: true`、`adopterAcceptance: passed`，
  全部步骤通过且 cleanup 已验证。
- 生命周期 verification evidence 为 schema 2，绑定 adopter `repositoryId`，并记录
  Runtime `0.2.43` 及 binary digest
  `sha256:d6334275904868d7e7e46a569e4198d75057d25f22997781df1a7097a3e70533`。
- 持久化 checksum 文件验证所有保留 artifact；历史 receipt 未被改写。

## 非声明

这是公开 artifact 的验收记录，不是新的 Runtime 治理权威、源码构建或第二技术栈验收。
