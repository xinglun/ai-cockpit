---
author: AI Cockpit maintainers
title: "WI-356——发布 adopter 验收生命周期顺序"
workItemId: WI-356-release-adopter-script-order
description: "使发布物 adopter harness 与 Runtime 生命周期入口门保持一致。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-356-release-adopter-script-order
terminalArchive: .ai/work-items/archive/WI-356-release-adopter-script-order.contract.json
terminalVerification: .ai/evidence/WI-356-release-adopter-script-order.verification.json
---

# WI-356——发布 adopter 验收生命周期顺序

[English](WI-356-release-adopter-script-order.md) · [日本語](WI-356-release-adopter-script-order.ja.md)

## 意图与边界

发布 adopter harness 必须先 attach 新 repository，显式安装 Agent adapter，
提交治理状态，然后才能创建第一个 Work Item scaffold。这样遵守 Runtime
fail-closed 的干净入口规则，并保持验收可重复。

本次仅修改 staged adopter harness 及其静态回归。Runtime 行为、公开发布物、
全局 Agent/MCP 配置和 upgrade harness 不在本 Work Item 范围内。

## 验证与交付边界

脚本静态检查已通过，并覆盖成功与失败路径的清理断言。归档 Contract、
verification evidence 以及 provider finalization/close receipt 是权威生命周期
记录；pre-merge parity 行会在审阅 PR 合并并关闭后才提升为“已实现”。
