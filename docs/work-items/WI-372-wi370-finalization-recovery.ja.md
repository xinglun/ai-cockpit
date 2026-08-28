---
author: AI Cockpit maintainers
title: "WI-372 — WI-370 provider finalization 復旧"
description: "不変な predecessor bytes を書き換えず、review 済み PR identity を束ねて performance Work Item を close する。"
workItemId: WI-372-wi370-finalization-recovery
audience: [maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-372-wi370-finalization-recovery
capabilityClaims: [governance_integrity, resource_finalization]
---

# WI-372 — WI-370 provider finalization 復旧

[English](WI-372-wi370-finalization-recovery.md) · [简体中文](WI-372-wi370-finalization-recovery.zh-CN.md)

## Intent と boundary

WI-370 は review 済み PR identity が確定する前に archive され、不変な resource context に
placeholder の pull-request URL が残った。この有界 successor は fresh verification 前に実際の
PR #333 identity を記録し、正確な branch/worktree finalization を完了する。predecessor の
Contract、verification、archive、outcome bytes は不変のまま保持する。

## Acceptance

- fresh verification 前に review 済み PR #333 context を束ねる。
- predecessor の archive と evidence bytes を変更しない。
- close 前に正確な branch と worktree の削除を検証する。
- hosted review、finalization、可視の human Outcome を記録する。

これは governance recovery であり、Runtime の性能や release artifact は変更しない。
