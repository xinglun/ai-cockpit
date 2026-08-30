---
author: AI Cockpit maintainers
title: "WI-428 — Recovery-chain finalization"
description: 残る recovery 境界を収束し、競合する successor を防止する。
audience: [contributor, maintainer]
status: in-progress
authority: governed
workItemId: WI-428-recovery-chain-finalization
predecessorWorkItemId: WI-426-recovery-successor-binding
lastVerifiedBy: WI-428-recovery-chain-finalization
---

# WI-428 — Recovery-chain finalization

## Intent と境界

この Work Item は、直接バインドされた successor により WI-426 の残存する
recovery 境界を解消し、recovery の選択を決定的にします。前置 Work Item の
Contract、Summary、Outcome、Events、evidence、recovery receipt bytes はすべて
不変のまま保持します。

対象範囲：

- 異なる Work Item を指す二つ目の `successor` decision を拒否すること。
- append-only の retry/supersede decision と安定した fail-closed エラーを保持すること。
- 三つの reference-parity ledger を実際の終端 receipt に更新すること。
- one-successor lineage ルールを文書化し、Rust テストで検証すること。

対象外：release artifact、無関係な Work Item、global Agent/MCP 設定、Runtime
architecture の分割。

## Acceptance と evidence

前置 Work Item に曖昧な競合 successor chain を蓄積させません。WI-426 は直接バインド
されたレビュー済み successor、WI-424 は不変の supersede receipt で表現し、三つの
parity ledger は実際の receipt を指す必要があります。履歴 bytes は書き換えません。
競合 successor request は
`recovery_decision_invalid:competing_successor` で fail closed します。

Verification evidence、archive manifest、finalization、close、および merge 済み PR は
`.ai/evidence/` と `.ai/decisions/` に記録します。

[English](WI-428-recovery-chain-finalization.md) · [中文](WI-428-recovery-chain-finalization.zh-CN.md)
