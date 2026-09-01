---
author: AI Cockpit maintainers
title: "WI-468 — reference ledger parity promotion"
description: "不変の WI-467 を引き継ぎ、三言語 parity 台帳登録を補って manifest 派生の current snapshot を再配信します。"
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-468-reference-ledger-parity-promotion
predecessorWorkItemId: WI-467-reference-ledger-projection
status: in_progress
authority: authorized
lastVerifiedBy: WI-468-reference-ledger-parity-promotion
---

# WI-468 — reference ledger parity promotion

## Intent と境界

WI-468 は不変の WI-467 に対する明示的な successor です。predecessor の
Contract、evidence、Outcome、archive、recovery receipt は変更しません。
同じ bounded manifest-derived current snapshot を再配信し、English・中文・
日本語の reference-parity ledger に登録します。これにより merge 前に
repository governance gate が documentation truth を検証できます。

## Scope と acceptance

- 三つの comparison page は canonical inventory manifest から派生させます。
- 三つの reference-parity page に同一の WI-468 行を追加します。
- historical section と predecessor bytes は immutable のままにします。
- current count または parity 登録が分岐した場合、documentation gate は
  fail closed します。

参照 source checkout は local semantic reference であり、Runtime や wire
format の依存ではありません。生成された archive、evidence、decision は
Runtime が管理し、手編集しません。

## Verification

installed Runtime を使い、すべての repository-bound command に明示的な
repository path を指定します。Contract が宣言する documentation、
conformance、workspace gate を実行します。

## Links

[English](WI-468-reference-ledger-parity-promotion.md) ·
[简体中文](WI-468-reference-ledger-parity-promotion.zh-CN.md)
