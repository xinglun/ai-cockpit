---
author: AI Cockpit maintainers
workItemId: WI-903-precheckpoint-contract-scope
title: checkpoint 前的 Contract scope 增补
description: 当遗漏必须的初始投影路径时，允许尚未 checkpoint 或验证的 active Work Item 通过 Runtime 追加 scope。
audience: [maintainer, reviewer, contributor]
status: implemented
authority: human:xinglun
lastVerifiedBy: WI-903-precheckpoint-contract-scope
terminalArchive: .ai/work-items/archive/WI-903-precheckpoint-contract-scope.contract.json
terminalVerification: .ai/evidence/WI-903-precheckpoint-contract-scope.verification.json
terminalDecision: .ai/decisions/WI-903-precheckpoint-contract-scope.close.json
---

# WI-903 — checkpoint 前的 Contract scope 增补

本 WI 修复启动 WI-902 时发现的生命周期死锁。Runtime 在 checkpoint 前要求
已声明文档投影，但此前又拒绝在不存在 checkpoint 时使用唯一受支持的 scope
增补入口。修复严格限定为首次 checkpoint 和 verification result 之前的附加
scope；不允许修改身份、授权、基线 revision、模式、既有验收条件或任何
checkpoint 后证据边界。

## 验收与证据

- fixture 可在 checkpoint 前追加遗漏路径，并通过正常 preflight/checkpoint。
- checkpoint 后 amendment 的 revalidation 规则保持不变。
- 修改不可变 Contract 字段的尝试仍被拒绝。
