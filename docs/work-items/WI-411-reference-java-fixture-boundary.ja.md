---
author: AI Cockpit maintainers
title: "WI-411 — Java マルチモジュール fixture の境界"
workItemId: WI-411-reference-java-fixture-boundary
description: "pinned Java fixture file を一つずつ比較し、source fixture をコピーしない reference-only boundary を記録します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-411-reference-java-fixture-boundary
canonical: docs/work-items/WI-411-reference-java-fixture-boundary.md
---

# WI-411 — Java マルチモジュール fixture の境界

## Intent と boundary

Reference commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の
`examples/fixtures/java-multimodule/` 9 ファイルを一つずつ読みます。これらは
reference repository の実行可能な Java/Maven sample であり、Rust Runtime code、
portable な governance policy、enterprise evidence ではありません。

| Pinned reference path | Classification | Target boundary の決定 |
| --- | --- | --- |
| `.gitignore` | `reference-only` | fixture build hygiene のみ。target release harness が隔離 temp root を管理します。 |
| `app/src/main/java/fixture/app/Main.java` | `reference-only` | Java application sample。generic argv 実行は Java-specific Runtime support を意味しません。 |
| `app/src/test/java/fixture/app/MainTest.java` | `reference-only` | fixture assertion。adopter verification は宣言された command を記録し、この test をコピーしません。 |
| `core/src/main/java/fixture/core/Decision.java` | `reference-only` | domain sample policy。target repository policy は明示的な typed data のままです。 |
| `core/src/test/java/fixture/core/DecisionTest.java` | `reference-only` | sample 専用 test であり、Runtime/enterprise evidence ではありません。 |
| `evidence.json` | `reference-only` | unavailable capability を含む source-local evidence。target release evidence には昇格しません。 |
| `fixture.json` | `reference-only` | source stack/module metadata。target は adopter capability を推論しません。 |
| `pom.xml` | `reference-only` | Maven build input。Java/Maven 実行は adopter または delegated provider の責任です。 |
| `scripts/lifecycle.sh` | `reference-only` | source fixture orchestration。target lifecycle は installed Rust Runtime が提供します。 |

Java source、Maven manifest、source shell orchestrator は target に追加しません。
Second-technology adopter acceptance は明示的に承認された別 Work Item とし、この
batch では主張しません。

## Acceptance

- 9 pinned path をすべて読み、machine ledger に各一回だけ登録します。
- 9 path はすべて `reference-only` で、non-empty reason と target boundary を持ち、
  この batch に `deferred-next-batch` / `migrate-gap` を残しません。
- English、Simplified Chinese、日本語の comparison/parity route が source pin、9 path、
  non-copy boundary で一致します。
- inventory regression と documentation gate が pass し、Runtime governance semantics と
  global Agent/MCP config は変更しません。

## Verification と non-claims

これは semantic/reference-boundary parity であり、Java toolchain support、source command
compatibility、JSON wire compatibility、second-stack adopter acceptance ではありません。
各 file の事実は machine ledger を正とします。

[English](WI-411-reference-java-fixture-boundary.md) · [简体中文](WI-411-reference-java-fixture-boundary.zh-CN.md)
