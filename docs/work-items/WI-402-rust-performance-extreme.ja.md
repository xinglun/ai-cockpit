---
author: AI Cockpit maintainers
title: WI-402 — Rust Runtime performance extreme
description: Governance truth を弱めずに Rust Runtime の不要なコストを測定・削減する。
workItemId: WI-402-rust-performance-extreme
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-402-rust-performance-extreme
---

# WI-402 — Rust Runtime performance extreme

この Work Item は Cockpit repository と接続された adopter repository の両方で
共有 Rust Runtime を最適化します。これは測定に基づく最適化であり、verification
strength、evidence identity、fail-closed、request-scoped repository context、
deterministic な human Outcome の意味を変更しません。

## Delivered boundary

- Exact verification reuse は shell/mise/Agent session bookkeeping を除外しますが、
  `PATH`、`PWD`、`TMPDIR`、`CARGO_HOME`、`RUSTFLAGS` など command/toolchain input は保持します。
- Runtime が生成する `.ai/` receipt を source content identity から除外します。
  そのため receipt 自身で reuse を stale にせず、source または非 `.ai` の変更は引き続き無効化します。
- Reuse は profile-authorized かつ全 identity 一致の場合だけです。explicit custom command は fresh のままで、
  mismatch なら宣言された check を実行します。
- session metadata、source content identity、first-run/second-run の exact reuse を回帰テストで確認します。

## Object-project inheritance

最適化は共有外部 binary に入り、adopter にコピーしません。各 repository は独立した `.ai/`
evidence を保持し、公開 Runtime に upgrade した後に同じ規則を継承します。各 verification
context には Runtime version/digest と repository identity が残ります。

## Verification

Work Item evidence は targeted Rust test、workspace quality、release/adopter acceptance を記録します。
Timing は advisory evidence であり、必須の Verification Tier や Evidence Assurance を下げません。

### ローカル測定（advisory）

2026-08-29、macOS arm64 で同じ小規模な attached repository を 10 回測定し、
インストール済み v0.2.40 binary と candidate release profile を比較しました。
warm 呼び出しの P95 は `inspect` 72.561 ms → 72.217 ms（-0.5%）、`status`
95.573 ms → 94.500 ms（-1.1%）、`doctor` 16.636 ms → 13.828 ms（-16.9%）、
`observe` 73.057 ms → 71.957 ms（-1.5%）でした。これはローカルの process
latency 観測であり、provider や enterprise の保証ではありません。candidate
は repository 外で測定しており、公開 release acceptance artifact ではありません。
