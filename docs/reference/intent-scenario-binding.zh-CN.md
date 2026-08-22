---
author: AI Cockpit maintainers
title: Intent、Scenario 与 Stage 绑定
description: 说明如何把 Contract 人类事实绑定到 Policy 驱动的 Verification routing。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-143-intent-scenario-binding
---

# Intent、Scenario 与 Stage 绑定

Contract 的 intent 与 scenario coverage 是人类定义的事实。执行前，route
validator 要求 intent 非空、每个必需 scenario 存在，以及 operation 与 stage
匹配，然后把这些事实绑定到已经由 Policy 生成的 `VerificationRequirement`。

Validator 不读取实现描述来推断 authority、risk、assurance 或 T3 requirement。
因此高风险 route 仍必须由 Planner 提供显式 Policy rule 及 stage/gate reference。
缺少事实或 route 不匹配时，在 Verification 开始前 fail-closed。

`FinalDimensionsReceipt` 仍是精确的治理维度集合。`fourPillarProjection` 只用于
展示，不能授权或削弱 route。
