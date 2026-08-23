---
author: AI Cockpit maintainers
title: "WI-157 — v0.2.17 release and adopter acceptance"
description: "Publish an immutable Runtime and prove it can govern a fresh adopter repository."
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
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

Release: https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.17

Workflow: https://github.com/xinglun/ai-cockpit/actions/runs/32606940727

Local public-artifact evidence: `.ai/evidence/external/v0.2.17/adopter/` and
`.ai/evidence/external/v0.2.17/upgrade/`. The installed public binary is
Runtime `0.2.17`, digest `sha256:4157cc04a23a24e6ac618e7079c123210920fba2e7fc5335c9f6a734c74721e3`.
The pre-release v0.2.16 evidence bytes remain preserved at
`.ai/evidence/external/v0.2.16/WI-157-release-v0-2-17-adopter-acceptance/` and
are not reused as current verification evidence.
