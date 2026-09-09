---
author: AI Cockpit maintainers
title: "WI-715 — WI-713 P0-B uniquely identified redelivery"
description: "最新の default base から、一意な Work Item identity で決定的な Outcome summary と完全な evidence view を再配信します。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: human:repository-owner
workItemId: WI-715-wi713-p0b-redelivery
lastVerifiedBy: WI-715-wi713-p0b-redelivery
---

[English](WI-715-wi713-p0b-redelivery.md) · [简体中文](WI-715-wi713-p0b-redelivery.zh-CN.md)

# WI-715 — WI-713 P0-B uniquely identified redelivery

## Intent

別 scope の closed Work Item が WI-714 を使用しており、deterministic な closed
Work Item promotion を一意に結び付けられなかったため、current remote default branch
から P0-B を再配信します。既定の human handoff は result、key changes、remaining
uncertainty、human next step の四つの deterministic section で構成し、完全な evidence
report は明示的に表示できます。

## Boundary

この Work Item は表示層だけを変更し、machine JSON、validation rule、authorization
semantics、exit code、永続化形式、historical evidence を保持します。CLI と MCP は同じ
repository renderer の facts を使い、測定済みの認知効果や実ユーザー研究は主張しません。

## Base と recovery lineage

- Current Remote/default base: `origin/main` at `7ada6cd0928a877fe2bc719689abfe99bd532d7e`。
- Predecessor: WI-714。recovered predecessor として保持し、その recovery bytes は immutable のままです。
- Predecessor recovery decision: `.ai/decisions/WI-714-wi713-current-base-revalidation.recovery.3efe3143da8d84cb32db0877de59ee702b12034925d300b895f5449cb756d676.json`。
- Historical evidence: `.ai/evidence/WI-714-wi713-current-base-revalidation.verification.json`。

## Acceptance

- CLI/MCP の既定 handoff は四つの reader-first section を使います。
- `view: full` / `--view full` は audit 用の完全 report を保持します。
- blocker、human decision、stale/invalid evidence、不確実性を隠しません。非 critical
  list のみ省略でき、完全 report への route を示します。
- 実際の Outcome structure を使う test で多言語と historical case を検証し、machine JSON
  は変更しません。

## Current state

unique successor は current default base から active です。verification、hosted delivery、
provider finalization、明示的な human close decision は未完了です。
