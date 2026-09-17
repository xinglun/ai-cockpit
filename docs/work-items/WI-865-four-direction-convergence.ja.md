---
author: AI Cockpit maintainers
workItemId: WI-865-four-direction-convergence
title: "四方向 convergence の受入れ"
description: "Performance、Outcome、CHI、architecture convergence の evidence-based acceptance report。"
audience:
  - adopter
  - contributor
  - maintainer
status: in_progress
authority: canonical
lastVerifiedBy: WI-865-four-direction-convergence
---

# WI-865 — 四方向 convergence の受入れ

## Scope と判断境界

この report は WI-865 Contract の四つの bounded work package を記録する。merge や release の権限は与えない。review 済み merge 後の公開は、明示的に認可された別 Work Item が担当する。

## Acceptance evidence

- **Performance (P):** release-grade statistic と P0 comparator は 99 個の valid warm sample を拒否し、100 個を受け入れ、raw sample を保持し、取得できない counter を理由として示す。collector は Runtime inspect/status/Outcome/verification planning と diagnose を対象にし、paired JSON は収集後にリンクする。
- **Outcome (O):** lifecycle、CLI、MCP、summary、full は Runtime-bound observation assembly を共有する。release fact は optional で、明示的な evidence binding が必要。changed path は audit detail に残す。
- **CHI (C):** 三言語の first Work Item route は `start --prepare` から始まり、reviewable、mergeable、closed を区別し、provider/release detail は詳細 workflow reference に置く。
- **Architecture (A):** observation、lifecycle、verification、projection、adapter の所有権を responsibility map に記載し、Outcome/lifecycle の focused test で保護する。

## Current status

下の table は Runtime receipt、hosted PR state、paired benchmark JSON の capture だけで埋める。unknown や unavailable は保持し、green check から benefit や authorization を推測しない。

| Area | Baseline → candidate | Direct evidence | Decision |
|---|---|---|---|
| Runtime と cycle cost | paired capture 待ち | `.ai/evidence/WI-865-four-direction-convergence/` | pending |
| Outcome consistency | lifecycle と projection test | `.ai/evidence/WI-865-four-direction-convergence.verification.json` | pending |
| CHI/default path | 三言語 semantic/documentation gate | `tests/docs/getting_started_semantic.sh` | pending |
| Architecture | responsibility map と pure-renderer test | `docs/reference/architecture-responsibility-map-2026-09.md` | pending |

## Remaining risks

Resident MCP と provider-side publication はこの Work Item の external boundary である。後続 release Outcome は immutable record と独自の evidence reference がある場合だけ、version、release link、install/upgrade acceptance、cleanup を記録する。
