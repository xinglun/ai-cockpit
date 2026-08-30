---
author: AI Cockpit maintainers
title: "WI-414 — Python fixture の境界"
workItemId: WI-414-reference-python-fixture-boundary
description: "pinned Python fixture file を一つずつ比較し、source fixture をコピーしない reference-only boundary を記録します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-414-reference-python-fixture-boundary
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
canonical: docs/work-items/WI-414-reference-python-fixture-boundary.md
---

# WI-414 — Python fixture の境界

## Intent と boundary

Reference commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の
`examples/fixtures/python/` 4 ファイルを一つずつ読みます。これらは reference
repository の実行可能な Python/pytest sample であり、Rust Runtime code、Python toolchain
support、portable な governance policy、enterprise evidence ではありません。

| Pinned reference path | Classification | Target boundary の決定 |
| --- | --- | --- |
| `fixture.json` | `reference-only` | sample の stack、platform、path metadata。target の事実は repository-local とし、この file から推論しません。 |
| `pyproject.toml` | `reference-only` | sample の packaging と pytest configuration。Python install と test command は adopter/provider の責任です。 |
| `src/service.py` | `reference-only` | `ok` を返す application sample。governance logic ではなく、コピーしません。 |
| `tests/test_service.py` | `reference-only` | fixture 専用 pytest assertion。Runtime/enterprise evidence ではなく、adopter が自分の verification command を宣言します。 |

Python source、dependency manifest、installer、test runner は Rust repository にコピーしません。
installed shared Runtime は Python adopter にも同じ Contract、evidence、lifecycle、human
Outcome control を提供しますが、これは semantic/documentation parity であり、Python toolchain
や source-command compatibility ではありません。Second-stack adopter acceptance は別途認可し、
この WI では主張しません。

## Acceptance

- 4 pinned path をすべて読み、machine ledger に各一回だけ登録します。
- 4 path はすべて `reference-only` で、non-empty reason と target boundary を持ち、この batch
  に `deferred-next-batch` / `migrate-gap` を残しません。
- English、Simplified Chinese、日本語の comparison/parity route が source pin、file list、
  non-copy boundary で一致します。
- inventory regression と documentation gate が pass し、Runtime governance semantics、Python
  tooling、global Agent/MCP config は変更しません。

## Verification と non-claims

これは semantic/reference-boundary parity であり、Python toolchain support、source command
compatibility、JSON wire compatibility、second-stack adopter acceptance ではありません。
各 file の事実は machine ledger を正とします。

[English](WI-414-reference-python-fixture-boundary.md) · [简体中文](WI-414-reference-python-fixture-boundary.zh-CN.md)
