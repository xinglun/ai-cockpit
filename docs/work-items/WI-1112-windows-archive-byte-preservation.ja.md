---
author: AI Cockpit maintainers
title: "WI-1112-windows-archive-byte-preservation — Windows archive byte preservation"
description: "バイト完全性検査を弱めず、Windows のローカル Git fixture と不変 evidence の checkout 失敗を修正します。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1112-windows-archive-byte-preservation
status: in_progress
authority: native-contract
lastVerifiedBy: WI-1112-windows-archive-byte-preservation
---

# WI-1112 — Windows archive byte preservation

この後継項目は [PR #1074](https://github.com/xinglun/ai-cockpit/pull/1074) の Windows repository-library job、run [40271040290](https://github.com/xinglun/ai-cockpit/actions/runs/40271040290/job/121404907259) で発生した 4 件の失敗に対応します。固定履歴を使う 2 件は Windows のローカルパスから repository を clone する際に失敗し、残り 2 件は `.ai` evidence が CRLF に変換された後の厳密な Markdown byte 比較に失敗しました。

## scope と境界

- `--shared --no-checkout` によるローカル Git clone の隔離を保ち、Windows path を remote transport URL と誤認しない移植可能な source 指定にします。
- `core.autocrlf=true` を含む current／historical fixture checkout で `.ai` governance artifact の実バイト列を保ちます。
- digest と byte equality の assertion は厳密なままにします。1 byte の変更は引き続き拒否し、checkout 変換を隠すために report 内容を normalize しません。
- active successor handoff に独立した versioned read-only delivery continuity section を追加します。bind 済み predecessor archive と recovery lineage を検証し、local archive と未完了の external validation を区別し、Runtime が admit した次の action だけを表示します。lineage が欠落または無効なら unknown のままにし、`OutcomeDelivery.deliveryState` は handoff transport state として保持します。
- read-only material plan は、archive receipt、request、source revision、repository、Contract lineage、および各 member の path/kind/cause/hunk/blob identity を検証した後だけ、predecessor が以前 review した正確な Unknown を表示します。これは参考情報のみです。現在の Unknown を解消せず、decision を移転せず、successor 自身の request も満たしません。
- 変更は active Contract に宣言された 27 paths に限定します。CI workflow、global Git 設定、WI-1111 の archive record は変更しません。

## 検証計画

空白を含む local repository path、autocrlf を有効にした current／historical checkout の byte、1 byte の evidence 変更後の拒否を対象にした回帰テストを追加します。さらに検証済み／破損した predecessor lineage、archive byte の保持、material identity の変更、predecessor context が現在の Unknown を解消しないことを検証します。repository、CLI、MCP の対象テスト後、Contract の workspace verification と warning-free Clippy を実行します。最終納品 HEAD 上の後続許可 run が成功するまでは hosted Windows CI を検証済みと主張しません。

## 現在の状態

検証済みで archived の WI-1111 から successor が作成され、その Windows repository-library job は失敗しました。Windows byte preservation、検証済み lineage の continuity、material reference identity、CLI/MCP successor handoff parity の各 test は成功しました。関連 repository material-review（41/41）、CLI material-review（4/4）、MCP material-review（2/2）、Outcome report（29/29）も成功しています。Contract 全体の verification と warning-free Clippy は Linux でローカル通過しました。hosted Windows CI は未実行であり、Windows hosted 成功とは報告しません。前任の archive と失敗した hosted run は変更していません。
