---
author: AI Cockpit maintainers
title: "Human Benefit レポート"
description: "1 つの Task Outcome を evidence から人間向けに要約する projection。"
audience:
  - adopter
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-323
capabilityClaims:
  - human_benefit_report
---

# Human Benefit レポート

## この機能でできること

人へ短く、しかし根拠のある答えを渡したい時に使います。何を完了し、何を見つけ、
何を停止し、何を解決し、どの risk が残り、何が不明で、次に何をするかを示します。
これは validated `OutcomeV2` の human-facing projection であり、第二の authority
source ではありません。

## Agent に handoff を依頼する

repository-bound CLI route は次のとおりです。

```sh
ai-cockpit work-item outcome --repo <repository> --id <work-item-id>
```

MCP を使う Agent は repository context を付けて `work_item_outcome` を呼び、
返された `humanHandoff` を人に提示します。`finish`、`archive`、`close` は
lifecycle JSON を stdout に保持し、既定では同じ handoff を stderr に表示します。
CLI は Cursor の chat panel を展開できないため、Agent または provider adapter が
handoff を表示するか、`work-item outcome` を再生してください。

Runtime は生成した label、status、unknown、decision、next action を localize します。
Contract intent と acceptance criteria は authored language のままで、governance fact
として機械翻訳しません。未宣言の user benefit は unknown のままです。このページは
source 固有の `implementation_approach_report` capability を claim しません。

自動処理では `--json` を付けて安定した machine-readable `OutcomeV2` と任意の
`taskOutcomeReport` を取得します。この mode は human handoff を抑止するだけで、
fact や lifecycle authorization を変更しません。

## 結果の形式

人の判断に使いやすいよう、handoff は次の順で表示します。

```text
Task Result
Status: Success / Partial / Blocked / Failed

What was completed
Problems found
Stops triggered
Problems resolved
Risks avoided
Remaining risks
Unknowns
Human decisions
Verification
Impact
Next action
```

finding、warning、risk、forced stop の数は evidence record の数です。生産性、時間、
金額、security、trust score ではありません。Green は current Contract/Summary/evidence
binding と verified Runtime fact を要求し、Yellow は調査または確認、Red は required
control の失敗による停止です。短い要約でも review を省略できません。

## 使用例

不足していた capability overview link を追加した場合、valid handoff は次のようになります。

```text
Completed: 不足していた capability overview link を追加した。
Resolved problem: docs entry から capability overview に到達できる。
Evidence: Contract、変更 file、passed した documentation-link check。
Remaining risk: Hosted provider review は未確認。
Next action: PR を review し、provider result を待ってから merge する。
```

根拠のない benefit は Runtime が `unknown` または `inference` と表示します。
Agent は prose を completed fact に変えてはいけません。Contract、evidence、decision
bytes は Runtime が生成するため手編集しません。

## 欠落、stale、無効な場合

report が欠落、malformed、stale、別 Work Item、foreign repository、contradictory、
archive Outcome と不一致の場合、source Outcome を停止して検証します。Runtime で
Contract/evidence を修正し、projection を再生成してください。report を手編集して
complete に見せてはいけません。historical evidence は history のままであり、current
verification pass/failure として表示しません。

## Lifecycle と責任境界

report は repository-local の `.ai/` record から次の明示的 lifecycle で生成されます。

```text
start → preflight → checkpoint → verify → finish → archive → close
```

`work-item outcome` は Agent と人の公式 replay route です。`work_item_get` は
machine-oriented lookup であり、human handoff の代替ではありません。Runtime が fact、
validation、localization を担当し、provider/Agent が conversation presentation を担当
します。PR 作成、Hosted CI、merge、branch cleanup、platform isolation、enterprise
compliance、production safety は別途 bound された external evidence がない限り、この
report では証明されません。

[Task Outcome Report](task-outcome-report.ja.md) | [Outcome reference](../reference/outcome-report.ja.md) |
[English](human-benefit-report.md) | [中文](human-benefit-report.zh-CN.md)
