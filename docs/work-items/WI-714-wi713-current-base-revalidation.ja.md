---
author: AI Cockpit maintainers
title: "WI-714 — WI-713 P0-B current-base revalidation"
description: "最新の default base から決定的な Outcome summary と明示的な完全 evidence view を再配信します。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: human:repository-owner
workItemId: WI-714-wi713-current-base-revalidation
lastVerifiedBy: WI-714-wi713-current-base-revalidation
---

[English](WI-714-wi713-current-base-revalidation.md) · [简体中文](WI-714-wi713-current-base-revalidation.zh-CN.md)

# WI-714 — WI-713 P0-B current-base revalidation

## Intent

前身 Work Item の archive 中に PR #704 が進んだ default base と競合したため、最新の
remote default branch から P0-B を再配信します。既定の human handoff は result、key
changes、remaining uncertainty、human next step の四つの deterministic section で構成し、
完全な evidence report は明示的に表示できます。

## Boundary

この Work Item は表示層だけを変更し、machine JSON、validation rule、authorization
semantics、exit code、永続化形式、historical evidence を保持します。CLI と MCP は同じ
repository renderer の facts を使い、測定済みの認知効果や実ユーザー研究は主張しません。

## Base と recovery lineage

- Historical attempted base: `origin/main` at `94fec9e43ff0c046a236a100d881b032124e973e`。
- Predecessor: WI-713。archive と historical verification は immutable のままです。
- Recovery decision: `.ai/decisions/WI-713-wi703-current-base-revalidation.recovery.09a026473430b0e8855dabacf111b91686189e3de7c009b241c8a03ac886e5cc.json`。
- Historical evidence: `.ai/evidence/WI-713-wi703-current-base-revalidation.verification.json`。
- Successor: WI-715。別 scope の closed Work Item がすでに WI-714 を使用しているため、WI-715 が一意な再配信を担当します。
- WI-714 recovery decision: `.ai/decisions/WI-714-wi713-current-base-revalidation.recovery.3efe3143da8d84cb32db0877de59ee702b12034925d300b895f5449cb756d676.json`。

## Acceptance

- CLI/MCP の既定 handoff は四つの reader-first section を使います。
- `view: full` / `--view full` は audit 用の完全 report を保持します。
- blocker、human decision、stale/invalid evidence、不確実性を隠しません。非 critical
  list のみ省略でき、完全 report への route を示します。
- 実際の Outcome structure を使う test で多言語と historical case を検証し、machine JSON
  は変更しません。

## Current state

この Work Item は recovered predecessor として保持します。別 scope の WI-714 と番号が
衝突したため、元の配信はこの番号では close せず、WI-715 が歴史を変更せずに限定的な
再配信を担当します。
