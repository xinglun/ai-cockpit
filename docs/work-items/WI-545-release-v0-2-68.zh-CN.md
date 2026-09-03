---
author: AI Cockpit maintainers
title: "WI-545——v0.2.68 发布与公开产物验收"
description: "发布下一版经过验证的 Runtime，并绑定公开安装证据。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: canonical
workItemId: WI-545-release-v0-2-68
lastVerifiedBy: WI-545-release-v0-2-68
terminalArchive: .ai/work-items/archive/WI-545-release-v0-2-68.contract.json
terminalVerification: .ai/evidence/WI-545-release-v0-2-68.verification.json
terminalFinalization: .ai/decisions/WI-545-release-v0-2-68.finalize.json
terminalDecision: .ai/decisions/WI-545-release-v0-2-68.close.json
---

[English](WI-545-release-v0-2-68.md) · [日本語](WI-545-release-v0-2-68.ja.md)

# WI-545——v0.2.68 发布与公开产物验收

## 意图与目标

从经过审查且已同步的默认分支发布 v0.2.68，并证明不可变公开产物可以在不回退到源码或工作区二进制的情况下安装和验收。

## 范围

- 更新 Cargo package identity 以及三语 release/versioning 页面。
- 在参考源 parity ledger 登记本次发布并保留 Work Item 证据路径。
- 执行发布前质量/策略检查；发布后执行公开产物、adopter、N-1、已安装 Runtime 和清理验收。

## 验收边界

公开 Release、archive/SBOM/provenance digest 和已安装 binary 必须与不可变 tag 一致。补充人类拥有的 Contract 字段前，`first-adopter-smoke` 保持 `not_ready`。安装不会 attach repository，本 Work Item 不修改对象工程或全局 Agent/MCP 配置。

## 验证

当前 Contract 对命令和 evidence 具有权威性。终态交付必须包含可见的人类 Outcome（状态、未知项、证据、人类决定、下一步），然后通过文档 promotion 和精确的分支/工作树清理。
