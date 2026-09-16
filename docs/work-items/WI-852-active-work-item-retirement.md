---
author: AI Cockpit maintainers
title: "WI-852 — active Work Item retirement"
description: "Retire already integrated active Work Items without rewriting their original bytes or claiming verification."
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
workItemId: WI-852-active-work-item-retirement
lastVerifiedBy: WI-852-active-work-item-retirement
---

[简体中文](WI-852-active-work-item-retirement.zh-CN.md) · [日本語](WI-852-active-work-item-retirement.ja.md)

# WI-852 — active Work Item retirement

This Work Item adds an explicit Runtime retirement route for active records
whose delivery is already represented on the synchronized base, or whose
unfinished scope has an explicitly linked successor. Retirement preserves the
original Contract, Summary, Outcome, event, and attempt bytes in an immutable
archive projection. It is not verification, completion, or permission to
rewrite history.

The route rejects stale, foreign, malformed, duplicate, or unlinked input
before writing archive state. A retirement receipt binds the repository,
Work Item, Contract, Summary, snapshot, Runtime, and every preserved artifact.
