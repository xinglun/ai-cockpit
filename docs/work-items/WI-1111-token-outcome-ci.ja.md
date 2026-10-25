---
author: AI Cockpit maintainers
title: "WI-1111-token-outcome-ci — Usage intake and Outcome parity"
description: "Source quality and separately authorized later delivery."
audience:
  - maintainer
  - reviewer
workItemId: WI-1111-token-outcome-ci
status: in_progress
authority: native-contract
lastVerifiedBy: WI-1111-token-outcome-ci
---

# WI-1111-token-outcome-ci — 使用量の取込み、Outcome の整合と有限 CI 高速化

実装計画はネイティブ Contract に従います。実際のユーザー確認と checkpoint 1 を記録済みで、必須12シナリオはすべて未検証です。テストや納品の成功は主張しません。

明示的に提供された completed Responses オブジェクトをネイティブ使用量の准入で取り込みます。応答ID、不変レシート、nullable カウンター、呼出元の出典を保持します。現在のホスト会話の使用量は取得できず、履歴を補造しません。累積 Codex JSONL を呼出ごとの使用量として加算しません。

JSON、Markdown、CLI/MCP の要約と完全 Outcome を三言語で一致させ、既知と不明を表示します。finish／archive の凍結バイトと別の close cutoff を保持します。

高コストの管理対象検証より前に、既存の文書予算、必須指針、関連ポリシー／workflow／テスト消費側を検査します。Windows の完全なコマンド集合を有限の隔離ジョブへ分割し、固定の集約チェック、警告ゼロ、全パッケージと source／tested 束縛を保持します。比較可能な実測前に効果を主張しません。

finish 前にソース品質証拠と独立レビューが必要です。公開、PR／CI／main 検証、復元可能なリソース最終化は別の後続義務で、適用される明示的許可に従います。実際の close は別途確認します。現在、公開 push、merge、release、本番有効化、共有 Runtime 置換は許可されていません。

現在の Contract：`.ai/work-items/active/WI-1111-token-outcome-ci.contract.json`。
以下は将来の義務であり、ファイルの存在を主張しません：

- `.ai/work-items/archive/WI-1111-token-outcome-ci.contract.json`
- `.ai/evidence/WI-1111-token-outcome-ci.verification.json`
- `.ai/decisions/WI-1111-token-outcome-ci.finalize.json`
- `.ai/decisions/WI-1111-token-outcome-ci.close.json`

過去の失敗、出力切捨て、不明な token 使用量、未測定の効果を保持します。
