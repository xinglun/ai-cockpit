---
author: AI Cockpit maintainers
workItemId: WI-140-verification-semantics
title: Verification 语义与 Artifact 归档完整性
description: 定义正交的 Verification 事实维度，并在归档时保留全部 Work Item 生成 Artifact。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-140-verification-semantics
---

# WI-140——Verification 语义

在 Planner 和性能工作之前，定义相互独立的 `VerificationTier` 与
`EvidenceAssurance`。同时修复生成的 implementation approach 或并行
intelligence sidecar 在归档后残留于 `active` 目录的仓库 Artifact 问题；持有中的
并行 slot 会阻止归档，必须先显式释放。

证据：

- `.ai/evidence/WI-140-verification-semantics.verification.json`
- `.ai/work-items/archive/WI-140-verification-semantics.archive.json`
- `.ai/decisions/WI-140-verification-semantics.close.json`
