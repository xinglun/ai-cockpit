---
author: AI Cockpit maintainers
title: "WI-514 — historical finalization compatibility"
description: "Provide a narrow, evidence-bound recovery projection for legacy shared worktrees and direct merges without rewriting history."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-514-historical-close-compatibility
lastVerifiedBy: WI-514-historical-close-compatibility
---

[简体中文](WI-514-historical-close-compatibility.zh-CN.md) · [日本語](WI-514-historical-close-compatibility.ja.md)

## Goal

Allow an upgraded Runtime to revalidate honest legacy finalization records
without treating them as current high-assurance evidence or rewriting the
immutable predecessor.

## Scope and boundary

- A legacy local provider, primary-checkout shared worktree with `retained`
  may be projected as `historical_low` only when branch, worktree, repository,
  Contract, and cleanliness facts are all bound and verifiable.
- Ordinary retained linked worktrees, foreign providers, ambiguous topology,
  malformed receipts, and stale facts remain fail-closed.
- Historical direct merges use the real merge commit, parents, base revision,
  and repository identity; a PR number is never invented.

## Evidence

- `.ai/evidence/WI-514-historical-close-compatibility.verification.json`
- `crates/cockpit-repository/tests/resource_finalization_transition.rs`
- `docs/reference/work-item-lifecycle-closure.md`

The archive, recovery, and projection paths preserve the original receipt
bytes and append only repository-bound recovery facts.

## Non-claims

This does not change object repositories, provider authorization, release
packaging, or unrelated lifecycle policy. `historical_low` is not a fresh
green verification result.
