---
author: AI Cockpit maintainers
title: "WI-212 — WI-211 finalization recovery"
description: "Recover the immutable WI-211 archive and restore the required PR resource-finalization order."
audience:
  - maintainer
  - reviewer
workItemId: WI-212-release-fixture-finalization-recovery
status: current
authority: canonical
lastVerifiedBy: WI-212-release-fixture-finalization-recovery
---

# WI-212 — WI-211 finalization recovery

WI-211 was verified and archived before `finalize-plan` bound PR #160. The
installed Runtime correctly refused the later finalization attempt because
verification evidence was already recorded. This successor preserves WI-211
as immutable recovered history and restores the missing resource-finalization
boundary without rewriting its bytes.

## Acceptance

1. WI-211 is linked by a strict successor recovery receipt and remains
   immutable.
2. PR #160 resource context is bound before this successor's verification and
   its pre-merge receipt remains `awaiting_merge_close` until merge.
3. Governance gates pass with WI-211 recovered and WI-212 awaiting merge close.
4. WI-212 is closed only after hosted merge, exact cleanup, finalize-verify,
   and structured human decision.

## Out of scope

Rewriting WI-211 records, moving v0.2.26, reference-source file comparison,
and user-global Agent/MCP configuration are out of scope.
