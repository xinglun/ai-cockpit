---
author: AI Cockpit maintainers
title: "WI-336 — 最初の 5 つの governance-documentation path"
workItemId: WI-336-reference-docs-foundation
description: "最初の 5 つの deferred reference governance document を比較し、source tooling を copy せず Rust-native boundary を記録する。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-336-reference-docs-foundation
---

# WI-336 — 最初の 5 つの governance-documentation path

## Intent と boundary

pinned reference commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の deferred path 5 件を
一つずつ比較します。目的は adopter 向けの audit 可能な Rust-native mapping であり、reference
Python、Make、provider、historical surface の byte copy ではありません。

## File-level comparison

| Pinned reference path | Classification | Rust/adopter counterpart と boundary |
| --- | --- | --- |
| `docs/reference/cross-wi-integration.md` | `reference-only` | Work Item ごとの archive validation、`reference-parity`、human Outcome が target の audit boundary です。source の WI-04..WI-13 aggregate report と UI receipt は Runtime command ではありません。 |
| `docs/reference/dependabot-intake.md` | `not-applicable` | Dependabot bot branch intake は provider 固有です。generic delegated evidence と明示的な Work Item source binding は external/provider が担当します。 |
| `docs/reference/deprecated-assets-registry.json` | `reference-only` | 明示的な Runtime lifecycle、immutable history、exact resource finalization が cleanup boundary です。source registry や Make scan は提供しません。 |
| `docs/reference/deprecated-assets.md` | `reference-only` | registry hygiene と obsolete command-chain は source documentation です。Rust は `check-deprecated-assets` を提供すると claim しません。 |
| `docs/reference/derived-artifacts.md` | `implemented-different-by-design` | Typed Contract/evidence/archive fact と status/Outcome projection を Runtime が分離し、Outcome/verification docs に記載します。derived view は後続 decision を authorize できません。 |

## Non-goals

この Work Item では cross-WI report engine、Dependabot integration、deprecated-asset deletion
command、derived-artifact registry、source Python/Make/V1 implementation を追加しません。
Immutable history と global Agent/MCP configuration も変更しません。

## Acceptance と verification

1. Pinned 5 path すべてに classification、counterpart、non-overclaiming reason を持つ inventory record が一つだけ存在する。
2. English、Simplified Chinese、日本語の comparison/parity ledger が classification と semantic/non-wire boundary で一致する。
3. Rust fact/view と external provider boundary を正しく説明し、unsupported source command を available として表示しない。
4. Inventory、documentation、parity、locked workspace verification が成功する。

[English](WI-336-reference-docs-foundation.md) ·
[简体中文](WI-336-reference-docs-foundation.zh-CN.md)
