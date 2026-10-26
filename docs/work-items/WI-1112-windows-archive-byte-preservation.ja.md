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
- 製品変更は `.gitattributes` と `crates/cockpit-repository/src/lib.rs` に限定します。英語・日本語・中国語の本ページと reference-parity 台帳を同期します。CI workflow、global Git 設定、WI-1111 のアーカイブ記録は変更しません。

## 検証計画

空白を含む local repository path、autocrlf を有効にした checkout での固定履歴 byte、1 byte の evidence 変更後の拒否を対象にした回帰テストを追加します。対象 repository test の後、Contract が宣言する workspace verification と warning-free Clippy を実行します。後続の許可された run が最終納品 HEAD で実行されるまでは、hosted Windows CI を検証済みとは主張しません。

## 現在の状態

検証済みで archived の WI-1111 から successor が作成され、その Windows repository-library job は失敗しました。実装と検証結果はまだありません。前任の archive と失敗した hosted run は変更していません。
