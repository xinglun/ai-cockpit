---
author: AI Cockpit maintainers
workItemId: WI-878-verification-target-policy
title: 検証 target cache policy
description: dependency の再利用を保ちながら Cargo 検証 cache の増加を制御する。
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-878-verification-target-policy
terminalArchive: .ai/work-items/archive/WI-878-verification-target-policy.contract.json
terminalVerification: .ai/evidence/WI-878-verification-target-policy.verification.json
terminalDecision: .ai/decisions/WI-878-verification-target-policy.close.json
---

[English](WI-878-verification-target-policy.md) · [简体中文](WI-878-verification-target-policy.zh-CN.md)

# WI-878 — 検証 target cache policy

この Work Item は Runtime が `CARGO_INCREMENTAL=0` と一つの安定した user-cache
target directory で Cargo verification を起動するようにします。dependency の再利用を
保ち、Cargo 以外の command を変更せず、repository-local incremental directory の安全な
削除範囲を文書化します。object repository、release artifact、verification authorization
semantics は変更しません。
