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

独立 review が directory-handle swap race を見つける前に、Runtime canonical verification は commit `6f5c4aaf` で 34/34 node を通過しました。この receipt は履歴 evidence であり、現在の tree には bind されません。その後 `faa3eb4` で 35/35 node を通過しましたが、必須 check の環境 overlay に関する Contract amendment より前の receipt のため、現在の証拠ではありません。独立 review は、未束縛の環境 overlay によって一致する必須 command が `forged-pass` で成功できることを確認しました。追加 regression はこの問題を再現し、限定的な拒否修正後に admission suite は 38/38 pass しました（Contract amendment 前）。現在の Contract は 42 acceptance criteria と 37 の必須 scenario を持ち、Windows 専用 rename-denial regression の CI 実行と checkpointed stale-preflight recovery も含みます。Hosted CI run `36215373771` は head `b57068c` で quality scope と Windows runtime job が失敗しました。ローカルの Contract 追加は不足していた regression file と canonical-root alias case を対象にしています。現在の evidence freshness は Runtime を参照してください。修正後の完全な canonical verification、成功した hosted CI、独立 review、merge、正確な cleanup は未完了で、Release は主張しません。

## Scope

- 信頼できる registration/composition identity（execution repository と CoordinationStore の Git common directory 一致を含む）、必須 check の完全性と環境 binding（Contract に宣言されていない caller overlay を拒否）、crash と live-lock の安全性、evidence containment、provider 単位の dependency、CLI の実際の reuse、MCP identity parity、read-only query/write 境界を再検証します。
- 実プロセスの複数 linked-worktree 動作を証明し、通常の単一 WI serial execution を維持します。
- checkpointed Work Item の preflight が未記録または stale の場合は `run_preflight` を提示して `run_verification` を保留し、更新後に検証を許可します。既に承認された開始に二度目の確認を追加しません。
- provider output の削除後も依存 chain に invalidation を伝播し、安全でない request や初期状態を bypass する request を拒否します。Contract facts を参照する前に読み込んだ registration identity を target に束縛します。
- 候補 CLI で Sentinel を検査します。source、Contract/evidence、lifecycle Runtime、coordination store には書き込みません。
- 既存の Windows CI job で repository library の containment regression を実行し、gate-manifest regression でその step を固定します。
- 英語、簡体字中国語、日本語の Work Item projection と reference parity を維持します。

## 対象外

Task 8（commit ごとの Runtime snapshot binding）は指定どおり後続の独立した serial WI に分けます。Runtime upgrade、Release 準備、tag 変更、public publication、Sentinel への書き込み、履歴の書き換えは対象外です。

## 受け入れと検証

[仕様](WI-1033-cross-wi-review-fixes-recovery/spec.md)と[実施計画](WI-1033-cross-wi-review-fixes-recovery/implementation-plan.md)を参照してください。必須 37 scenario には、明確な expected result と verification plan を宣言します。canonical documentation gate は `bash tests/docs/documentation_acceptance.sh` です。

## Delivery boundary

Work Item は進行中です。現在の verification freshness は Runtime の照会のみを根拠とし、以前の receipt は歴史 evidence として保持します。完了には最終 Runtime verification、独立 review、hosted CI、merge、正確な cleanup、および Runtime が認める historical lineage close が必要です。Release は対象外です。
