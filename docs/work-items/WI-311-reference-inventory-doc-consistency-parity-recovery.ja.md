---
author: AI Cockpit maintainers
title: "WI-311 — reference inventory documentation parity recovery"
workItemId: WI-311-reference-inventory-doc-consistency-parity-recovery
description: "manifest 派生の inventory count と、archive 前の三言語 parity 登録を再配信します。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
---

# WI-311 — reference inventory documentation parity recovery

## Intent と boundary

WI-310 は archive 後に必要な tri-language parity 登録を欠いたまま保存されました。
この successor は最新 `origin/main` から同じ bounded な inventory 文書修正を再配信し、
verification evidence より前に parity row を登録します。前身の bytes は immutable に保持し、
source wire compatibility は主張しません。

## Scope

- 3 言語の reference-file comparison ledger を pinned manifest の count（合計 5,119、
  generated-history 4,262、implemented-different-by-design 182、implemented-equivalent 1、
  not-applicable 3、reference-only 2、deferred-next-batch 669、migrate-gap 0）へ同期します。
- manifest から count を導出し、3 言語の machine-readable marker を検証する deterministic
  regression を追加します。
- evidence 生成前に English/中文/日本語の reference-parity ledger へ Work Item を登録します。
- 3 言語の Work Item 文書を同期します。

## Out of scope

Rust Runtime behavior、reference classification、source implementation copy、release/adopter/CI
workflow、global Agent/MCP configuration、WI-310 または historical evidence の書き換えは対象外です。

## Acceptance と verification

comparison docs、parity rows、Work Item docs は documentation と governance-integrity check を通過し、
stale/malformed/missing/言語間不一致の marker は fail します。installed Runtime を明示的な `--repo`
で使い、reviewed lifecycle を完了し、最終 human Outcome は中国語で表示します。
