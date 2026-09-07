---
author: Codex
title: WI-640 documentation promotion
description: Promote the closed WI-640 documentation projection to its verified terminal state.
audience: maintainers, agents, reviewers
workItemId: WI-641-wi640-documentation-promotion
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-641-wi640-documentation-promotion
---

# WI-641 — WI-640 documentation promotion

This Work Item applies the deterministic post-close projection for WI-640 to
its English, Chinese, and Japanese Work Item pages and parity rows. It does not
change the immutable WI-640 Contract, verification evidence, archive,
finalization, or close records.

## Scope and acceptance

- Run `tests/docs/promote_closed_work_item.py` for WI-640.
- Update only the helper-controlled status, verification, and terminal fields.
- Confirm all documentation and governance gates pass.
- Preserve all immutable governance bytes and keep object repositories out of scope.

## Evidence and next action

Verification evidence is produced by the installed Runtime after the
projection is applied. The Work Item remains in progress until reviewed,
merged, and closed through the normal lifecycle.
