---
author: AI Cockpit maintainers
title: "WI-157 — v0.2.17 release and adopter acceptance"
description: "Publish an immutable Runtime and prove it can govern a fresh adopter repository."
audience:
  - adopter
  - contributor
  - maintainer
status: in_progress
authority: canonical
lastVerifiedBy: WI-157-release-v0-2-17-adopter-acceptance
workItemId: WI-157-release-v0-2-17-adopter-acceptance
---

# WI-157 — v0.2.17 release and adopter acceptance

This Work Item publishes the next Runtime only after source, package, and
documentation identities agree. Its post-release acceptance uses the immutable
public archive, never a workspace build or fallback binary, and records the
Runtime digest, adopter repository identity, isolation manifests, evidence
reuse, scaffold `not_ready` state, and complete Work Item lifecycle receipts.

The acceptance receipt is post-release evidence. A failed receipt cannot rewrite
the already-published Release truth.
