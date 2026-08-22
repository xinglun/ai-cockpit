---
author: AI Cockpit maintainers
title: Verification 成本观测
description: 说明如何在不削弱治理的前提下记录 Verification 成本估计与执行事实。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-146-verification-cost-observation
---

# Verification 成本观测

AI Cockpit 将 Verification 成本作为可审计、仅供参考的投影。它记录计划
节点、实际执行节点、可复用节点、资源单元、耗时、进程数和观测到的并行度。
成本投影不会改变 `VerificationTier`、`EvidenceAssurance`、Policy 要求、受保护
门禁或最终治理结果。

## 两个正交维度

Verification 强度与证据保证等级保持分离：

- `VerificationTier`：`T0`、`T1`、`T2`、`T3`。
- `EvidenceAssurance`：`SelfDeclared`、`RepositoryVerified`、
  `ProviderVerified`、`EnterpriseVerified`。

执行更快不代表 Verification 更强，较高 Tier 也不代表已经取得 Provider 或
Enterprise assurance。所需 Verification 仍由 Policy 与 protected gate 引用决定；
成本观测器只记录计划内容和实际执行内容。

## 估计与观测

`VerificationExecutionPlan::cost_estimate` 在执行前提供估计，
`VerificationReceipt::cost_observation` 在执行后投影事实。二者都包含 schema
版本、明确的 confidence 与 `advisoryOnly` 标记。当 worker/resource 预算、执行
状态或 repository/Runtime identity 不确定时，confidence 为 `partial` 或
`unknown`；未知测量不得变成绿色治理结果。

Reuse 与 affected verification 的减少量可以作为成本事实观察，但不能授权跳过
受保护节点或 Policy 要求的节点。物理执行复用也与每个 Work Item 的 Evidence
Receipt 分离；每个 Work Item 必须获得自己的 identity-bound receipt。

## 单个与并行 Work Item

观测同时支持单个 Work Item 和独立并行节点。`maxConcurrentProcesses` 表示实际
观测到的并发度，不是承诺，也不是性能目标。资源预算和依赖就绪条件仍约束执行；
即使成本估计不完整，受保护节点仍必须执行。

工程顺序是：

> **先统一 Verification Truth，再优化 Verification Cost。**

先保护 Policy、Tier、Assurance、范围、证据身份和受保护门禁，再用成本事实减少
不必要的执行工作。任何固定延迟或吞吐目标都不是 assurance 声明。
