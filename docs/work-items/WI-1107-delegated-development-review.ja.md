---
author: AI Cockpit maintainers
title: "WI-1107 — 完全な開始計画と明示的な委任開発レビュー"
description: "Complete initial scenario plans and finite explicit agent technical review; production activation excluded."
audience:
  - maintainer
  - reviewer
workItemId: WI-1107-delegated-development-review
status: implemented
authority: work-item-record
lastVerifiedBy: WI-1107-delegated-development-review
terminalArchive: .ai/work-items/archive/WI-1107-delegated-development-review.contract.json
terminalVerification: .ai/evidence/WI-1107-delegated-development-review.verification.json
terminalFinalization: .ai/decisions/WI-1107-delegated-development-review.finalize.json
terminalDecision: .ai/decisions/WI-1107-delegated-development-review.close.json
---

# WI-1107 — 完全な開始計画と明示的な委任開発レビュー

CLI `start --scenario-coverage <json-file>` は既存の型
`ScenarioCoverageEntry` の配列を受け取る。MCP `work_item_start.scenarioCoverage` も同じ型と
リポジトリ検証を使う。初期状態は unverified、evidence は空で、最初の preflight 前に保存する。
不正入力ではライフサイクル記録を作らない。省略時は従来の形を維持する。

CLI `start --development-review <json-file>` と MCP `developmentReview` は
`governanceProfile.developmentReview` の明示的な Work Item ポリシーを選ぶ。既定は strict のまま。
選択を含む正確な基準 Contract に対する正式なネイティブ所有者レビューが必要。
actor や出典文字列、任意の承認ファイルだけでは権限にならない。

```json
{"schemaVersion":1,"reviewerActor":"agent:reviewer","authoritySource":"user:explicit-bounded-delegation","allowedChanges":["append_scenarios","fill_verification_plan","append_verification"],"preapprovedCommands":["cargo test --locked"]}
```

typed amendment の任意 `developmentReview` は AuthorityEvidence を再利用する。
明示 agent／出典、`assurance=self_declared`、唯一の操作 `review_development_amendment` が必要。
許可は三種類のみ：必須・未検証・証拠なしで正確な宣言済み／事前承認コマンドを使うシナリオの追加、
既存の空の verificationPlan への同種コマンドの補完、有限集合の正確な事前承認コマンドの追加。
既存の順序と検査を保つ。scope、権限、リスク、ポリシー、受入基準、任意コマンド変更は従来の人間確認／拒否に従う。

Runtime は既存ロック内で別の `delegatedReview` を prepared／committed 履歴に保存し、
所有者基準と記録、ポリシー、repo／WI、要求、前後 Contract、ソース snapshot、Runtime を結び付ける。
欠落・改名・改変・別項目・不完全な記録は拒否する。元の所有者記録は不変で、agent レビューは
`decisionEvidence` にならず、close、Finding、失敗検証、強い人間／provider ポリシー、宿主／GitHub 承認を満たさない。

これはローカル出典であり外部本人認証ではない。全ローカル記録の書換えに対する保証もない。
共有 Runtime と本番設定は変更しない。本番利用には別途明示選択と新しい所有者レビューが必要で、agent の技術判断は誤り得る。
WI 間の期限／取消と外部 B は対象外。隔離テストでの確認回数削減は実際の利益測定ではない。

範囲は本番ソース9件、テスト4件、三言語文書12件とこの WI のネイティブ記録。統制には開発中の機能ではなく固定の正式 v1.0.1 b2 を使う。12シナリオと9コマンドの実検証、独立した全分岐レビューが必要で、RED／失敗は保持する。公開、merge、実際の close には対応する承認が必要。本片は WI-1087 全体の完了を意味しない。
