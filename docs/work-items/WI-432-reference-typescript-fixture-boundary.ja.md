---
author: AI Cockpit maintainers
title: "WI-432 — TypeScript web fixture の境界"
workItemId: WI-432-reference-typescript-fixture-boundary
description: "pinned TypeScript web fixture を一つずつ比較し、Node toolchain をコピーせず Rust-native の reference-only boundary を記録します。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-432-reference-typescript-fixture-boundary
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
---

# WI-432 — TypeScript web fixture の境界

## Intent と boundary

reference commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の
`examples/fixtures/typescript-web/` にある11ファイルを一つずつ確認します。
これは reference repository の TypeScript/npm executable sample であり、Rust
Runtime code、Node/TypeScript toolchain support、portable governance policy、
provider/enterprise evidence ではありません。

各 path は machine ledger で `reference-only` とし、[adaptation guide](../reference/typescript-fixture-adaptation.ja.md)
と[file comparison ledger](../reference/reference-file-comparison.ja.md)に Rust-native adopter boundary を記録します。
source fixture、npm dependency、installer、Node lifecycle script はコピーしません。

## Acceptance

- pinned 11 path をすべて読み、machine ledger に一度ずつ登録します。
- すべて non-empty reason/counterpart を持つ `reference-only` であり、batch に
  `deferred-next-batch`/`migrate-gap` を残しません。
- English、Simplified Chinese、Japanese の adaptation/comparison/index/parity route が source pin、file list、no-copy boundary で一致します。
- inventory/documentation gate を通し、Runtime governance semantics、adopter toolchain、global Agent/MCP configuration は変更しません。

## Verification と non-claims

これは semantic/reference-boundary parity であり、TypeScript toolchain support、source-command compatibility、
JSON-wire compatibility、second-stack adopter acceptance の主張ではありません。file-level truth は machine ledger にあります。

[English](WI-432-reference-typescript-fixture-boundary.md) · [简体中文](WI-432-reference-typescript-fixture-boundary.zh-CN.md)
