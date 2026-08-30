---
author: AI Cockpit maintainers
title: "WI-426 — Recovery successor binding compatibility"
description: 厳格な successor lineage を保ち、terminal な legacy successor だけを安全に認識します。
workItemId: WI-426-recovery-successor-binding
audience: [contributor, maintainer, reviewer]
status: recovered
authority: human-authorized
lastVerifiedBy: WI-426-recovery-successor-binding
---

# WI-426 — Recovery successor binding compatibility

## Intent

immutable な archived predecessor に有効な successor recovery receipt がある一方、
旧 Runtime の successor Contract に新しい predecessor field がない lifecycle gap を
閉じます。新しい Runtime が作成する successor は引き続き strict に bind されます。

## Scope

- 新しい successor Contract は predecessor Work Item、Contract digest、recovery path、
  repository identity を bind します。
- terminal evidence が揃う historical successor だけに限定した互換経路を許可し、
  新しい append-only recovery receipt に marker を記録します。
- foreign、stale、malformed、symlink、不完全、改ざんされた record は拒否します。
- predecessor bytes を保持し、三言語で境界を説明します。

## Evidence boundary

Historical compatibility は新しい green result ではありません。有効な archive manifest、
strict verification evidence、confirmed structured close が必要です。marker は
`successorBindingMode: legacy_terminal_evidence` です。

[English](WI-426-recovery-successor-binding.md) · [中文](WI-426-recovery-successor-binding.zh-CN.md)
