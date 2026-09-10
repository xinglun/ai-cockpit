---
author: AI Cockpit maintainers
title: "WI-779 — abandoned finalization repair"
description: "Add a truthful terminal state for explicitly closed unmerged failed deliveries."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
workItemId: WI-779-abandoned-finalization-repair
lastVerifiedBy: WI-779-abandoned-finalization-repair
---

[简体中文](WI-779-abandoned-finalization-repair.zh-CN.md) · [日本語](WI-779-abandoned-finalization-repair.ja.md)

# WI-779 — abandoned finalization repair

## Intent and boundary

WI-779 repairs the Runtime resource-finalization boundary for a failed
delivery whose provider Pull Request was explicitly closed without merging.
The new `abandoned` terminal state is a truthful failure record: it requires
the exact `unmerged_pull_request` failure code, no merge commit, and removed
branch/worktree resources. It is never projected as a merged success.

PR #760 is retained as closed historical input for the failure analysis. This
Work Item starts from current `main`; it does not revive that PR or branch and
does not rewrite immutable WI-774, WI-775, WI-776, or WI-778 records.

## Scope

- Extend the typed Runtime protocol and repository close/finalization checks.
- Add focused protocol, repository, CLI, and documentation-promotion
  regression coverage, including fail-closed invalid abandoned receipts.
- Document the boundary in English, Simplified Chinese, and Japanese and
  register this Work Item in the reference-parity tables.

Performance implementation and measurement, release publication, unrelated
product behavior, provider API behavior, and user-benefit claims are outside
scope.

## Verification

The declared workspace verification is `cargo test --locked --workspace`.
Focused checks cover the protocol, repository, CLI, and closed-documentation
promotion suites. The hosted quality route must pass documentation acceptance,
governance integrity, status consistency, and all existing repository gates.

