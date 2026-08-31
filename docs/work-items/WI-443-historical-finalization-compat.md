---
title: "WI-443 — Historical finalization compatibility"
workItemId: WI-443-historical-finalization-compat
description: "Honest recovery paths for legacy shared worktrees and direct merges without PRs."
audience: [maintainer, adopter, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-443-historical-finalization-compat
---

# WI-443 — Historical finalization compatibility

This adds explicit `historical_low` records for legacy shared-primary worktrees
and direct local merges without a PR. Direct merges bind the real commit,
parents, base, repository identity, and authority; Runtime checks Git and never
invents a PR. Readiness classifies repository-wide historical debt and offers a
recovery action without weakening the new entry gate.

The object repositories remain frozen until this Runtime release is installed
and their immutable history is re-evaluated.

[简体中文](WI-443-historical-finalization-compat.zh-CN.md) ·
[日本語](WI-443-historical-finalization-compat.ja.md)
