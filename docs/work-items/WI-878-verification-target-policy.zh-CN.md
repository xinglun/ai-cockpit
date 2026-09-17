---
author: AI Cockpit maintainers
workItemId: WI-878-verification-target-policy
title: 验证 target 缓存策略
description: 在保留依赖复用的同时控制 Cargo 验证缓存增长。
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-878-verification-target-policy
terminalArchive: .ai/work-items/archive/WI-878-verification-target-policy.contract.json
terminalVerification: .ai/evidence/WI-878-verification-target-policy.verification.json
terminalDecision: .ai/decisions/WI-878-verification-target-policy.close.json
---

[English](WI-878-verification-target-policy.md) · [日本語](WI-878-verification-target-policy.ja.md)

# WI-878——验证 target 缓存策略

本 Work Item 让 Runtime 以 `CARGO_INCREMENTAL=0` 和一个稳定的用户缓存 target
目录启动 Cargo 验证。它保留依赖复用，保持非 Cargo 命令不变，并记录安全删除仓库本地
增量目录的边界。不修改对象仓库、发布产物或验证授权语义。
