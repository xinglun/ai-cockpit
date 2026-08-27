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

top-level の `finish`、`archive`、`close` は既存の stdout lifecycle JSON を保持し、
既定では同じ検証済み report を stderr に render します。明示的な `--json` mode は
機械専用 caller のため stderr report を抑止します。`finish` が block された場合、
永続化済みの赤または黄の Outcome を表示してから元の nonzero error を返します。
追加の handoff が gate を弱めることはありません。CLI は host application に会話 UI
の表示・展開を強制できません。host は stderr を提示し、人は
`ai-cockpit work-item outcome --repo <repository> --id <work-item>` で durable handoff
を決定的に再生できます。

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

古い Runtime が生成した archived v2 evidence は、黄色の履歴マーカーと
`historical_evidence_not_revalidated` で表示します。handoff に
`verification_or_human_input` や missing-evidence の recovery gate を追加してはいけません。
これは現在の verification failure ではなく、有効な historical context です。current result
が必要な場合だけ新しい verification を実行します。
machine projection では `historicalStatus: "runtime_historical"` を使用し、人間向け handoff でも missing-evidence と recovery の案内を表示しません。

v2 envelope の `createdAt` と retention の `createdAt` は RFC3339 timestamp でなければなりません。
任意の retention `expiresAt` は RFC3339 または互換性のための epoch seconds 形式を受け付けます。形式または意味が不正な
timestamp は証拠の破損として赤色にし、`finish`、`archive`、`close` を停止します。
この検査は現在の証拠と retention metadata を保護しますが、過去の bytes は書き換えません。

受入れ基準、intent、scope などは Work Item owner が記述したガバナンス原文です。
表示では「受入れ基準（Contract 原文）」として保持し、Contract bytes を勝手に翻訳・変更
しません。Runtime が生成する固定見出し、要約、状態、不明点、復旧案内だけを会話言語に合わせます。

predecessor に明示的な `supersede` recovery decision がある場合、Outcome は
`historicalStatus: "superseded"` を含み、黄色の履歴マーカーを表示します。
これは元の evidence を保持し、現在の結果として再検証していないことを示します。
赤い失敗でも緑の認可でもありません。

resource context を持つ通常の archived Work Item で provider finalization
receipt が欠落または無効な場合、Outcome は stable unknown
`resource_finalization_pending` を追加し、green/verified にはなりません。
この receipt は repository verification とは別の provider-side 境界です。

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

## Task Outcome report と event

新しく生成された OutcomeV2 には strict な `taskOutcomeReport` も含まれます。section
は evidence に束縛され、空でもよいですが、空であることは成功を意味しません。
repository-local evidence reference がない claim には `inference: true` が必要です。
必須の control が yellow または red の場合は `failedGate` と `recoveryCondition` を含めます。

`finish` がブロックされた場合、active Work Item は checkpointed の lifecycle state を保持し、
active な `state: "blocked"` Outcome projection が記録されます。この projection は現在の
repository と Work Item に bind され、`decisionState: "red"` と失敗した gate、決定的な復旧条件を
示します。その後の有効な retry は completion event を追加するだけで、先行する blocked event を
書き換えません。不正形式、外部 identity、symlink、未知の event type は fail closed になります。

失敗した `finish` projection の後に retry する場合、Runtime は identity-bound recovery receipt
を通じて active Summary を `checkpointed` に戻します。blocked Outcome を green にはせず、
archive または close の前に `verify` と `finish` を再実行して新しい current Outcome を生成する
必要があります。

`finish` は active outcome と同じ場所に `<id>.events.jsonl` を書きます。event stream は
append-only で、malformed、foreign、secret らしい内容、関係不正の event を拒否します。
archive の作成時には、manifest を束縛する前に、生成された report reference と
`changedPaths` を `.ai/work-items/active/` から対応する
`.ai/work-items/archive/` へ投影します。`eventsDigest` と report digest は投影後の
archive bytes を対象にします。`close` は投影済み stream を検証し、close receipt へ
`finalReport` と `finalReportDigest` を記録します。既存の historical archive bytes は
書き換えず backfill もしません。この active から archive への投影は新規 archive のみで行います。
