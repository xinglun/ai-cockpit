---
author: AI Cockpit maintainers
title: "WI-716 — WI-715 governance scope recovery"
description: "不変の履歴を書き換えず、マージ済み WI-715 の scope-aware governance close を完了する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-716-wi715-governance-scope-recovery
lastVerifiedBy: WI-716-wi715-governance-scope-recovery
---

[English](WI-716-wi715-governance-scope-recovery.md) · [简体中文](WI-716-wi715-governance-scope-recovery.zh-CN.md)

# WI-716 — WI-715 governance scope recovery

## Intent

WI-715 が PR #707 としてマージされた後、Runtime は元の Contract が archive
された後に追加された parity/gate 修正が、元の Contract に宣言されていない
governance test path に触れていることを正しく検出した。本 Work Item は
有界な successor close を記録し、P0-B の製品実装をやり直さない。

## Boundary

完全な Work Item ID を扱う governance parity/status check と三言語の文書投影を
対象とする。Outcome の動作、検証規則、終了コード、machine JSON、認可 semantics、
WI-715 の archive/evidence bytes は変更しない。

## Base and recovery lineage

- 現在の remote/default base: `origin/main` at `d1141480fb7a045979098480c3770d06002e2a87`.
- Predecessor: WI-715。マージ済み delivery、finalization receipt、verification evidence は不変に保持する。
- Recovery decision: `.ai/decisions/WI-715-wi713-p0b-redelivery.recovery.json`.
- Predecessor finalization: `.ai/decisions/WI-715-wi713-p0b-redelivery.finalize.json`.
- PR #707: `https://github.com/xinglun/ai-cockpit/pull/707`.

## Acceptance

- Governance gate、documentation status consistency、promotion helper と回帰テストは、
  数字 prefix が同じ無関係な記録を統合せず、完全な Work Item ID を扱う。
- 三言語 parity projection は WI-715 の recovery facts を保持し、製品またはユーザー便益を
  捏造せずに successor の evidence と terminal decision を記録する。
- 現在の default base で `cargo test --locked --workspace`、documentation acceptance、
  repository governance checks が通過する。
- Provider finalization、正確な cleanup、human close、WI-715 の履歴 close は Runtime の生成証拠で表現する。

## Current state

Successor はマージ済み default base から active になった。Verification、reviewed delivery、
finalization、archive、明示的な human close は未完了である。
