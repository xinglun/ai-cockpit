---
author: AI Cockpit maintainers
workItemId: WI-144-cross-work-item-dedup
title: 跨 Work Item 的物理执行复用
description: 分离共享物理执行与每个 Work Item 的授权证据。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-144-cross-work-item-dedup
---

# WI-144——跨 Work Item 的物理执行复用

本 Work Item 增加显式的 `PhysicalExecution`、`ExecutionResult` 与每个 Work Item
独立的 `WorkItemEvidenceReceipt` 边界。只有 repository、snapshot、command、
environment、Runtime 和 toolchain 身份一致时才能共享物理执行；授权证据始终
保持 Work Item 隔离。

实现证据：`.ai/evidence/WI-144-cross-work-item-dedup.verification.json`。
关闭决定：`.ai/decisions/WI-144-cross-work-item-dedup.close.json`。
