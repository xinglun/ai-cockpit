---
author: AI Cockpit maintainers
title: "WI-400 — v0.2.40 public Release adopter acceptance"
description: "Validate the immutable v0.2.40 Release binary from zero in an isolated adopter repository."
workItemId: WI-400-release-v0-2-40-adopter-acceptance
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-400-release-v0-2-40-adopter-acceptance
capabilityClaims: [release_acceptance, repository_isolation, evidence_reuse]
---

# WI-400 — v0.2.40 public Release adopter acceptance

[简体中文](WI-400-release-v0-2-40-adopter-acceptance.zh-CN.md) · [日本語](WI-400-release-v0-2-40-adopter-acceptance.ja.md)

## Intent

Prove that the immutable public v0.2.40 Release can govern a fresh adopter
from zero, with runtime identity, evidence reuse, lifecycle records, and
global-root isolation all independently auditable.

## Boundary

This Work Item covers only post-release artifact acceptance, the temporary
adopter and its cleanup receipt, promotion of the closed WI-399 projection,
and preservation of the generated acceptance evidence. It does not change
Runtime semantics, reference-source parity, business-project code, or global
Agent/MCP configuration. The harness must never fall back to a source build.

## Acceptance

1. The public v0.2.40 archive and binary are downloaded and checked against
   their release manifest and SHA-256 identities.
2. A fresh adopter receives an isolated scaffold and distinct repository
   identity; `first-adopter-smoke` remains `not_ready` until human fields are
   supplied.
3. A real Work Item lifecycle records schema-2 evidence, exact reuse and
   re-execution behavior, structured close decision, and runtime identity.
4. HOME/XDG roots remain unchanged, isolated runtime-write roots are accounted
   for, and the temporary run root is removed after the receipt is written.

## Verification boundary

The published Release is the only Runtime under test. Acceptance artifacts are
post-release evidence and never rewrite Release truth or historical records.
