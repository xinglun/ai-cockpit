---
author: AI Cockpit maintainers
title: "WI-288 — Provider finalization correction"
workItemId: WI-288-provider-finalization-correction
description: "Re-deliver the predecessor implementation with an actual provider-bound PR identity and immutable recovery linkage."
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-288-provider-finalization-correction
authority: canonical
---

# WI-288 — Provider finalization correction

## Purpose

WI-287 was intentionally preserved as immutable history after its provider
context contained a placeholder PR URL. This successor re-delivers the same
implementation with the actual GitHub PR identity known before final
verification. It changes no predecessor bytes and adds no new runtime feature.

## Boundary

- Preserve the WI-287 archive and recovery decision exactly.
- Bind this Contract's `resourceContext` to the actual PR before verification.
- Re-run the installed Runtime verification and hosted checks.
- Record provider finalization, verify it, close with a structured decision,
  and remove only the exact merged branch/worktree.

## Object/adopter parity

The successor exercises the same explicit-repository, fail-closed Runtime
behavior that an adopter receives. The visible human Outcome remains the
handoff; no provider approval is inferred from local records.

## Verification

Declared verification: `cargo test --locked --workspace`, conformance and
documentation acceptance, hosted PR checks, provider finalization verification,
and post-close repository status/doctor checks.
