---
author: AI Cockpit maintainers
title: Input trust data flow
description: Repository content、tool output、generated interpretation の provenance-aware な扱い。
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: translation
canonical: docs/reference/input-trust-dataflow.md
lastVerifiedBy: WI-347-reference-knowledge-trust-lifecycle-assessment
capabilityClaims:
  - provenance_aware_observation
---

# Input trust data flow

[English](input-trust-dataflow.md) · [简体中文](input-trust-dataflow.zh-CN.md) · [日本語](input-trust-dataflow.ja.md)

AI Cockpit は repository content と tool output を authority ではなく、分類すべき input として扱います。Markdown の command らしい行、Issue の role claim、Agent が生成した結論は、観測されたという理由だけで permission や独立した evidence にはなりません。

## Rust-native provenance

Runtime は typed `FactOrigin`、`TraceableFact`、`TraceableDerivation` で限定された provenance を表します。主な origin は `Observed`、`Declared`、`Derived`、`External`、`Unknown` です。Snapshot fact、build detection、test output、repository document は repository と operation に追跡でき、derived signal は input reference と rule を保持します。

これは semantic parity であり、source JSON wire compatibility ではありません。Reference の Python trust module や provider authentication はコピーしません。

## 安全なルール

- Direct user instruction と repository policy は限定された operation の authority になり得ます。repository document、Issue、PR、web page、fixture、log は content または untrusted observation です。
- Tool output は data です。Agent の解釈は新しい独立 verification result ではありません。
- Cross-step では元の origin を保ち、derivation を追加します。後続 step が unknown や untrusted source を消すことはできません。
- Provenance の欠落、identity の矛盾、安全でない injection、高リスク境界の unknown/generated conclusion は local action を停止し、安全な代替または human review を示します。

Trust layer は人の認証、provider の検証、外部 merge/release の認可を行いません。これらは human decision、provider、enterprise evidence の外部境界です。

## Adopter repository

Attach した Runtime は同じ fail-closed ルールを継承しますが、fact と evidence は repository ごとに隔離されます。すべての call は明示的な `--repo` を要求し、global current project や共有 provenance state はありません。
