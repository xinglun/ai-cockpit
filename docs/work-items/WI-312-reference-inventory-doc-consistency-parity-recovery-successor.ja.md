---
author: AI Cockpit maintainers
title: "WI-312 — reference inventory documentation parity recovery successor"
workItemId: WI-312-reference-inventory-doc-consistency-parity-recovery-successor
description: "WI-311 の immutable retry boundary 後に、manifest 派生 inventory count と archive 前の三言語 parity 登録を再配信します。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-312-reference-inventory-doc-consistency-parity-recovery-successor
---

# WI-312 — reference inventory documentation parity recovery successor

## Intent と boundary

この successor は最新の `origin/main` から bounded な inventory 文書修正を再配信します。
WI-311 は、installed Runtime が recovery 中の二重 completion event を拒否したため、
immutable な履歴として保持します。本 Work Item は inventory の分類も Runtime の動作も
変更しません。

## Scope と acceptance

三つの comparison page は 5,119 records の inventory から同じ marker を導出しなければ
なりません（generated history 4,262、implemented-different 182、equivalent 1、
not-applicable 3、reference-only 2、deferred 669、migrate gap なし）。deterministic
conformance test は stale、malformed、missing、または言語間で異なる marker を拒否します。
三つの parity page は verification evidence より前に本行を登録し、三つの Work Item 文書は
同じ bounded scope と `lastVerifiedBy` metadata を保持します。

検証は installed Runtime と repository の documentation/inventory gate を使用します。
source project は semantic reference であり、wire format や Runtime dependency ではありません。

