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
読者優先の四つのセクションを持つ summary を表示します。完全な audit handoff
は `--view full`、機械処理用の安定した `OutcomeV2` は `--json` を指定します。
summary と full view は presentation projection であり、保存された Outcome
や governance の意味を変更しません。

先頭行は常に `Outcome: 🔴/🟡/🟢 ...` です。たとえば緑は汎用的な成功ではなく
`Outcome: 🟢 宣言された検証済み` と表示されます。CLI stdout と MCP の
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

既定の summary は四つのセクションです。

1. 結果：現在の検証、ライフサイクル、人間の判断、ガバナンスシグナル
2. 主な変更：evidence-backed な完了事項
3. 残る不確実性：blocker、リスク、制限、不明点、未宣言の効果
4. 人間の次のアクション：必要な判断とその理由、または新しい判断が不要であること

full view は audit 向けの順序を保持します。結果と検証・ライフサイクル・人間の判断・ガバナンスシグナルを分けて示し、完了したこと、発見された問題、発動した停止、解決した問題、
回避したリスク、残存リスク、不明点、人間の判断、検証と証拠、影響、次のアクションを含みます。

summary では判断に関係しない空の章を省略します。blocker、未処理の人間の判断、無効または期限切れの evidence、履歴分類、不明点は長さ制限で省略しません。
その他の一覧を黙って切り詰めることはなく、完全な一覧が必要な場合は full view を使用します。summary の主張は evidence-bound data として扱い、元の evidence テキストを instruction や権限の出所として解釈しません。

## Release note: reader-first Outcome summary

今回の presentation release では、人間向け `work-item outcome` の既定 view を完全な audit report から上記の四つの summary へ変更しました。通常のタスクで空の章を読む負担を減らしつつ、検証、ライフサイクル、判断、blocker、不確実性、evidence 参照を残します。
完全な report は `--view full` で表示でき、MCP `work_item_outcome` は `view: "summary"`（既定）または `view: "full"` を受け付けます。
machine JSON、検証ルール、exit code、権限 semantics、永続化された evidence は変更しません。top-level の `finish`、`archive`、`close` は lifecycle error と audit context のため stderr に完全な handoff を表示し続け、`--json` は従来どおり人間向け channel を抑止します。

状態マーカーは判断のシグナルであり、リリース承認ではありません。

- `🟢` 検証証拠が存在します。続行前に証拠を確認してください。
- `🟡` 部分完了、未準備、または不明です。修復または調査が必要です。
- `🔴` 必須の制御、権限、または範囲が無効です。停止して復旧してください。

空のデータを肯定的な事実として扱いません。証拠が足りない場合、リスク所見は
`未記録` または `未評価` と表示します。指定された検査範囲でリスクが見つからなかったと
言えるのは、明示的な evidence-backed claim がある場合だけです。その他の空の章は
`未記録` と表示します。推論でガバナンス判断を補完することはなく、緑の結果も merge、
release、公開、安全性を承認するものではありません。

レポートは四つの軸を分けて表示します。

- 検証状態は `OutcomeState` を示します。例: `宣言された検証済み`。
- ライフサイクル状態は実装中、チェックポイント、finish 準備完了、archive、close の投影を示します。
- 人間の判断は `未記録`、`記録済み: <判断>`、または `不明` であり、検証状態から推測しません。
- ガバナンスシグナルは緑・黄・赤のシグナルであり、人間の承認ではないことを明示します。

有効な構造化された人間の判断には、実行者、権限の出所、evidence と policy の参照、保証レベルも表示します。
保証レベルの事実がない場合は `不明` とし、表示層が権限の出所や evidence を高い保証レベルへ昇格させることはありません。

テスト弱化の表示は検査範囲に限定します。弱化ルールが発動していないことは、参照された検査範囲で
発動が記録されなかったことだけを示し、テストが弱化されていない証明ではありません。

緑のマーカーは、Runtime が `evidenceSchemaVersion=2` の検証証拠を読み取り、現在の Work Item
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
archived Work Item は明示的な close decision が有効になるまで terminal ではありません。
Human handoff は、対象の branch/worktree を cleanup し、finalization を記録し、
`finalize-verify` を実行してから `close` する順序を示します。外部 resource がない
item は review 済みの人間の `close` 判断だけが必要です。machine status projection
はこの gap を blocker として示し、対応する `safeActions` を返すため、Agent が
静かに次の Work Item へ進むことはできません。

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
