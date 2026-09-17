---
author: AI Cockpit maintainers
workItemId: WI-889-object-performance-current
title: 現行 object repository の performance evidence
description: 選定した object repository で現行 Runtime と開発サイクルを paired measurement する。
audience: [adopter, contributor, maintainer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-889-object-performance-current
---

# WI-889 — 現行 object repository の performance evidence

この Work Item は、同じ machine、toolchain、build mode、scenario 定義で
現行 Runtime と公開 baseline を paired measurement する。選定した object
repository は一時的な隔離 view からのみ観測し、main branch と既存の
working-tree state は範囲外にする。

## Acceptance boundary

- release-grade percentile comparison は 100 個以上の valid warm sample を
  必須とし、99 個は diagnostic-only として gate が拒否する。
- Runtime latency と開発サイクル cost は分けて、raw sample、p50/p95/p99、
  environment identity、operation/reuse counter、invalidation または
  unavailable reason を報告する。
- goods-garden、sentinel、ai-investigation-orchestrator は一時的な隔離
  view だけで測定し、収集後に clean up する。
- archive 前に三言語の Work Item projection と reference-parity row を
  同期する。この Work Item は release 公開を含まない。

## Verification plan

既存の performance harness を `CARGO_INCREMENTAL=0` と共有 verification
target directory で実行する。高価な workspace verification の前に format、
performance、documentation projection の focused check を行う。raw
evidence を保持し、取得できない metric は zero ではなく unknown と記録する。

## 現在の paired result

baseline（`0.2.93`）と candidate（`0.2.95`）を、同じ
`aarch64-apple-darwin`、Rust/Cargo `1.98.1`、同じ隔離 object repository view
で七つの scenario に対して paired 測定した。各 operation は 100 個の
valid warm sample を持つ。比較器は p50、p95、p99 を出力し、5 ms noise
判定は従来どおり安定した p50/p95 の予算で行い、p99 は tail diagnostic
として保持する。38 比較はすべて `within_noise` であり、有効な比較だが
改善を証明するものではない。

測定 view は goods-garden（current repository と file-change scenario）、
sentinel（many-files-clean）、ai-investigation-orchestrator
（many-historical-wi）である。small-clean の直接測定も試したが、提供された
object repository はすべて harness の `<=100` tracked-file 条件を超えたため、
synthetic repository で補わず unavailable とした。process/resource counter と
invalidation reason を含む collector の完全出力は、[圧縮 raw capture archive](../../.ai/evidence/WI-889-object-performance-current/raw/runtime-captures.tar.gz)
と [SHA-256](../../.ai/evidence/WI-889-object-performance-current/raw/SHA256SUMS)
に保持する。

diagnostics overhead は別の paired measurement とした。current repository では
diagnostics-on は off に対して `verification-plan` で p50 5.134 ms、p95 5.255 ms、
`status` で p50 0 ms、p95 1.183 ms、`work-item-outcome` で p50 0.674 ms、p95
1.689 ms 増加した。

Runtime latency と development-cycle cost は分離して報告する。Contract→reviewable
PR、verification→finish、post-merge cleanup は lifecycle timestamp が得られるまで
主張せず、未取得の stage は development-cycle report で明示的に unavailable とする。
