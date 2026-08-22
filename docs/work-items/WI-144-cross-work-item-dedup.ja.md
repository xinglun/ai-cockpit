---
author: AI Cockpit maintainers
workItemId: WI-144-cross-work-item-dedup
title: Work Item 間の物理実行再利用
description: 共有物理実行と Work Item ごとの認可 Evidence を分離します。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-144-cross-work-item-dedup
---

# WI-144 — Work Item 間の物理実行再利用

この Work Item は `PhysicalExecution`、`ExecutionResult`、Work Item ごとの
`WorkItemEvidenceReceipt` 境界を追加します。repository、snapshot、command、
environment、Runtime、toolchain identity が一致する場合だけ物理実行を共有し、
認可 Evidence は常に Work Item ごとに分離します。

実装 Evidence: `.ai/evidence/WI-144-cross-work-item-dedup.verification.json`。
クローズ決定: `.ai/decisions/WI-144-cross-work-item-dedup.close.json`。
