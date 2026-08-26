---
author: AI Cockpit maintainers
title: "WI-291 — CI Contract-aware quality gate"
workItemId: WI-291-ci-contract-aware-gates
description: "hosted stale-parity rejection 後の immutable failed delivery を保持し、WI-292 と WI-293 が recovery history を継承する。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-291-ci-contract-aware-gates
authority: canonical
---

# WI-291 — CI Contract-aware quality gate

## 目的

WI-291 は bounded Rust Contract-aware CI gate を実装しましたが、parity が
verification 後に登録されたため hosted quality が documentation projection
を拒否しました。lifecycle bytes は immutable に保持し、WI-292 と WI-293 が明示的な
successor として同じ実装を再配信します。

## Boundary

- WI-291 の archive、verification、blocked finalization、recovery をそのまま保持する。
- failed PR を merge/release 済みと扱わない。
- 最新の remote default base からの再配信は successor chain（WI-292、WI-293）が行う。

## adopter との一致

この repository と fresh adopter は、同じ installed Runtime、explicit repository
context、fail-closed lifecycle、human-visible Outcome で統治します。

## 検証

Hosted PR の結果は failed delivery evidence として保持し、新しい verification
と provider lifecycle は WI-293 が担当します。
