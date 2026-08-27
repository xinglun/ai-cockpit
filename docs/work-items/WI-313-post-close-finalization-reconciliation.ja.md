---
author: AI Cockpit maintainers
title: "WI-313 — post-close finalization reconciliation"
workItemId: WI-313-post-close-finalization-reconciliation
description: "cleanup-before-close を強制し、immutable な legacy close record のための限定的な recovery path を追加する。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-313-post-close-finalization-reconciliation
---

# WI-313 — post-close finalization reconciliation

## Intent and boundary

W312 は、旧 Runtime が provider finalization が `retained` のまま Work Item を
close でき、closed-document promotion gate が cleanup 完了の主張を拒否する順序欠陥を
示しました。本 Work Item は Runtime の境界を修正し、履歴 bytes を保持します。新しい
Work Item は close 前に provider resource を cleanup し、immutable な legacy close だけが
その後に bound deleted transition を 1 件追加できます。

## Scope and acceptance

Rust protocol/repository lifecycle は close 時に retained、blocked、unknown の
finalization を拒否します。close 後の transition は closed root digest、Work Item/repository
identity、次の sequence、branch/worktree が削除済みである正確な state を束縛した場合だけ
受理されます。close と元の finalization bytes は変更しません。documentation promotion
gate と三言語 workflow は通常 path と legacy path を説明し、未束縛または不完全な例外を
拒否します。

## Verification

Rust finalization targeted tests、closed-document promotion fixture、format/lint、workspace
tests、repository documentation gates を必須とします。最終 evidence には hosted CI と
installed Runtime identity を記録し、release acceptance の source-build fallback は認めません。
