---
author: AI Cockpit maintainers
workItemId: WI-879-verification-target-policy
title: 验证目标缓存策略替代修复
description: 将共享的非增量 Cargo 验证目标策略带入新的审查分支，并修复 WI-878 归档后发现的质量门禁问题。
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-879-verification-target-policy
terminalArchive: .ai/work-items/archive/WI-879-verification-target-policy.contract.json
terminalVerification: .ai/evidence/WI-879-verification-target-policy.verification.json
terminalDecision: .ai/decisions/WI-879-verification-target-policy.close.json
---

[English](WI-879-verification-target-policy.md) · [日本語](WI-879-verification-target-policy.ja.md)

# WI-879 — 验证目标缓存策略替代修复

本 successor 从最新远端 main 重新承接 WI-878 的实现，并修复 WI-878
归档后质量门禁发现的回归。Cargo 验证固定使用 `CARGO_INCREMENTAL=0`
和一个稳定的用户缓存目标目录，以复用依赖而不为每次验证产生新的增量树。
非 Cargo 命令、对象工程、Issue #851 恢复行为和发布均不在范围内。

WI-878 的归档证据保持不可变；本 Work Item 只记录替代实现、文档投影和
新的质量证据。
