---
author: AI Cockpit maintainers
title: "WI-664 — ライフサイクル状態遷移境界"
workItemId: WI-664-state-transition-boundary
description: "ライフサイクルの合法な遷移を明示し、ライフサイクル・証拠・ガバナンス判断・権限・履歴投影を混同しない。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-664-state-transition-boundary
terminalArchive: .ai/work-items/archive/WI-664-state-transition-boundary.contract.json
terminalVerification: .ai/evidence/WI-664-state-transition-boundary.verification.json
terminalFinalization: .ai/decisions/WI-664-state-transition-boundary.finalize.json
terminalDecision: .ai/decisions/WI-664-state-transition-boundary.close.json
authority: canonical
---

# WI-664 — ライフサイクル状態遷移境界

## 目的

ライフサイクル状態の変更を明示的かつ fail-closed にしながら、ライフサイクル
状態、証拠、ガバナンス判断、人の権限、履歴投影の型の分離を維持する。

## 変更前

`cockpit-core::WorkItemState` はシリアライズ可能な語彙に留まり、検査済みの
遷移操作を持たなかった。直接のテストも enum の同値性だけだったため、合法な
遷移の規則を呼び出し側が個別に再現する余地があった。`DecisionState`、
`AuthorityState`、`EvidenceState` は既に独立していたが、ライフサイクル遷移が
それらの事実を生成しないことを示す純粋な境界がなかった。

## 変更後

`WorkItemState::can_transition_to` が確認済みの通常遷移と回復遷移を定義し、
`transition_to` は後続状態または型付きの `WorkItemTransitionError` を返す。
通常経路は次の通りである。

`Created → PreflightReady → ImplementationActive → VerificationPending → FinishReady → Archived → Closed`。

Paused は `ImplementationActive` へ、Blocked または Stale は
`PreflightReady` を経由して回復するため、回復時に新しいチェックを省略しない。
終端状態からの遷移はない。これらのメソッドは純粋なドメイン判定であり、
ファイル、Git、プロセス、永続化、権限の推測、証拠の生成を扱わない。

## 契約と互換性

- `finish_ready`、`closed` を含む既存の `snake_case` serde 値を変更しない。
- protocol、repository、verification、ファイル配置、履歴レコードの形式を変更しない。
- `cockpit-core` への追加 API であり、観測、権限、証拠、書込み順序は呼び出し側の責任である。
- 履歴レコードを内部の遷移語彙に合わせて書き換えない。

## P3 物理実行の監査

本 WI は P3 の実装を変更しない。既存の verification 境界では
`PhysicalExecutionKey` が WI の識別子から独立し、その結果を
`WorkItemEvidenceReceipt::bind` と `validate_for` が特定 WI に結び付ける。
物理実行テストは、異なる WI の receipt、外部 receipt の拒否、key 不一致、
改ざん、外部 execution result を検証している。したがって、キャッシュ命中や
共有実行結果だけで WI の権限や合格状態を与えることはない。

## 検証と残余リスク

焦点化した `cockpit-core` テストは、通常・スキップ・逆行・回復・終端の遷移、
旧 serde 値、未知値の拒否を検証する。終了前に workspace テスト、Runtime 検証、
ガバナンス検査、hosted PR 検査も通す必要がある。遷移表は意図的に限定しており、
すべての証拠・ポリシーの組合せを巨大な enum にしない。永続化前には呼び出し側が
現在の観測事実とガバナンス事実を検証しなければならない。

