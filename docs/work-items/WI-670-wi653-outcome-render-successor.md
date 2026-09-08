---
author: AI Cockpit maintainers
title: "WI-670 — WI-653 Outcome render successor"
description: "Redeliver the P1-A Outcome presentation boundary from the latest default branch with repository facts assembled before rendering."
audience: [contributor, maintainer, reviewer]
status: in_progress
authority: human:repository-owner
workItemId: WI-670-wi653-outcome-render-successor
lastVerifiedBy: WI-670-wi653-outcome-render-successor
---

[简体中文](WI-670-wi653-outcome-render-successor.zh-CN.md) · [日本語](WI-670-wi653-outcome-render-successor.ja.md)

# WI-670 — WI-653 Outcome render successor

## Intent

Redeliver the bounded P1-A architecture change from the latest remote default
branch after the original WI-653 remained archived and unmerged. The renderer
accepts an assembled `OutcomeRenderInput`; repository observation, human-decision
loading, lifecycle status, and archive-state checks remain in the assembly use
case used by CLI and MCP.

## Boundary

This successor preserves the current Outcome trust projection, machine JSON,
exit codes, authorization semantics, Contract-language boundary, and governance
facts. It does not change Outcome wording or localization, introduce a second
governance rule set, or alter another Work Item's branch, worktree, archive, or
evidence. The current-main `outcome_report.rs` integration test is in scope
because its fixture must migrate to the pure renderer input boundary.

## Change and verification

- `outcome_render_input` and its Runtime-bound variant assemble the validated
  facts once; `render_human_outcome` performs formatting only.
- CLI, MCP, lifecycle handoff, blocked handoff, and integration fixtures use
  the assembled input.
- In-memory renderer tests cover missing, valid, invalid, archived-unclosed,
  and superseded-history facts without a repository directory.
- Required checks are `cargo fmt --all -- --check`, workspace Clippy with
  warnings denied, workspace tests, documentation acceptance, and governance
  checks on the exact successor head.

## Out of scope

P0 responsibility mapping, P1-B observation-context redesign, P2 lifecycle or
storage extraction, P2-B state-type redesign, P2-C multi-file consistency, and
P3 physical-execution sharing remain separate Work Items.
