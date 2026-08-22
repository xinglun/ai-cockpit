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

先頭行は常に `Outcome: 🔴/🟡/🟢 ...` です。CLI stdout と MCP の
`content[0].text` が handoff を直接返すため、Agent や UI は折りたたんだログに
隠してはいけません。`work_item_status` は別の read-only status projection です。
archive 後の lifecycle phase は `archived` で、repository に bind された confirmed
close decision が有効な場合だけ `closed` になります。欠落または不正な decision は
archive を `closed` に昇格させません。

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
現在の CLI は `verify`/`finish`/`archive`/`close` を実行する Runtime の
`runtimeVersion` と `runtimeDigest` を証拠に bind します。そのため、別 Runtime が作った
形式上正しい証拠も拒否されます。v2 envelope と保存された receipt は unknown field を拒否し、
Work Item、repository、Runtime の nested identity を要求します。`digest_only` retention には
検証可能な captured receipt がありません。読み取り可能な pre-v2 record（
`evidenceSchemaVersion` がないもの）は黄色の `legacy_evidence_historical` として表示します。
これは履歴入力であり、現在の失敗でも fresh green でもありません。v2 record の identity 欠落は
引き続き赤色です。

v2 envelope の `createdAt` と retention の `createdAt` は RFC3339 timestamp でなければなりません。
任意の retention `expiresAt` は RFC3339 または互換性のための epoch seconds 形式を受け付けます。形式または意味が不正な
timestamp は証拠の破損として赤色にし、`finish`、`archive`、`close` を停止します。
この検査は現在の証拠と retention metadata を保護しますが、過去の bytes は書き換えません。

受入れ基準、intent、scope などは Work Item owner が記述したガバナンス原文です。
表示では「受入れ基準（Contract 原文）」として保持し、Contract bytes を勝手に翻訳・変更
しません。Runtime が生成する固定見出し、要約、状態、不明点、復旧案内だけを会話言語に合わせます。

CLI の直接出力は `AI_COCKPIT_LANGUAGE`、次にプロセス locale を使用します。Agent
の会話では利用者の言語で同じ handoff を表示します。JSON のフィールド名と enum
値は言語に依存せず安定しています。

## MCP の human handoff

Agent が人間に結果を示す場合、明示的な `workItemId` を指定して repository-bound の
`work_item_outcome` を呼び出します。text content は CLI と同じ localized handoff であり、raw JSON dump
ではありません。`structuredContent.outcome` は安定した OutcomeV2 object のままです。
`humanHandoff` は presentation projection であり、merge、release、human decision を認可しません。
`work_item_get` は machine record lookup です。任意の `language` で `en`、`zh`、`ja` の Runtime label を
選択できますが、Contract source text は変更されません。
