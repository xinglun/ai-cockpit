---
author: AI Cockpit maintainers
title: "WI-740 — P1 協作不変量カバレッジ対応付け(再送出)"
description: "複数エージェントのWI番号衝突後にWI-681の不変量カバレッジ対応付けを再送出し、不変量8の判定を修正し、不変量5・9が既に独立して解消済みであることを反映する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-740-p1-invariant-coverage-mapping
status: implemented
authority: authorized
lastVerifiedBy: WI-740-p1-invariant-coverage-mapping
terminalArchive: .ai/work-items/archive/WI-740-p1-invariant-coverage-mapping.contract.json
terminalVerification: .ai/evidence/WI-740-p1-invariant-coverage-mapping.verification.json
terminalFinalization: .ai/decisions/WI-740-p1-invariant-coverage-mapping.finalize.json
terminalDecision: .ai/decisions/WI-740-p1-invariant-coverage-mapping.close.json
---

[English](WI-740-p1-invariant-coverage-mapping.md) · [简体中文](WI-740-p1-invariant-coverage-mapping.zh-CN.md)

# WI-740 — P1 協作不変量カバレッジ対応付け(再送出)

## 意図

当初 WI-681 として試みた協作不変量カバレッジ対応付けを再送出する。
WI-681 はマージされずにクローズされた。理由は、真に発生した複数エージェ
ント間の WI 番号衝突であり(別の並行稼働エージェントが、無関係な文書昇
格 Work Item に対して独立に同じ短い番号 `WI-681` を使用し、PR #677 で先
にマージされた)、内容上の欠陥ではない。今回の再送出では、あわせて一件
の判定を修正する:不変量8(セッション切替をまたいだ授権適用性)は当初
「自動テスト未発見」とされていたが、より注意深く読み直した結果、
`crates/cockpit-repository/tests/preflight_review.rs::bound_human_review_receipt_allows_checkpoint_but_not_stale_reuse`
が既にその核心的な主張を証明していることが判明した。元の草稿が正しく欠
落として指摘していた不変量5と9は、この再送出より前にそれぞれ WI-682
(PR #679)と WI-710(PR #702)によって独立に解消されたため、本ページは
古い欠落一覧ではなく、この現状を反映する。リポジトリ所有者からの明示的
な委任に基づき、AI Cockpit 協作言語専項を継続する。

## 境界

これは文書のみの Work Item である。
`docs/reference/collaboration-invariant-coverage.md`(+ zh-CN/ja)、
`docs/reference/README.md`(+ zh-CN/ja)への索引エントリ1件、本記録ペー
ジ、および自身の reference-parity 登録を追加する。テストファイル、
CLI/MCP の挙動、Contract/Outcome スキーマはいずれも変更しない。不変量7
のテストを書くことは明示的に対象外であり、納品文書内で境界の明確な後続
Work Item として明記する。

## 受け入れとライフサイクル

- 不変量1、2、3、4、5、6、8、9、10 はすべて(完全または部分的に)自動化
  済みと述べ、それぞれが納品時点で現在のテストソースを直接読んで確認し
  たテストファイルと関数を引用する。不変量7は唯一残る、名指しされた欠落
  として述べる。
- 文書は、本ページがなぜ WI-681 に取って代わるのか、その草稿以降に何が
  変わったのかを明記する。
- `start → preflight → checkpoint → verify → finish → archive → close` が統治
  された経路であり、`user_visible_benefit_not_declared` は明示されたまま
  である。
- 精査対象の head 上で `bash tests/docs/documentation_acceptance.sh` が合格
  する。

## 証拠

- archive: `.ai/work-items/archive/WI-740-p1-invariant-coverage-mapping.contract.json`
- verification: `.ai/evidence/WI-740-p1-invariant-coverage-mapping.verification.json`
- finalization: `.ai/decisions/WI-740-p1-invariant-coverage-mapping.finalize.json`(合併後に生成予定)
- close: `.ai/decisions/WI-740-p1-invariant-coverage-mapping.close.json`(合併後に生成予定)
