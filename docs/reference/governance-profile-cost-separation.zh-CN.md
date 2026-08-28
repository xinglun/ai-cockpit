---
author: AI Cockpit maintainers
title: 治理 profile 与成本分离
description: 保持 profile 强度、verification strength、assurance 与操作级升级彼此独立。
audience:
  - contributor
  - maintainer
  - adopter
status: implemented
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
---

# 治理 profile 与成本分离

Rust Runtime 有 `light`、`standard` 和 `strict` 质量路线。路线描述本次变更需要的 verification 强度，不是成本目标，也不代表组织的 assurance 等级。`release` 是操作类别，不是第四种 profile。

必须保持以下维度分离：

- `VerificationTier`（`T0`–`T3`）描述 verification strength；
- `EvidenceAssurance`（`SelfDeclared`、`RepositoryVerified`、`ProviderVerified`、`EnterpriseVerified`）描述 evidence provenance；
- cost observation 只描述已测量的工作量，仅供 advisory 使用。

有效路线由 stage、risk、声明的 operation、protected gates 和 repository policy 共同决定。Planner 可以提出 tier 或 escalation，但要求必须可追溯到 policy 或 protected gate。请求的 profile 可以提高路线，不能降低有效下限。

如果 operation 与 release 有关，路线可能要求 release preflight 和 distribution evidence。非 release 的 strict Work Item 不会仅因 strict 而继承 release graph。如果 policy 要求 `T3` 或 `ProviderVerified`，local-only 执行不能宣称完成，必须提供对应 provider 或 external evidence。

adopter repository 通过共享 Runtime 使用相同的 profile/cost 边界。不会创建进程级 current project 或隐藏的 planner policy；耗时和缓存也不能授权更弱的路线。

