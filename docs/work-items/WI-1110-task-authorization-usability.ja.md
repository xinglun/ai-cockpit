---
author: AI Cockpit maintainers
title: "WI-1110-task-authorization-usability — 有限タスク権限と使いやすさ"
description: "Source-stage implementation and later conditional delivery remain separate."
audience:
  - maintainer
  - reviewer
workItemId: WI-1110-task-authorization-usability
status: in_progress
authority: native-contract
lastVerifiedBy: WI-1110-task-authorization-usability
---

# WI-1110-task-authorization-usability — 有限タスク権限と使いやすさ

実装中。sequence5 の実際の計画確認を記録済みです。その時点の必須17シナリオは未検証で、以後の結果はネイティブ証拠に従います。この管理用 Work Item と共有 Runtime で候補権限ポリシーは有効化しません。

権限をリポジトリ、Work Item、目的、パス、操作、検証基準、明示的な呼出元と出典に結び付けます。完了、撤回、宣言済み期限の最初の時点で将来の権限を終了します。ソース変更は再検証を必要としても自動的に再承認を要求しません。

既知のクロスプラットフォーム警告を修正し、Linux／Windows の正確な警告ゼロ品質チェックを追加します。staged CLI／MCP 開始は完全な宣言を preflight／checkpoint より前に保存します。既存の厳密なレビューとネイティブ履歴の検証を保持します。

finish 前にソース段階の実検証と独立レビューが必要です。最終 HEAD の Windows／Linux CI、通常の PR マージ、実際の main 検証、復元可能なリソース整理は後続の納品義務です。実際の close は別途確認します。

token 使用量は不明、導入効果は未測定、過去の失敗を保持します。リリース、本番有効化、外部 B、共有 Runtime の置換は含みません。

Current native Contract: `.ai/work-items/active/WI-1110-task-authorization-usability.contract.json`.

Planned terminal records (not asserted present):

- archive `.ai/work-items/archive/WI-1110-task-authorization-usability.contract.json`
- verification `.ai/evidence/WI-1110-task-authorization-usability.verification.json`
- finalization `.ai/decisions/WI-1110-task-authorization-usability.finalize.json`
- close `.ai/decisions/WI-1110-task-authorization-usability.close.json`
