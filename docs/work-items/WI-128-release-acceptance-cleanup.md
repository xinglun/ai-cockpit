---
author: AI Cockpit maintainers
workItemId: WI-128-release-acceptance-cleanup
title: Release adopter acceptance cleanup and isolation truth
description: Make post-release acceptance cleanup fail closed and keep isolation receipts auditable.
audience:
  - maintainer
  - release-engineer
status: implemented
authority: canonical
lastVerifiedBy: release-acceptance
---

# WI-128 — Release acceptance cleanup

The N-1 post-release harness now carries one explicit exit-status variable
through its finish trap. A successful upgrade and cleanup therefore return zero;
an unset status cannot turn a valid acceptance into a shell error.

Both adopter harnesses retain validated temporary-root cleanup, immutable
`releasePublished` truth, cleanup receipts, and typed isolation manifests. The
manifest policy remains explicit: HOME/XDG configuration roots are forbidden
to Runtime writes, while TMPDIR/CARGO_HOME are isolated roots whose writes are
recorded and allowed.
