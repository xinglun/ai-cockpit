---
author: AI Cockpit maintainers
title: "WI-514 — historical finalization 互換"
description: "legacy shared worktree と direct-merge を、履歴を書き換えず evidence-bound に recovery projection する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-514-historical-close-compatibility
lastVerifiedBy: WI-514-historical-close-compatibility
---

[English](WI-514-historical-close-compatibility.md) · [简体中文](WI-514-historical-close-compatibility.zh-CN.md)

## Goal

Runtime upgrade 後も、immutable predecessor を書き換えずに、事実を検証できる legacy finalization を再検証できるようにする。historical evidence は current high-assurance evidence には昇格しない。

## Scope と boundary

- local provider、primary checkout の shared worktree、`retained`、branch/worktree/repository/Contract/cleanliness の完全な binding が確認できる場合だけ `historical_low` に投影する。
- linked worktree、foreign provider、曖昧な topology、malformed/stale facts は fail-closed のままにする。
- historical direct-merge は実際の merge commit、parents、base revision、repository identity を使い、PR number を捏造しない。

## Evidence

- `.ai/evidence/WI-514-historical-close-compatibility.verification.json`
- `crates/cockpit-repository/tests/resource_finalization_transition.rs`
- `docs/reference/work-item-lifecycle-closure.ja.md`

archive、recovery、projection は元の receipt bytes を保持し、repository-bound recovery facts だけを append する。

## Non-claims

object repository、provider authorization、release packaging、無関係な lifecycle policy は変更しない。`historical_low` は新しい green verification ではない。
