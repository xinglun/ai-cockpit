---
author: AI Cockpit メンテナー
title: WI-1033 — Work Item 横断レビュー修正の recovery
description: 選択された cross-WI review-fix lineage の Runtime-bound 受け入れと lifecycle closure を完了する。
workItemId: WI-1033-cross-wi-review-fixes-recovery
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-1033-cross-wi-review-fixes-recovery
---

# WI-1033 — Work Item 横断レビュー修正の recovery

## Intent

同じ限定された Work Item 横断レビュー修正を引き継ぎ、Runtime verification、独立した PR review、merge、正確な cleanup まで進めます。Release 前で停止します。

## Lineage と現在の状態

WI-1033 は WI-1032 の Runtime-bound recovery successor です。Runtime は WI-1032 を verification claim not_verified の replaced として archive し、元の source record と失敗した precondition evidence を保持しました。実装 commit は存在しますが、WI-1033 自身の Contract-bound verification が必要です。既存のローカルテスト報告は Runtime verification receipt ではありません。

先行する WI-1031 は有効な historical verification を持ちますが、human close は未完了です。履歴 bytes は不変のままとし、Runtime が終端 evidence をそろえた場合だけ selected successor lineage を処理します。

独立 review が directory-handle swap race を見つける前に、Runtime canonical verification は commit `6f5c4aaf` で 34/34 node を通過しました。この receipt は履歴 evidence であり、現在の tree には bind されません。Contract は 24 acceptance criteria と 20 scenario になり、Windows 専用の rename-denial regression を CI で実行することも求めます。macOS の format check と repository library test は 35/35 pass し、3 件の directory-swap regression を含みます。gate-manifest regression は Windows workflow step の追加前に想定どおり失敗し、追加後に pass しました。Windows の動作と workflow 変更は hosted CI 実行まで未検証です。現在の evidence freshness は Runtime を参照してください。完全な canonical Runtime verification、hosted CI、merge、正確な cleanup は未完了で、Release は主張しません。

## Scope

- 信頼できる registration/composition identity（execution repository と CoordinationStore の Git common directory 一致を含む）、必須 check の完全性、crash と live-lock の安全性、evidence containment、provider 単位の dependency、CLI の実際の reuse、MCP identity parity、read-only query/write 境界を再検証します。
- 実プロセスの複数 linked-worktree 動作を証明し、通常の単一 WI serial execution を維持します。
- provider output の削除後も依存 chain に invalidation を伝播し、安全でない request や初期状態を bypass する request を拒否します。Contract facts を参照する前に読み込んだ registration identity を target に束縛します。
- 候補 CLI で Sentinel を検査します。source、Contract/evidence、lifecycle Runtime、coordination store には書き込みません。
- 既存の Windows CI job で repository library の containment regression を実行し、gate-manifest regression でその step を固定します。
- 英語、簡体字中国語、日本語の Work Item projection と reference parity を維持します。

## 対象外

Task 8（commit ごとの Runtime snapshot binding）は指定どおり後続の独立した serial WI に分けます。Runtime upgrade、Release 準備、tag 変更、public publication、Sentinel への書き込み、履歴の書き換えは対象外です。

## 受け入れと検証

[仕様](WI-1033-cross-wi-review-fixes-recovery/spec.md)と[実施計画](WI-1033-cross-wi-review-fixes-recovery/implementation-plan.md)を参照してください。必須 20 scenario には、明確な expected result と verification plan を宣言します。canonical documentation gate は `bash tests/docs/documentation_acceptance.sh` です。

## Delivery boundary

Work Item は進行中です。現在の verification freshness は Runtime の照会のみを根拠とし、以前の receipt は歴史 evidence として保持します。完了には最終 Runtime verification、独立 review、hosted CI、merge、正確な cleanup、および Runtime が認める historical lineage close が必要です。Release は対象外です。
