---
author: AI Cockpit maintainers
title: "WI-391 — C# adaptation example"
description: "Pinned C# adaptation example を比較し、installer と legacy wire format はコピーしません。"
workItemId: WI-391-reference-csharp-adaptation
audience:
  - adopter
  - contributor
  - maintainer
authority: canonical
status: in_progress
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
lastVerifiedBy: WI-391-reference-csharp-adaptation
---

# WI-391 — C# adaptation example

[English](WI-391-reference-csharp-adaptation.md) · [简体中文](WI-391-reference-csharp-adaptation.zh-CN.md)

## Intent と boundary

Pinned `examples/csharp/README.md` を section ごとに比較し、C#/.NET adopter に適用できる意味を Rust-native
に記録します。shared Runtime と repository-local responsibility を明確にし、source installer、Makefile、
guard YAML、Python orchestration、legacy JSON example を target requirement にしません。

## Scope

- 三言語 C# adaptation reference と reference index link を追加します。
- installation、quality gate、Contract、coverage、guideline evidence の mapping を三言語の比較/parity ledger に記録します。
- pinned source commit と semantic/non-wire boundary を明記します。

## Out of scope

.NET tool、C# fixture、second-technology adopter acceptance、installer 実装、Makefile、source guard parser、
Python check、provider integration、新しい Contract wire schema は追加しません。

## Acceptance criteria

1. Source の四つの section（installation、quality gates and guards、Contract、`guidelinesCompliance`）ごとに Rust-native mapping または明示的な external/non-applicable decision があること。
2. Source front matter を説明用 metadata として扱い、target の authority/capability claim にせず、installer variable/flag を異なる Rust install boundary に明示的に対応させること。
3. adaptation page が一つの immutable shared Runtime、明示的 `attach --repo`、repository-local `.ai/`、明示的 Agent adapter setup、adopter/provider-owned `dotnet` check を説明すること。
4. source `contractVersion: 2`、`ai*` verification name、`Makefile.ai.stack`、`guidelinesCompliance` は Rust JSON-wire requirement ではなく、現在の Contract/evidence/decision boundary を使うことを明記すること。
5. English、Simplified Chinese、日本語の page、index link、inventory、parity row が同期していること。
6. installed Runtime の fresh verification で documentation/conformance check が通ること。

## Evidence boundary

これは documentation/semantic parity です。C# adopter を実行した証明ではありません。将来の C# acceptance は
immutable public Release、独立した repository context、自身の evidence/decision chain を使用します。
