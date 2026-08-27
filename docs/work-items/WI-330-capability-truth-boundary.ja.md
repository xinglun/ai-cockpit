---
author: AI Cockpit maintainers
title: "WI-330 — capability-truth boundary の決定"
workItemId: WI-330-capability-truth-boundary
description: "V1 asset を copy せず、reference の capability claim、freshness、truth-matrix 文書を file 単位で比較して閉じる。"
audience:
  - adopter
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-330-capability-truth-boundary
capabilityClaims:
  - reference_parity
---

# WI-330 — capability-truth boundary の決定

## Intent と boundary

この Work Item は pinned source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の
4 file を一つずつ再読し、semantic comparison を閉じます。Target の Rust Runtime は
repository governance layer のままで、source Python checker、matrix bytes、V1 runtime
state は copy しません。

## File-level decision

| Pinned source path | Classification | Target responsibility |
| --- | --- | --- |
| `docs/reference/capability-claim-authoring.md` | `reference-only` | Target の文書 metadata は説明用です。`capability show` と capability registry は observed な bind 済み fact を報告し、lexical trigger で public wording を authorize しません。 |
| `docs/reference/capability-evidence-freshness.md` | `reference-only` | Work Item verification receipt には identity/freshness check がありますが、source Capability Truth row expiry と portable-environment policy は current Runtime feature ではありません。 |
| `docs/reference/capability-truth-matrix.json` | `reference-only` | Source 30-row matrix は Rust wire format や authorization source ではありません。Target truth は request-scoped、repository/snapshot-bound projection で adopter と external exclusion を明示します。 |
| `docs/reference/capability-truth-matrix.md` | `reference-only` | Target capability/adoption page は observed fact、repository evidence、adopter installation、delegated provider evidence、enterprise boundary を説明し、source matrix/checker を宣伝しません。 |

これは明示した product boundary であり、未追跡 omission ではありません。将来 claim binding や
row freshness を追加する場合は、別の human-owned Work Item で Rust-native schema、evidence
generation、stale handling、multilingual scope、adopter acceptance を定義します。

## Acceptance

1. Inventory と tri-language comparison page が 4 path の classification、counterpart、reason を個別に記録します。
2. 三言語の comparison と parity page が同じ non-copy / non-authorization boundary を示します。既存の capability index はこの文書 scope の外側です。
3. Source Python script、source matrix JSON、V1 state、global Agent/MCP configuration、unsupported claim は追加しません。
4. Inventory/documentation gate、Runtime verification、reviewed PR、merge、finalization、close、exact cleanup が pass します。

[English](WI-330-capability-truth-boundary.md) · [简体中文](WI-330-capability-truth-boundary.zh-CN.md)
