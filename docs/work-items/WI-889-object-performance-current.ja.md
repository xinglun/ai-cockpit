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
