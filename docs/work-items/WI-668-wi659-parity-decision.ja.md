---
author: AI Cockpit maintainers
title: "WI-668 — WI-659 parity decision link 修正"
description: "三つの reference-parity ledger に WI-659 の正確な versioned supersede decision を投影し、履歴記録を変更しない。"
workItemId: WI-668-wi659-parity-decision
audience: [maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-668-wi659-parity-decision
capabilityClaims: [governance_integrity, reference_parity]
---

# WI-668 — WI-659 parity decision link 修正

[English](WI-668-wi659-parity-decision.md) · [简体中文](WI-668-wi659-parity-decision.zh-CN.md)

## 意図と境界

閉じられた WI-659 predecessor には canonical successor recovery と digest で
versioned された supersede recovery の両方があります。三つの
reference-parity 投影は正確な terminal supersede decision を示さなければ、
governance gate が履歴チェーンを検証できません。本 Work Item は文書リンクだけを
変更し、WI-659 の archive、evidence、recovery、close 記録は immutable のままにします。

## 範囲

- 三つの reference-parity ledger の WI-659 行に、正確な versioned supersede
  decision path と superseded close path を追加する。
- English、簡体字中国語、日本語の投影を意味的に一致させる。
- 本 Work Item の三言語ドキュメント対応ページを追加する。

## 範囲外

WI-659 の archive、verification、recovery、close またはその他の生成 `.ai` 記録、
WI-664/WI-665 の文書、Runtime/Rust code、schema、test、release artifact、
global Agent/MCP configuration。

## 受入条件

- 各 WI-659 parity 行が archive、verification、canonical recovery を保持し、
  正確な versioned supersede recovery と superseded close decision path を含む。
- governance integrity gate が WI-659 の `missing_parity_decision` を報告しない。
- WI-659 の archive、evidence、recovery、close bytes が変更されない。

## 検証と終端記録

明示的な `--repo` を付けた installed Runtime、対象 parity/documentation checks、
`git diff --check`、repository hosted checks を使います。reviewed merge 後、Runtime
が宣言した archive、verification、finalization、close path を記録します。
