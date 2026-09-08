---
author: AI Cockpit maintainers
title: “WI-702 — WI-701 parity order recovery”
description: “verification evidence より先に parity 登録を commit して WI-701 recovery を再配信します。”
audience: [contributor, maintainer, reviewer]
workItemId: WI-702-wi701-parity-order-recovery
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-702-wi701-parity-order-recovery
---

[English](WI-702-wi701-parity-order-recovery.md) · [简体中文](WI-702-wi701-parity-order-recovery.zh-CN.md)

# WI-702 — WI-701 parity order recovery

WI-702 は immutable failed delivery となった WI-701 の fresh successor です。
記録された `origin/main` base に bind し、fresh verification evidence の生成前に
parity projection を登録します。

## Boundary

WI-701、WI-700、WI-698 の archive と evidence は保持します。新しい code semantics、
governance rule、protocol format は導入せず、他 agent の Work Item も変更しません。

## Acceptance

- recovery decision と predecessor digest が replacement delivery を bind すること。
- 三言語 parity 登録が WI-702 verification evidence より前の commit history にあること。
- terminal 前に fresh verification、hosted review、provider finalization、archive、close、
  exact cleanup の evidence がそろうこと。
