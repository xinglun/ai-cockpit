---
author: AI Cockpit maintainers
workItemId: WI-154-policy-bound-runtime-route
title: Policy 绑定的 Runtime Verification route
description: 将 Policy requirement 与 stage/base facts 接入真实 Verification receipt，同时保持无 Policy 兼容性。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-154-policy-bound-runtime-route
---

# WI-154——Policy 绑定的 Runtime Verification route

Runtime 会在执行前解析已声明的 repository/Work Item verification
requirement。`VerificationTier` 与 `EvidenceAssurance` 保持正交：本地 route
即使命令成功，也不能因此满足 `T3` 或 `ProviderVerified`。`pr`、`merge`、
`release` stage 要求 Contract 有效的 `baseRevision`；`task` 不要求。

新的 Work Item receipt 绑定 repository/Work Item identity、snapshot digest、
base revision、Policy 引用、所需与实际 route 维度、受影响路径和 dependency
confidence。生命周期会重新校验这些绑定，因此 receipt 被篡改时不能成为
finish/archive 的事实。没有 typed verification requirement 的仓库继续保留
无 Policy/legacy route。

参见 [Verification route](../reference/verification-route.zh-CN.md)、
[English](WI-154-policy-bound-runtime-route.md) 和
[日本語](WI-154-policy-bound-runtime-route.ja.md)。
