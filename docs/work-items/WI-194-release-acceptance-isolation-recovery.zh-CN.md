---
author: AI Cockpit maintainers
title: "WI-194——发布验收隔离恢复"
description: "保留不可变的 WI-194 恢复历史，并将有界的发布隔离交付交给 WI-195。"
audience:
  - maintainer
  - reviewer
workItemId: WI-194-release-acceptance-isolation-recovery
status: historical
authority: canonical
lastVerifiedBy: WI-195-governance-recovery-gate
---

# WI-194——发布验收隔离恢复

WI-194 保留了 WI-193 的发布验收隔离实现，但其 archive evidence 由 source-built
Runtime 生成，resource context 还引用了不存在的 provider PR。这些 bytes 保持不变，
属于历史 evidence，不是当前 Release 证明。

因此 WI-194 的状态是已恢复，而不是已完成。显式 recovery 回执绑定了 archive 的
Contract、Summary、Outcome、events、repository 与 Runtime identity，并将同一有界交付
交给 WI-195。不会改写历史 archive、evidence 或已发布 Release truth。

Evidence：`.ai/evidence/WI-194-release-acceptance-isolation-recovery.verification.json`；
recovery：`.ai/decisions/WI-194-release-acceptance-isolation-recovery.recovery.json`；
archive：`.ai/work-items/archive/WI-194-release-acceptance-isolation-recovery.archive.json`。

[English](WI-194-release-acceptance-isolation-recovery.md) ·
[日本語](WI-194-release-acceptance-isolation-recovery.ja.md)
