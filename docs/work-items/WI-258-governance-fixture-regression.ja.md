---
author: AI Cockpit maintainers
title: "WI-258 — Governance fixture registry 回帰"
workItemId: WI-258-governance-fixture-regression
description: "pending parity 検証導入後も governance fixture を schema 完備に保つ。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-258-governance-fixture-regression
authority: canonical
---

# WI-258 — Governance fixture registry 回帰

## Intent

すべての governance-integrity fixture が空の pending parity registry を明示的
に生成するようにします。fixture builder の制御ファイル欠落でテストが失敗
してはいけません。

## Scope

変更範囲は fixture builder、対応する回帰テスト、三言語の Work Item/parity
投影に限定します。Runtime validator と本番の governance semantics は変更
しません。

## Acceptance

- 生成された各 fixture に regular な
  `docs/reference/pending-parity-registry.json` があり、明示的に pending
  entry を作るテスト以外では `schemaVersion: 1` と `entries: []` である。
- governance-integrity と pending-registry の正常系/ adversarial test が
  決定的な report で通過する。
- 実装と evidence は review 後に archive Contract、verification、finalization、
  close の記録へ結び付く。

## Evidence boundary

空 registry は fixture の基線であり、実際の Work Item が pending だという
宣言ではありません。pending registration を試すテストは entry を明示的に
作成し、identity、parity rows、lifecycle を引き続き検証します。

## Recovery boundary

WI-258 は immutable な履歴 delivery として保持します。Runtime close は
confirmed ですが、human decision は promotion gate が要求する canonical な
`approved` 値ではなく説明文でした。全記録を保存し、bounded successor
[WI-259](WI-259-close-decision-recovery.ja.md) が predecessor を Recovered として
投影します。WI-258 の `.ai` bytes は書き換えません。
