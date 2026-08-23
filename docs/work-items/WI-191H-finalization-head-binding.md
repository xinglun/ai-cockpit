---
author: AI Cockpit maintainers
title: WI-191H — Governance finalization head binding
workItemId: WI-191H-finalization-head-binding
description: "Bind the governance receipt append that legitimately advances a finalization head."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-191H-finalization-head-binding
---

# WI-191H — Governance finalization head binding

WI-191's immutable pre-merge receipt correctly bound archive commit `70c17e4`, while committing that receipt advanced PR #152 to `8f5a025`. WI-191H represents that self-referential governance append explicitly instead of treating arbitrary head drift as identity. Only the first unmerged-to-merged transition may bind `governanceAppendRevision`; all three resource heads must advance together, the old head must be a Git ancestor, and every range change must be a newly added regular finalization receipt JSON for the same Work Item. The cleanup transition keeps the new head unchanged. Foreign paths, malformed receipt names, symlinks, non-append changes, non-ancestor revisions, non-merge transitions, and later drift fail closed.
