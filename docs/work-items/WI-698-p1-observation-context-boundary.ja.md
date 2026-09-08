---
author: AI Cockpit maintainers
title: “WI-698 — P1 明示的 observation context 境界”
description: “一つの governance judgment を一つの検証済み phase-scoped repository observation context に束ねます。”
audience: [contributor, maintainer, reviewer]
workItemId: WI-698-p1-observation-context-boundary
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-698-p1-observation-context-boundary
---

[English](WI-698-p1-observation-context-boundary.md) · [简体中文](WI-698-p1-observation-context-boundary.zh-CN.md)

# WI-698 — P1 明示的 observation context 境界

## Intent

snapshot の有効期間を mutation の境界を越えて拡張せず、残っていた P1-B の架構境界を完成させます。各 governance judgment は、下位 helper が repository identity と snapshot fact を暗黙に再解決するのではなく、明示的な request-scoped observation phase を使います。

## Boundary

`RepositoryExecutionContext` は `before_governance`、`after_execution`、`before_persistence` の named phase ごとに `ObservationContext` を作ります。context は repository identity、任意の Runtime identity、Contract identity、source snapshot digest、governance configuration/policy identity、parsed repository observation、project-governance facts、unknowns と consistency を束ねます。

`validate_current` は新しい境界チェックを行います。source、attached identity、Contract、governance configuration が変わった場合、古い context は拒否されます。また別の lifecycle phase への再利用も許可しません。preflight governance path はこの context を使い、parsed project-governance facts を再利用します。既存の compatibility wrapper と protocol bytes は維持します。

## Compatibility と制限

新しい crate、trait、global cache、protocol field、governance rule、Outcome wording、lifecycle transition、physical-execution policy は追加しません。context は transaction ではなく、複数ファイルの read を atomic にするものでもありません。execution 後または persistence 前には新しい phase context を capture します。benchmark 済みの性能向上は主張しません。

## Verification

focused test は request 内の reuse、source mutation、governance configuration mutation、repository identity mutation、Contract mutation、phase separation を確認します。finish 前に lifecycle preflight/order test と workspace gate を通過させ、terminal status は hosted check と Runtime の archive/finalization/close receipt で証明します。

## 残存リスク

他の lifecycle entry point は compatibility の root-plus-snapshot 形式をまだ保持します。全体移行は別の bounded Work Item で行います。本 WI は repository-wide atomic snapshot や cross-request cache を主張しません。

## Recovery boundary

archive された delivery は immutable な historical evidence として保持します。
default branch の進行後、元の hosted branch は WI-700、続いて WI-701 が current
default base から再検証します。この page は元 WI の archive、verification、recovery
record を書き換えません。
