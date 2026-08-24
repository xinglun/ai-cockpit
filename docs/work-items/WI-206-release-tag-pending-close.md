---
author: AI Cockpit maintainers
title: "WI-206 — release-tag pending-close governance boundary"
description: "Allow a proven post-merge release tag to publish while retaining the required Runtime closure step."
audience:
  - maintainer
  - adopter
workItemId: WI-206-release-tag-pending-close
status: implemented
authority: canonical
lastVerifiedBy: WI-206-release-tag-pending-close
---

# WI-206 — release-tag pending-close governance boundary

The v0.2.25 source-quality gate correctly rejected a tag whose current-cycle
Work Item was merged but not yet closed. That exposed an ordering deadlock:
the post-merge finalization transition requires the Runtime from the Release,
while the Release gate ran before that Runtime could be installed.

This Work Item makes the boundary explicit. A release tag may temporarily
project `awaiting_merge_close` only when a valid pre-merge finalization receipt
is identity-bound and its recorded PR head is proven to be an ancestor of the
tagged commit. The public binary must still complete finalization and the
structured human close after publication. Ordinary branches and unproven tags
remain fail-closed.

## Acceptance boundary

1. Valid release-tag ancestor proof is accepted only as `awaiting_merge_close`.
2. Non-ancestor, malformed, foreign, and ordinary-branch cases remain blocked.
3. English, Simplified Chinese, and Japanese workflow documentation state the
   release ordering and the post-release closure requirement.
