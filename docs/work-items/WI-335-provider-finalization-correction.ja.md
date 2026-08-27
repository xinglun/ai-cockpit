---
author: AI Cockpit maintainers
title: "WI-335 — Provider finalization correction"
workItemId: WI-335-provider-finalization-correction
description: "実在する reviewed provider identity を verification 前に bind して WI-334 の bounded evidence-parity 文書を再配信する。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-335-provider-finalization-correction
---

# WI-335 — Provider finalization correction

WI-334 は immutable history として保持します。archive された Contract は、
実際の PR identity が確定する前に placeholder PR URL を記録していました。
この successor は predecessor を書き換えず、recovery linkage を記録し、
verification 前に実際の provider PR を bind した同じ bounded evidence-parity
文書を再配信します。

## Boundary

- WI-334 の archive、evidence、recovery bytes をすべて保持する。
- reviewed PR が存在してから WI-335 の provider context を記録する。
- installed Runtime lifecycle と hosted checks を再実行する。
- 正確な branch/worktree を finalize し、structured human decision で close し、正確な merge resource だけを削除する。

Cursor adopter feedback は external validation input のままです。stable stdout
JSON、replayable Outcome、lifecycle entry gate、verification invalidation は
既存 Runtime boundary であり、IDE chat への自動投稿は host Adapter の責任です。

## Acceptance

1. WI-334 predecessor の bytes と repository identity が変更されない。
2. 三言語 parity ledger が推測した PR URL なしで recovery を記録し、作成後に実際の provider PR を link する。
3. active Contract が verification 前に実際の PR を bind し、全 finalization receipt が installed Runtime と repository に一致する。
4. hosted checks と完全な lifecycle が監査可能な evidence を生成し、その後 exact branch/worktree を cleanup する。

[English](WI-335-provider-finalization-correction.md) · [简体中文](WI-335-provider-finalization-correction.zh-CN.md)
