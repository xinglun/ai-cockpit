---
author: AI Cockpit maintainers
title: "WI-283 — reference Contract semantics on current main"
workItemId: WI-283-reference-contract-semantics-current-main
description: "Revalidate the bounded Rust Contract-semantics parity batch from the latest reviewed default branch after WI-282 was rejected for an older base revision."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-283-reference-contract-semantics-current-main
authority: canonical
---

# WI-283 — reference Contract semantics on current main

WI-283 is the explicit successor of immutable WI-282. It revalidates the same
bounded Contract-semantics implementation from default branch revision
`622836157e945a46f8cb34ee747084d3193e7f28`, while preserving all predecessor
Contract, evidence, archive, and recovery bytes. The predecessor was not
rewritten; its hosted-quality rejection is recorded as recovery history.
