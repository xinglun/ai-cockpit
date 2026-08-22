---
author: AI Cockpit maintainers
title: "WI-156——物理执行与 Work Item 证据回执"
description: "将共享物理计算与 Work Item 授权分离，并拒绝伪造的成本遥测。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-156-physical-execution-receipt
workItemId: WI-156-physical-execution-receipt
---

# WI-156——物理执行与 Work Item 证据回执

WI-156 将物理执行结果与授权 Work Item 的治理回执分离。同一物理结果可以被多个
Work Item 观察，但每个 Work Item 必须绑定并校验自己的回执；任何 Work Item 都不能
把另一个 Work Item 的回执当作自己的授权或决定证据。

成本观测只是建议性遥测。持久化或缓存的观测只有在身份、计数器和规范化小写
SHA-256 digest 都与执行回执完全一致时才可接受。伪造缓存会被投影为
`unknown` 并标记 `cost_observation_invalid`，不能使治理结果变绿。

证据：`.ai/evidence/WI-156-physical-execution-receipt.verification.json`。
决定：`.ai/decisions/WI-156-physical-execution-receipt.close.json`。
