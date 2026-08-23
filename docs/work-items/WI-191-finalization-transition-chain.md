---
author: AI Cockpit maintainers
title: WI-191 — Append-only finalization transition chain
workItemId: WI-191-finalization-transition-chain
description: "Append-only finalization transitions preserve immutable pre-merge evidence across merge and cleanup."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-191-finalization-transition-chain
---

# WI-191 — Append-only finalization transition chain

WI-190 exposed that a valid pre-merge blocked canonical receipt could not advance after merge and cleanup. WI-191 preserves that receipt and adds typed, digest-addressed transitions with exact state continuity and a unique-head resolver. Merge observation and cleanup are separate transitions; `finalize-verify` and `close` bind the latest head. Foreign, stale, forked, malformed, symlinked, and sequence-invalid chains fail closed, while legacy canonical receipts remain supported.
