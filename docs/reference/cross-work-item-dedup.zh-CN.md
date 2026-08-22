---
author: AI Cockpit maintainers
title: 跨 Work Item 的物理执行复用
description: 分离共享执行成本与 Work Item 授权证据。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-144-cross-work-item-dedup
---

# 跨 Work Item 的物理执行复用

当以下物理身份全部一致时，AI Cockpit 可以复用一次 Verification 执行成本：

`repository + repository snapshot + command + environment + Runtime + toolchain`

结果由 `PhysicalExecution` 和 `ExecutionResult` 表示。这两个类型都不包含
Work Item 身份，也不授予授权。

每个 Work Item 都必须从该结果生成自己的 `WorkItemEvidenceReceipt`。Work Item
ID 会参与 receipt digest，因此即使 A、B 共享一次物理执行，二者仍然拥有不同
的 Evidence Receipt。

> 任何 Work Item 都不得把另一个 Work Item 的 Evidence Receipt 作为自己的授权证据。

物理复用只是成本优化，不能降低 Policy 要求的 VerificationTier、
EvidenceAssurance、protected gate、authority 或 freshness 要求。repository、
snapshot、Runtime、command 或 toolchain 不一致时必须分开执行；身份未知时必须
fail closed。

实现证据：`crates/cockpit-verification/src/lib.rs` 与
`crates/cockpit-verification/tests/physical_execution.rs`。
