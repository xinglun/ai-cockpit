---
author: AI Cockpit maintainers
title: "WI-763 — WI-752 P1 fact-reuse 再配信"
description: "最新 main から P1 fact reuse を raw 測定し、fail-closed な最適化判断を記録する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-763-wi752-fact-reuse-redelivery
lastVerifiedBy: WI-763-wi752-fact-reuse-redelivery
---

# WI-763 — WI-752 P1 fact-reuse 再配信

## Contract と境界

この Work Item は最新の remote `main`（`a8fa804057cad8792f56b580f665046d2d3fc52d`）
から開始する測定のみの再配信です。不変な WI-752 PR #733 は hosted quality の
`docs_governance_integrity: invalid_premerge_finalize` で拒否されました。旧 PR、branch、
worktree、evidence は履歴として保持し、書き換えも削除もしません。

範囲は raw 測定 evidence、この三言語レポート、三つの reference-parity 投影だけです。
Production Runtime の動作、IncrementalMerkle、release 公開、旧 WI-752 のリソースは範囲外です。

## 測定と判断

`tests/performance/runtime_benchmark.sh` で同一 Runtime の再測定を二回行いました。
identity probe 後の独立 CLI の最初の測定値を保持し、1 回の warmup 後に `inspect`、
`status`、`doctor`、`observe`、`work-item status`、`diagnose` を各 8 warm samples
測定しました。最初の値は真の cold cache とは主張しません。8 samples は信頼できる
p95/p99 の閾値に満たないため、これらの percentile は unavailable です。

raw fixture は `tests/performance/fixtures/WI-763-fact-reuse-measurement.json` です。
二回とも Runtime `0.2.87`、同じ Runtime digest、repository revision、machine、
unavailable の filesystem comparison key を使用しました。warm p50 の変化は約
`status +0.4%`、`observe +2.1%`、`inspect +2.2%`、`doctor -0.2%`、Work Item status
`+0.5%`、diagnose `+0.3%` でした。同一 Runtime の再測定だけでは候補最適化も信頼できる
利益も証明できないため、production optimization は採用せず candidate を decline します。

Runtime が公開しない実 read bytes、hash bytes、Git call 数、child process、peak memory、
cache invalidation event はゼロで埋めず unavailable と記録しました。resident MCP と
concurrent request は portable harness の対象外です。

## Correctness と有効性

`cargo test -p cockpit-repository --test repository_context -- --nocapture` は 7/7 passed。
one-snapshot memoization、repository isolation、明示的な RuntimeSession binding、source/config
変更後の fail-closed invalidation、phase boundary、validated observation context の governance
利用を検証しています。独立した `status` と `doctor` の反復出力も一致しました。

fresh verification evidence より前に parity 行を
`In progress → verified close 後 Implemented` として登録し、terminal paths を bind します。
close 後、Runtime promotion check が三言語ページと parity 行を terminal status に投影します。

## Terminal evidence

期待する binding は archive
`.ai/work-items/archive/WI-763-wi752-fact-reuse-redelivery.contract.json`、
verification `.ai/evidence/WI-763-wi752-fact-reuse-redelivery.verification.json`、
finalization `.ai/decisions/WI-763-wi752-fact-reuse-redelivery.finalize.json`、
close `.ai/decisions/WI-763-wi752-fact-reuse-redelivery.close.json` です。
