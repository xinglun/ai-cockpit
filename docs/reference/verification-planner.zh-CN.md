---
author: AI Cockpit maintainers
title: Policy 驱动的 Verification Planner
description: 说明 Policy 与 Stage 如何生成可追溯的 Verification plan。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-141-policy-planner
---

# Policy 驱动的 Verification Planner

Planner 按 Organization、Project、Work Item 的顺序消费显式 Policy layer。
Policy rule 可以携带 `VerificationRequirement`，其中
`requiredTier` 与 `requiredAssurance` 相互独立。`T3` 只表示需要更强的
Verification，不表示 ProviderVerified 或 EnterpriseVerified evidence。

对于选定的 operation 与 stage，Planner 要求：

- 每个输入 Policy layer 都有匹配 rule；
- requirement 有效，并引用来源 policy id；
- stage reference 与请求 stage 匹配；
- 请求 protected gate 时，必须存在对应 gate reference。

缺少 rule 或 reference 时 fail-closed。下层 Policy 可以增加 evidence 或
提高 tier/assurance，但不能削弱上层要求。Planner 输出记录来源 policy id
与 escalation reason，因此 required tier 不会隐藏在 operation 名称规则中。

Planner 只定义 Verification requirement，不创建人类授权、provider
assurance、依赖完整性、执行复用或性能豁免。

WI-139C 与 WI-139F 的历史 approach artifact 保持原始 bytes，并绑定到 archive
manifest；active 中不再存在被误认为当前项目状态的孤儿 artifact。
