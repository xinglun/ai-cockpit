---
author: AI Cockpit maintainers
title: "WI-421 — mixed-monorepo fixture boundary"
description: "業務コードや toolchain asset をコピーせず、固定 mixed Python/Node fixture を file-by-file 比較します。"
workItemId: WI-421-reference-mixed-monorepo
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-421-reference-mixed-monorepo
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
---

# WI-421 — mixed-monorepo fixture boundary

[English](WI-421-reference-mixed-monorepo.md) · [简体中文](WI-421-reference-mixed-monorepo.zh-CN.md)

## Intent と boundary

`examples/fixtures/mixed-monorepo/` の pinned file を一つずつ読み、attached Rust repository
へ portable な責務かを記録します。Python/Node application、package metadata、source command、
installer behavior は Runtime に持ち込まず、adopter が継承できる fact、scope、provider-owned
execution、evidence binding の境界だけを残します。

## Scope

対象は `fixture.json`、`package.json`、`pyproject.toml`、`services/api/app.py`、
`services/api/tests/test_app.py` の 5 file です。inventory、三言語 comparison/parity route、
reference index、adaptation page を同時に更新します。

## Acceptance

- 5 pinned path をすべて読み、inventory に一度ずつ `reference-only` として登録し、理由と Rust/adopter counterpart を記録します。
- fixture source、Python/Node dependency、installer、provider-global configuration、source JSON wire はコピーしません。
- English/中文/日本語 route が source pin、file list、inheritance boundary、non-claim で一致します。
- inventory、documentation、object/adopter inheritance check が pass します。

## Verification boundary

これは semantic/documentation parity であり、Python/Node toolchain support、source-command
compatibility、second-technology adopter acceptance ではありません。検証は明示的 `--repo` を付けた
installed shared Runtime で行い、adopter の interpreter、dependency、command、provider evidence は
この Work Item の外側です。
