---
author: Codex
title: WI-642 documentation promotion
description: Promote the closed WI-642 terminal documentation projection into its verified terminal state.
audience: maintainers, agents, reviewers
workItemId: WI-643-wi642-doc-promotion
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-643-wi642-doc-promotion
---

# WI-643 — WI-642 documentation promotion

This Work Item applies the deterministic post-close projection for WI-642 to
its English, Chinese, and Japanese Work Item pages and parity rows. It does not
change the immutable WI-642 Contract, verification evidence, archive,
finalization, or close records.

## Scope and acceptance

- Run `tests/docs/promote_closed_work_item.py` for WI-642.
- Add this Work Item's own tri-language pages and pre-archive parity entries.
- Update only helper-controlled status, verification, and terminal fields.
- Confirm documentation and governance gates pass.
- Preserve immutable governance bytes and keep object repositories out of scope.

## Evidence and next action

Verification evidence is produced by the installed Runtime after the
projection is applied. The Work Item remains in progress until reviewed,
merged, and closed through the normal lifecycle.
