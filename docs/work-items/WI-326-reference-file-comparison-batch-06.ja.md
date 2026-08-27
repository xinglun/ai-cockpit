---
author: AI Cockpit maintainers
title: "WI-326 — reference file comparison batch 06"
workItemId: WI-326-reference-file-comparison-batch-06
description: "pinned reference の quality gate、overview、design philosophy、closure plan の 9 path を比較し、証拠付き Rust-native boundary を登録します。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-326-reference-file-comparison-batch-06
---

# WI-326 — reference file comparison batch 06

## Intent と境界

固定した reference commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の 9 path を一つずつ比較します。
object repository に必要な reader/governance semantics を保ち、source の Python、Make、
installer、fixture、internal progress implementation は copy しません。

共有 Rust Runtime は外部に一つだけ install し、すべての repository request は明示的な
`--repo` に bind します。この batch は documentation と conformance ledger に限定され、
Runtime behavior や source wire compatibility を追加主張しません。

## file ごとの比較

| pinned reference path | Classification | Rust/adopter の対応と境界 |
| --- | --- | --- |
| `docs/non-make-adaptation.ja.md` | `implemented-different-by-design` | Installation と Agent workflow route が external Runtime と repository-local adapter boundary を示します。Adopter-owned stack command は Core の外であり、source `Makefile.ai` bridge は copy/require しません。 |
| `docs/operations/quality-gates.ja.md` | `implemented-different-by-design` | Japanese CI quality-gate/manifest route が gate ownership、evidence、traceability、policy-selected `light`/`standard`/`strict` routing を保ちます。source Make/Python orchestration は copy しません。 |
| `docs/operations/quality-gates.md` | `implemented-different-by-design` | Versioned Rust-native gate manifest と CI route が quality-gate semantics を保ち、hosted CI と adopter stack check の owner boundary を分けます。 |
| `docs/operations/quality-gates.zh-CN.md` | `implemented-different-by-design` | Chinese quality-gate/manifest route は同じ evidence と dynamic-routing boundary を保ち、source Make/Python checker registry は target command ではありません。 |
| `docs/overview.ja.md` | `implemented-different-by-design` | Rust architecture、capabilities、Agent workflow、command route が source five-layer overview を request-scoped/repository-bound governance として保ちます。source status/verification registry は copy しません。 |
| `docs/philosophy/design-philosophy.ja.md` | `implemented-different-by-design` | Japanese product-boundary、capability、enterprise-governance docs が calibrated trust、evidence over self-declaration、proportional control、human responsibility を保ちます。 |
| `docs/philosophy/design-philosophy.md` | `implemented-different-by-design` | English product-boundary、capability、enterprise-governance docs が同じ原則を保ちます。Core は Agent Runtime、sandbox、identity provider、compliance certificate ではありません。 |
| `docs/philosophy/design-philosophy.zh-CN.md` | `implemented-different-by-design` | Chinese product-boundary、capability、enterprise-governance docs が同じ原則と明示的 non-goal を保ちます。 |
| `docs/plans/harden-work-item-pr-closure.md` | `reference-only` | Source は Python `ai-finish`/`ai-close` の internal historical hardening plan です。Current Rust lifecycle と governance-integrity route は closure intent を保ちますが、obsolete step/command name は current capability ではありません。 |

## Out of scope

Runtime command の追加、source Python/Make/YAML/installer file の copy、`Makefile.ai` の必須化、
global Agent/MCP 設定の変更、任意の host panel、controls scaffold、close-gap convenience は含みません。
Pinned source/target commit も変更しません。

## Acceptance と evidence

1. 9 つの pinned path をすべて読み、各 path に non-empty で証拠に基づく inventory record が一つだけあります。
2. Generated inventory はこの Work Item に `implemented-different-by-design` 8 件と
   `reference-only` 1 件を登録し、deferred/migrate gap を残しません。
3. English、簡体中文、日本語の comparison/parity page と本 Work Item は source pin、分類、境界で一致します。
4. Internal progress claim、source 固有 fixture、未実行の provider/enterprise assurance を current Runtime の事実として表示しません。
5. Installed Runtime inspect/status/doctor、docs/conformance、lifecycle closure、hosted CI、
   正確な cleanup が terminal evidence を提供します。Historical evidence は書き換えません。

[English](WI-326-reference-file-comparison-batch-06.md) ·
[中文](WI-326-reference-file-comparison-batch-06.zh-CN.md)
