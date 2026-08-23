---
author: AI Cockpit maintainers
title: "WI-195——治理完整性恢复质量门"
description: "让动态治理 inventory 识别 recovery，并加固公开 adopter 隔离回执。"
audience:
  - maintainer
  - reviewer
workItemId: WI-195-governance-recovery-gate
status: historical
authority: canonical
lastVerifiedBy: WI-196-governance-recovery-gate-retry
---

# WI-195——治理完整性恢复质量门

这是当前批次曾使用的 corrective Work Item：动态治理质量门会把有效的 superseded
predecessor 识别为 `recovered` 历史，同时对 malformed、foreign 或缺失的 recovery
保持 fail-closed。公开 adopter 与 N-1 验收 harness 也会绑定来源仓库 identity，检查每个
回执写入，并只在完成 identity-safe 校验后删除临时 run root。

Recovery 不是批准、验证或合并授权。blocked predecessor bytes 保持不可变且为红色；
successor 必须独立完成 Contract、evidence、托管 PR 与 closure lifecycle。

finish evidence 写入后发现了同范围 parity 修正。WI-195 保持为不可变的 recovered 历史，
全新交付由 WI-196 继续。修正后的 Release 和不可变 public-artifact acceptance 完成后，
下一批才开始参考源工程的逐文件对比。

[English](WI-195-governance-recovery-gate.md) ·
[日本語](WI-195-governance-recovery-gate.ja.md)
