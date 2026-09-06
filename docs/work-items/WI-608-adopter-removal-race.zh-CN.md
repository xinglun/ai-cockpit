---
title: "WI-608 —— 抗竞态的 adopter 工作树清理"
description: "为发布验收提供有界重试且 fail-closed 的临时检出清理。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
lastVerifiedBy: WI-608-adopter-removal-race
terminalArchive: .ai/work-items/archive/WI-608-adopter-removal-race.contract.json
terminalVerification: .ai/evidence/WI-608-adopter-removal-race.verification.json
terminalFinalization: .ai/decisions/WI-608-adopter-removal-race.finalize.json
terminalDecision: .ai/decisions/WI-608-adopter-removal-race.close.json
workItemId: WI-608-adopter-removal-race
---

[English](WI-608-adopter-removal-race.md) · [日本語](WI-608-adopter-removal-race.ja.md)

# WI-608 —— 抗竞态的 adopter 工作树清理

## 目的

发布 adopter 脚本只删除精确的临时检出目录；有界重试容忍短暂的 Git
维护竞态，而清理失败仍会明确报告并保留验收凭证。

## 边界

本变更覆盖 staged/public adopter 与 N-1 验收脚本、回归包装脚本以及发布
工作流的认证传递。不改变 Runtime 治理语义、对象工程或全局 Agent/MCP 配置。

## 证据

- Archive：`.ai/work-items/archive/WI-608-adopter-removal-race.archive.json`
- Verification：`.ai/evidence/WI-608-adopter-removal-race.verification.json`
- Finalization：`.ai/decisions/WI-608-adopter-removal-race.finalize.json`
- Close：`.ai/decisions/WI-608-adopter-removal-race.close.json`
