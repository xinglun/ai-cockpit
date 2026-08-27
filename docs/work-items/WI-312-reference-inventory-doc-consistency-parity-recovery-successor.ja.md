---
author: AI Cockpit maintainers
title: "WI-312 — reference inventory documentation parity recovery successor"
workItemId: WI-312-reference-inventory-doc-consistency-parity-recovery-successor
description: "WI-311 の immutable retry boundary 後に、manifest 派生 inventory count と archive 前の三言語 parity 登録を再配信します。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-312-reference-inventory-doc-consistency-parity-recovery-successor
---

# WI-312 — reference inventory documentation parity recovery successor

## Intent と boundary

この immutable delivery は retained provider finalization が後続の cleanup gate を満たせなかったため、
historical evidence として保持します。Contract、Summary、Outcome、Events、archive、verification、
finalization、close の bytes は書き換えません。WI-314 が同期済み default branch から bounded correction と
reconciliation boundary を再配信する明示的 successor です。

## Scope と acceptance

三つの comparison page は 5,119 records の inventory から同じ marker を導出しなければ
なりません（generated history 4,262、implemented-different 182、equivalent 1、
not-applicable 3、reference-only 2、deferred 669、migrate gap なし）。deterministic
conformance test は stale、malformed、missing、または言語間で異なる marker を拒否します。
三つの parity page は verification evidence より前に本行を登録し、三つの Work Item 文書は
同じ bounded scope と `lastVerifiedBy` metadata を保持します。

検証は installed Runtime と repository の documentation/inventory gate を使用します。
source project は semantic reference であり、wire format や Runtime dependency ではありません。
