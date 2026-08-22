---
author: AI Cockpit maintainers
title: "人間向け Outcome"
description: "Work Item Outcome から人に引き渡す結果表示。"
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: outcome-dialog-acceptance
capabilityClaims:
  - human_outcome_handoff
---

# 人間向け Outcome

`ai-cockpit work-item outcome --repo <repository> --id <work-item>` は既定で
人間向けの handoff を表示します。機械処理用の安定した `OutcomeV2` が必要な
場合は `--json` を指定します。

表示順は、結果と状態、完了したこと、発見された問題、発動した停止、解決した問題、
回避したリスク、残存リスク、不明点、人間の判断、検証と証拠、影響、次のアクションです。

状態マーカーは判断のシグナルであり、リリース承認ではありません。

- `🟢` 検証証拠が存在します。続行前に証拠を確認してください。
- `🟡` 部分完了、未準備、または不明です。修復または調査が必要です。
- `🔴` 必須の制御、権限、または範囲が無効です。停止して復旧してください。

空の章は `なし` と明示します。推論でガバナンス判断を補完することはなく、
緑の結果も merge、release、公開、安全性を承認するものではありません。

緑は、Runtime が `evidenceSchemaVersion=2` の検証証拠を読み取り、現在の Work Item
と repository に結び付いており、鮮度と digest が有効だと確認した場合だけ表示します。
証拠の欠落または snapshot の期限切れは黄色、改ざん・不正形式・identity 不一致・
digest 不一致は赤色です。同じ検証を `finish`、`archive`、`close` でも行い、証拠ファイル
が存在するだけでは成功にしません。旧形式の証拠は自動的に緑へ書き換えず、再検証が必要です。

受入れ基準、intent、scope などは Work Item owner が記述したガバナンス原文です。
表示では「受入れ基準（Contract 原文）」として保持し、Contract bytes を勝手に翻訳・変更
しません。Runtime が生成する固定見出し、状態、不明点、復旧案内だけを会話言語に合わせます。

CLI の直接出力は `AI_COCKPIT_LANGUAGE`、次にプロセス locale を使用します。Agent
の会話では利用者の言語で同じ handoff を表示します。JSON のフィールド名と enum
値は言語に依存せず安定しています。
