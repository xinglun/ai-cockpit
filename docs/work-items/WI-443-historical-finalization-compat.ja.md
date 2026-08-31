---
title: "WI-443 — Historical finalization compatibility"
workItemId: WI-443-historical-finalization-compat
description: "旧 shared worktree と PR のない local merge の正直な recovery 経路。"
audience: [maintainer, adopter, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-443-historical-finalization-compat
---

# WI-443 — Historical finalization compatibility

旧 shared-primary worktree と PR のない local merge に明示的な
`historical_low` record を導入します。direct merge は実際の commit、parents、base、
repository identity、authority を束縛し、Runtime が Git と照合します。Readiness は
repository-wide historical debt と recovery action を示し、新しい entry gate は弱めません。

対象 repository は Runtime release の導入と再検証まで凍結します。

[English](WI-443-historical-finalization-compat.md) · [简体中文](WI-443-historical-finalization-compat.zh-CN.md)
