---
author: AI Cockpit maintainers
title: WI-405 — Active artifact recovery
description: 不変な履歴を隠したり書き換えたりせず、失敗した Work Item の成果物を復旧します。
workItemId: WI-405-active-artifact-recovery
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-405-active-artifact-recovery
---

# WI-405 — Active artifact recovery

## Intent

失敗または中断した Work Item の成果物を監査可能な状態に保ち、`active/`
の残留ファイルが有効なアクティブ状態と誤認されないようにします。

## 範囲

- 既知の失敗試行 outcome/event 変種を検出して復旧する。
- archive manifest に bytes と digest を保持する。
- 孤立した active 成果物を有効な active Contract と分離して報告する。
- repository と Runtime evidence の分離を維持する。

## Evidence

- Archive Contract: `.ai/work-items/archive/WI-405-active-artifact-recovery.contract.json`
- Verification: `.ai/evidence/WI-405-active-artifact-recovery.verification.json`
- Installed Runtime: v0.2.40

## 境界

この Work Item は過去の evidence を書き換え・削除せず、release automation や
既存の Work Item decision の意味も変更しません。
