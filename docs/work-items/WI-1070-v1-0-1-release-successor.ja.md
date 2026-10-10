---
author: AI Cockpit maintainers
title: "WI-1070 — v1.0.1 stable release successor"
description: "承認済みの WI-1068 parity 行を修正し、三言語 Work Item 文書を追加して、段階的な v1.0.1 release acceptance を維持します。"
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-1070-v1-0-1-release-successor
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1070-v1-0-1-release-successor
contractDigest: sha256:b92b2eecb57e68f7943b54f67bca249158d31c55824146c47827cb8ea43f2657
---

[English](WI-1070-v1-0-1-release-successor.md) · [简体中文](WI-1070-v1-0-1-release-successor.zh-CN.md)

# WI-1070 — v1.0.1 stable release successor

このページは現在の [WI-1070 Contract](../../.ai/work-items/active/WI-1070-v1-0-1-release-successor.contract.json) の読者向け投影であり、`sha256:b92b2eecb57e68f7943b54f67bca249158d31c55824146c47827cb8ea43f2657` に bind されています。状態、evidence、admission、lifecycle の判断は Runtime の記録を正とします。

## 目標と段階順序

目標は、正確な英語・簡体字中国語・日本語の release reference と、公開 artifact、インストール、N-1 upgrade が検証された安定版 v1.0.1 を提供することです。これらは現時点では意図された benefit にすぎず、対応する evidence が pass するまで delivered と報告しません。

source stage の finish gate は、三言語ドキュメント、canonical promotion check、documentation acceptance、strict source quality route、8 件の `cognitive_benefit` integration test、および successor の正確な source 上での完全な `cargo test --locked --workspace -- --quiet` です。integration test では同じ private immutable `CARGO_BIN_EXE_ai-cockpit` copy を Python と Rust の両方に使い、完全な JSON/Markdown equality と binary digest/path assertion を維持します。workspace 検証は `RUST_TEST_THREADS=2`、`CARGO_BUILD_JOBS=1`、`CARGO_PROFILE_TEST_DEBUG=0`、`CARGO_INCREMENTAL=0`、Runtime worker 1、明示 timeout の上限 900 秒で実行します。Contract はこの ceiling を5つの対応 verification stage（`task`、`pre_ci`、`pr`、`merge`、`release`）に bind し、完全な workspace command 自体は task stage で実行します。timeout を省略した場合の既定値は 300 秒のままです。push 後は PR の正確な head が必須 CI と release-plan check を pass してから、PR を ready にするか通常 merge します。通常 merge と Runtime による新鮮な publication admission の後に v1.0.1 tag を一度作成し、その後に既存 workflow が候補を build/test します。四対象の candidate install/smoke と、stable v1.0.0 からの Linux x86_64 段階 upgrade は公開前に pass する必要があります。公式公開 artifact、公開 Linux install/N-1、Apple Silicon macOS CLI/MCP acceptance、finalization/close は後続 gate です。

## 範囲と境界

source scope は以下の6つの documentation file と既存の cognitive-benefit integration test です。

- `docs/reference/reference-parity.md`
- `docs/reference/reference-parity.zh-CN.md`
- `docs/reference/reference-parity.ja.md`
- `docs/work-items/WI-1070-v1-0-1-release-successor.md`
- `docs/work-items/WI-1070-v1-0-1-release-successor.zh-CN.md`
- `docs/work-items/WI-1070-v1-0-1-release-successor.ja.md`
- `crates/cockpit-cli/tests/cognitive_benefit.rs`

production source と CI policy は scope 外です。test 変更は A6 に示す `crates/cockpit-cli/tests/cognitive_benefit.rs` の private binary isolation に限ります。現在の Contract を新鮮な preflight に通し、明示的な人間の確認を得た後にだけ実施し、assertion は変更しません。WI-1068 の全履歴と WI-1069 の記録を保持します。tag、Release、immutable asset を上書きしません。stable v1.0.0 が N-1 predecessor であり、v1.0.1 tag と Release は作成前に未使用であることを確認します。

## Contract acceptance criteria

- **A1** WI-1068 の Contract、Summary、Outcome、events、archive、verification evidence、および過去の全成功/失敗結果を byte-for-byte で保持し、履歴を再分類せず successor evidence を追加する。
- **A2** canonical Runtime promotion projection を使用し、英語・簡体字中国語・日本語の reference-parity にある承認済み WI-1068 の3行だけを修正する。無関係な行は保持する。
- **A3** 本三言語 WI-1070 page と三つの parity 文書の WI-1070 行を追加し、最終 Contract に bind して WI-1069 の記録/worktree を保持する。
- **A4** 正確な successor base/head に対して documentation acceptance、canonical promotion `--check-all`、strict quality route を pass し、stale/pending parity を残さない。
- **A5** PR #1022 とその正確な merged source、stable v1.0.0 N-1、未使用の v1.0.1 tag/Release 名、provider immutability metadata、workflow gates を bind する新しい Runtime release plan を解決する。
- **A6** `RUST_TEST_THREADS=2` で同じ private immutable `CARGO_BIN_EXE_ai-cockpit` copy を Python と Rust の両方に使用し、完全な JSON/Markdown equality と `runtimeBinaryDigest`/path assertion を維持して、8 件すべての `cognitive_benefit` integration test を pass させます。続けて `RUST_TEST_THREADS=2`、`CARGO_BUILD_JOBS=1`、`CARGO_PROFILE_TEST_DEBUG=0`、`CARGO_INCREMENTAL=0`、Runtime worker 1、有限な明示 timeout 上限 900 秒で、`task`、`pre_ci`、`pr`、`merge`、`release` に bind された Work Item の `modify_source` verification policy の下、task stage で正確に受け入れた source 上の `cargo test --locked --workspace -- --quiet` を pass させます。timeout 省略時の既定値は 300 秒のままです。過去の失敗は履歴として保持します。
- **A7** PR を ready にするか通常 merge する前に、PR の正確な head で全必須 GitHub Actions、Rust Contract、repository-quality、package-coverage、Windows Runtime、behavioral-oracle gate を pass する。head が変われば exact-head CI を再実行し、以前の PR #1022 revision-binding failure も保持する。
- **A8** 通常 merge と新鮮な Runtime publication admission 後に、不変の v1.0.1 tag を一度だけ作成する。公開 Release 前に `aarch64-apple-darwin`、`x86_64-unknown-linux-gnu`、`aarch64-unknown-linux-gnu`、`x86_64-pc-windows-msvc` の candidate install/smoke と、v1.0.0 からの staged Linux x86_64 N-1 が pass する。
- **A9** 公開後、公式 manifest、SHA256SUMS、Linux x86_64 asset の identity/digest を検証し、公開 install と v1.0.0 からの N-1 upgrade を pass する。失敗した download は保持し、immutable asset は上書きしない。
- **A10** Apple Silicon macOS host で公式 macOS asset を install し、archive、manifest、checksum、installed SHA-256、version、absolute CLI path、MCP initialize、read-only request を検証する。local build で代替しない。
- **A11** すべての必須 release/platform acceptance が成功し、Runtime が cleanup を許可した後にだけ、正確な branch/worktree finalization と lifecycle receipt を記録して close する。WI-1068 の履歴を変更しない。
- **A12** 対応する documentation と publication/install/upgrade evidence が pass するまでは、多言語 documentation と検証済み stable v1.0.1 offering を意図された benefit のまま扱う。

candidate、public release、install、N-1、macOS/MCP、close はこのページで完了と主張しません。tag 作成、公開 release、user-visible delivery も主張しません。
