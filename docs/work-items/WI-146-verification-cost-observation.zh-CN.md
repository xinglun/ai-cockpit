---
author: AI Cockpit maintainers
workItemId: WI-146-verification-cost-observation
title: Verification 成本观测
description: 增加带 identity 的成本估计与执行观测，但不改变治理语义。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-146-verification-cost-observation
---

# WI-146——Verification 成本观测

本 Work Item 为单个和并行 Verification 增加成本估计与执行观测。该投影仅供参考，
不能改变由 Policy 决定的 `VerificationTier`、`EvidenceAssurance`、受保护门禁或治理
结果。confidence 明确区分 `complete`、`partial` 和 `unknown`，未知事实保持可见。

实现证据：`.ai/evidence/WI-146-verification-cost-observation.verification.json`。
关闭决定：`.ai/decisions/WI-146-verification-cost-observation.close.json`。
