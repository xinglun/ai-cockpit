---
author: AI Cockpit maintainers
title: "WI-319 — close decision と promotion の互換性"
workItemId: WI-319-close-decision-and-promotion-compatibility
description: "静的な promotion と governance consumer を、installed Runtime の close/finalization binding と整合させる。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-319-close-decision-and-promotion-compatibility
---

# WI-319 — close decision と promotion の互換性

## Intent と boundary

installed Runtime は明示的な positive close decision（`approved` と
`confirmed`）を受け付け、`close` 前に deleted sequence-1 finalization transition を
追加する場合があります。静的ドキュメントと governance consumer は現在の記録を
理解しつつ、従来の close 後 reconciliation path も保持しなければなりません。
この Work Item は consumer と三言語 documentation だけを対象とし、不変の Runtime
record は書き換えません。

## Scope と acceptance

- promotion、status、governance check は current sequence-1 cleanup-before-close と
  historical root-bound reconciliation の両方を受け付け、predecessor、identity、path、
  digest の不一致は引き続き拒否します。
- `approved` と明示的な `confirmed` structured close decision は positive、`rejected`
  は Work Item を Implemented に昇格させません。
- W317 の close projection を三言語の Work Item document と parity ledger に反映し、
  immutable archive、verification、finalization、close bytes は変更しません。
- regression fixture は両方の finalization path と `confirmed` decision token を対象とし、
  documentation acceptance と governance gate の厳格さを維持します。
- installed Runtime の lifecycle に従い、hosted checks 成功後にのみ finalize、review、merge、
  close、exact cleanup を行います。

## Verification

installed Runtime に明示的な repository context を渡し、promotion、status-consistency、
governance-integrity、documentation regression、locked workspace test、reviewed branch の
hosted checks を実行します。

## Out of scope

Rust Runtime production code、release/adopter harness、immutable な `.ai` archive/decision bytes、
global Agent/MCP configuration、無関係な reference comparison batch は対象外です。

[English](WI-319-close-decision-and-promotion-compatibility.md) ·
[简体中文](WI-319-close-decision-and-promotion-compatibility.zh-CN.md)
