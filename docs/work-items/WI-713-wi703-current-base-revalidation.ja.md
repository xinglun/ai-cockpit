---
author: AI Cockpit maintainers
title: "WI-713 — WI-703 P0-B current-base revalidation"
description: "最新の default base から決定的な Outcome summary と明示的な完全 evidence view を再配信します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-713-wi703-current-base-revalidation
lastVerifiedBy: WI-713-wi703-current-base-revalidation
---

[English](WI-713-wi703-current-base-revalidation.md) · [简体中文](WI-713-wi703-current-base-revalidation.zh-CN.md)

# WI-713 — WI-703 P0-B current-base revalidation

## Intent

前身 Work Item の archive 中に PR #703 が進んだ default base と競合したため、最新の
remote default branch から P0-B を再配信します。既定の human handoff は result、key
changes、remaining uncertainty、human next step の四つの deterministic section で構成し、
完全な evidence report は明示的に表示できます。

## Boundary

この Work Item は表示層だけを変更し、machine JSON、validation rule、authorization
semantics、exit code、永続化形式、historical evidence を保持します。CLI と MCP は同じ
repository renderer の facts を使い、測定済みの認知効果や実ユーザー研究は主張しません。

## Base と recovery lineage

- Remote/default base: `origin/main` at `00700d88647246622729bca2d623eafd779eb1f3`。
- Predecessor: WI-703。archive と historical verification は immutable のままです。
- Recovery decision: `.ai/decisions/WI-703-wi669-current-base-revalidation.recovery.json`。
- Historical evidence: `.ai/evidence/WI-703-wi669-current-base-revalidation.verification.json`。

## Acceptance

- CLI/MCP の既定 handoff は四つの reader-first section を使います。
- `view: full` / `--view full` は audit 用の完全 report を保持します。
- blocker、human decision、stale/invalid evidence、不確実性を隠しません。非 critical
  list のみ省略でき、完全 report への route を示します。
- 実際の Outcome structure を使う test で多言語と historical case を検証し、machine JSON
  は変更しません。

## Current state

latest-base の recovery branch で実装中です。verification、hosted delivery、provider
finalization、明示的な human close decision は未完了です。
