---
author: AI Cockpit maintainers
title: "WI-249 — Parity finalization registration"
workItemId: WI-249-parity-finalization-registration
description: "Make parity-writing Work Items register lifecycle-bound terminal paths before verification."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-249-parity-finalization-registration
authority: canonical
---

# WI-249 — Parity finalization registration

WI-249 recovers the immutable WI-247 predecessor and removes the ordering loop
that required a documentation mutation after archive. Its own row is committed
before verification and names the future archived Contract, verification,
canonical finalize, and close paths. The status is explicitly conditional:
`In progress → Implemented after verified close`. It does not claim completion
before PR #199 is reviewed, merged, finalized, cleaned up, and closed.

## Conditional control and quality profiles

The governance integrity gate inspects an active Contract's scope and
acceptance plus the active Summary's changed paths. It requires the three exact
lifecycle-bound rows only when one of those declarations owns
`docs/reference/reference-parity*` or parity registration. The static selector
runs in the light profile; standard and strict inherit it. Non-parity code Work
Items remain `active_non_parity` and do not acquire documentation scope merely
because a broader profile runs.

The archived-code pending registry remains a separate temporary bridge. Its
repository/PR/head/base/record bindings, registry-only append topology,
regular-file containment, and default-branch stale behavior are unchanged.

## Fail-closed evidence

The regression proves missing, partial, terminal-only, foreign-path, and
post-archive-only projections fail deterministically. For a valid row, Git
blame identifies the row commit and Git history proves it strictly precedes the
verification evidence addition. The same row bytes then pass active,
awaiting-merge-close, and closed states without rewriting archive evidence.
