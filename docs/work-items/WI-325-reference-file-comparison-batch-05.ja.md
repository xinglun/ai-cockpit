---
author: AI Cockpit maintainers
title: "WI-325 — reference file comparison batch 05"
workItemId: WI-325-reference-file-comparison-batch-05
description: "pinned reference の次の 9 文書を一つずつ比較し、Rust-native な意味境界を登録します。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-325-reference-file-comparison-batch-05
terminalArchive: .ai/work-items/archive/WI-325-reference-file-comparison-batch-05.contract.json
terminalVerification: .ai/evidence/WI-325-reference-file-comparison-batch-05.verification.json
terminalFinalization: .ai/decisions/WI-325-reference-file-comparison-batch-05.finalize.json
terminalDecision: .ai/decisions/WI-325-reference-file-comparison-batch-05.close.json
---

# WI-325 — reference file comparison batch 05

## Intent と境界

固定した reference commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の 9 path を一つずつ比較します。
reference の Python、Make、fixture、内部進捗の実装をコピーせず、証拠で確認できる意味だけを
Rust Runtime と adopter に引き継ぎます。

共有 Rust Runtime は外部に一つだけインストールし、すべての repository を明示的な
`--repo` に bind します。Cursor adopter feedback は外部観測として扱いました。安定した
Outcome と entry gate は既存 Runtime でカバーされ、任意の host UI 便利機能はこの batch の
実装済み事実として扱いません。

## file ごとの比較

| pinned reference path | 分類 | Rust/adopter の対応と境界 |
| --- | --- | --- |
| `docs/features/task-outcome-report-self-check.md` | `reference-only` | 現行の Outcome/report/event ページと `.ai/README.md` が対応。WI22 の内部進捗と古い release claim はコピーしません。 |
| `docs/fixtures/real-fixture-evidence.ja.md` | `implemented-different-by-design` | 日本語 fixture layout、Release adopter/upgrade acceptance、distribution、adversarial-validation。local/provider/enterprise evidence は分離します。 |
| `docs/fixtures/real-fixture-evidence.md` | `implemented-different-by-design` | Rust fixture と immutable Release adopter/upgrade harness。source の 7 stack `make`/Python matrix は Runtime capability ではありません。 |
| `docs/guides/lightweight-verification.ja.md` | `implemented-different-by-design` | 日本語 verification route/semantics、CI quality、cost pages。warning は authorize せず、critical failure は停止します。 |
| `docs/guides/lightweight-verification.md` | `implemented-different-by-design` | Rust の stage-aware verification と動的 light/standard/strict route。source checker script はコピーしません。 |
| `docs/guides/lightweight-verification.zh-CN.md` | `implemented-different-by-design` | 中国語の verification route/semantics、CI quality、cost pages と同じ fail-closed 境界。 |
| `docs/installation.md` | `implemented-different-by-design` | reader-first installation、Release distribution/security、`.ai/README.md`。install は repository attach や calibration 完了を意味しません。 |
| `docs/maintainers/adding-or-classifying-a-check.md` | `implemented-different-by-design` | versioned gate manifest、dynamic route、runner、regression checks。profile、依存、skip、hard failure は明示的です。 |
| `docs/maintainers/task-outcome-events.md` | `implemented-different-by-design` | typed Rust Task Outcome events、append-only correction、privacy validation、archive binding、human handoff。 |

## Out of scope

Runtime command の追加、source Python/Make/YAML/fixture のコピー、`Makefile.ai` の必須化、
Cursor/global Agent/MCP 設定の変更、任意の `close-gap`、controls template 自動生成、host
panel 展開は含みません。これらは別の product decision であり、parity に隠しません。

## Acceptance と evidence

1. 9 つの pinned path をすべて読み、各 path に non-empty で証拠に基づく inventory record が一つだけあります。
2. 生成 inventory は WI-325 に `implemented-different-by-design` 8 件と `reference-only` 1 件を登録し、deferred/migrate gap を残しません。
3. English、簡体中文、日本語の parity page と本 Work Item は source pin、分類、意味境界で一致します。
4. 内部進捗、source 固有 fixture 結果、未実行の provider/enterprise assurance を現行 Runtime の事実として表示しません。
5. installed Runtime、docs/conformance、hosted CI、lifecycle closure、正確な branch/worktree cleanup が terminal evidence を提供します。historical evidence は書き換えません。

[English](WI-325-reference-file-comparison-batch-05.md) ·
[中文](WI-325-reference-file-comparison-batch-05.zh-CN.md)
