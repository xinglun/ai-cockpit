---
author: AI Cockpit maintainers
title: "WI-670 — WI-653 Outcome 表示層 successor"
description: "最新の default branch から P1-A の Outcome 表示境界を再配信し、描画前に repository facts を組み立てる。"
audience: [contributor, maintainer, reviewer]
status: implemented
authority: human:repository-owner
workItemId: WI-670-wi653-outcome-render-successor
lastVerifiedBy: WI-670-wi653-outcome-render-successor
terminalArchive: .ai/work-items/archive/WI-670-wi653-outcome-render-successor.contract.json
terminalVerification: .ai/evidence/WI-670-wi653-outcome-render-successor.verification.json
terminalFinalization: .ai/decisions/WI-670-wi653-outcome-render-successor.finalize.json
terminalDecision: .ai/decisions/WI-670-wi653-outcome-render-successor.close.json
---

[English](WI-670-wi653-outcome-render-successor.md) · [简体中文](WI-670-wi653-outcome-render-successor.zh-CN.md)

# WI-670 — WI-653 Outcome 表示層 successor

## Intent

元の WI-653 は archive 済みで未 merge のため、最新の remote default branch から
限定された P1-A の構造変更を再配信する。renderer は組み立て済みの
`OutcomeRenderInput` だけを受け取り、repository observation、人間の判断の読み取り、
lifecycle status、archive-state check は CLI/MCP 共通の assembly use case が担当する。

## Boundary

現在の Outcome trust projection、machine JSON、exit code、authorization semantics、
Contract 原文境界、governance facts を維持する。Outcome 文言や localization、第二の
governance rule set、他 Work Item の branch/worktree/archive/evidence は変更しない。
current main の `outcome_report.rs` integration test は pure renderer input boundary へ
移行する必要があるため範囲に含める。

## Change and verification

- `outcome_render_input` と Runtime-bound variant が validated facts を一度だけ組み立て、
  `render_human_outcome` は formatting のみを行う。
- CLI、MCP、lifecycle handoff、blocked handoff、integration fixture は assembly input を使う。
- in-memory unit test は missing/valid/invalid decision、archived-unclosed、superseded history
  を repository directory なしで検証する。
- exact successor head で fmt、strict Clippy、workspace tests、documentation acceptance、
  governance checks を通過させる。

## Out of scope

P0 responsibility map、P1-B observation context、P2 lifecycle/storage、P2-B state types、
P2-C multi-file consistency、P3 physical-execution sharing は別 Work Item とする。
