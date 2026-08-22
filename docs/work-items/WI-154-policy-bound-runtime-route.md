---
author: AI Cockpit maintainers
workItemId: WI-154-policy-bound-runtime-route
title: Policy-bound Runtime verification route
description: Connect policy requirements and stage/base facts to actual verification receipts without weakening no-policy compatibility.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-154-policy-bound-runtime-route
---

# WI-154 — Policy-bound Runtime verification route

The Runtime now resolves a declared repository/Work Item verification
requirement before execution. `VerificationTier` and `EvidenceAssurance` stay
orthogonal: a local route cannot satisfy `T3` or `ProviderVerified` merely by
running a successful command. `pr`, `merge`, and `release` stages require a
valid Contract `baseRevision`; `task` does not.

New Work Item receipts bind repository and Work Item identity, snapshot digest,
base revision, policy references, required and actual route dimensions,
affected paths, and dependency confidence. Lifecycle validation rechecks these
bindings, so receipt tampering cannot become finish/archive truth. Repositories
without a typed verification requirement retain the no-policy/legacy route.

See [Verification route](../reference/verification-route.md) and the
[Chinese](WI-154-policy-bound-runtime-route.zh-CN.md) and
[Japanese](WI-154-policy-bound-runtime-route.ja.md) versions.
