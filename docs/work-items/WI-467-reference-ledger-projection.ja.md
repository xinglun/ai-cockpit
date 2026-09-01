---
author: AI Cockpit maintainers
title: "WI-467 — reference ledger projection consistency"
workItemId: WI-467-reference-ledger-projection
description: "Current reference-ledger snapshot と三言語ドキュメントを一つの検証済み source に束ねる。"
audience: [maintainer, reviewer]
status: in_progress
authority: authorized
lastVerifiedBy: WI-467-reference-ledger-projection
---

# WI-467 — reference ledger projection consistency

[English](WI-467-reference-ledger-projection.md) · [简体中文](WI-467-reference-ledger-projection.zh-CN.md)

## Intent

Reference-file comparison snapshot の prose count が machine ledger から分岐した問題を修正する。
Historical snapshot と retired path は保持し、marker だけを更新する変更を拒否する regression gate を追加する。

## Scope

- `tests/conformance/reference_file_inventory.json` から current の三言語 snapshot table を導出する。
  Current count は retired path を除外し、append-only total は別に保持する。
- English、簡体字中国語、日本語の current snapshot に同じ canonical count を表示する。
- `reference_inventory_docs_test.py` と shell wrapper を拡張し、既存 marker と human-readable table の両方を検証する。

## Out of scope

Reference inventory bytes、source lock、historical narrative、Runtime/object repository、workflow architecture、
release script、global Agent/MCP configuration。

## Acceptance

1. Current table は machine count と一致する：4,450 current path、3,681 generated-history、252 implemented-different-by-design、
   1 implemented-equivalent、4 not-applicable、62 reference-only、450 deferred-next-batch、0 migrate-gap、
   669 retired path、5,119 append-only record。
2. Machine marker が同じでも table を意図的に変更した場合は fail する。
3. Historical section と retired-path record は declared current snapshot の変更以外では保持する。
4. 三言語 page の reader route と semantic/non-wire boundary を保持する。

## Verification

- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/conformance/reference_file_inventory_test.sh`
- Contract で宣言した repository quality/documentation gate

## Boundary

Ledger は current count の authority である。Historical narrative は immutable audit record であり、後続 snapshot に合わせて静かに書き換えない。
