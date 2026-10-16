---
author: AI Cockpit maintainers
title: "WI-1087 — finite goal authority scope and expiry"
description: "Planned design for finite goal authorization with continuous work inside its approved boundary."
audience:
  - maintainer
  - reviewer
workItemId: WI-1087-contract-authority-scope-and-expiry
status: planned
authority: design-and-registration-only
---

[简体中文](WI-1087-contract-authority-scope-and-expiry.zh-CN.md) · [日本語](WI-1087-contract-authority-scope-and-expiry.ja.md)

# WI-1087 — finite goal authority scope and expiry

This is a planned design, recorded through Runtime new as planning/not_ready. No start, checkpoint, product implementation, actual permission activation, merge or cleanup. Product implementation authority remains missing.

## Goal

One verifiable finite goal authorization may cover explicitly allowed sequential/parallel Work Items, repair, retry and recovery without per-WI human reapproval. An old goal cannot authorize a new goal. notBefore/expiresAt are mandatory and finite; completion, revocation or expiry terminates future execution while preserving history.

## Versioned design

The versioned Chinese design is canonical; these three summaries refer to it. It specifies current source identity/goal coverage, narrowing only, locks/effects/time checks, bounded safe stopping, metadata-only extension requests, one approved extension for applicable active WIs, no self-renewal or revival, local-clock/identity limitations, and separate legacy migration.

- [Design v1](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-20261016/design-v1.zh-CN.md)
- [Acceptance v1](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-20261016/acceptance-v1.json)
- [Staged scope v1](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-20261016/implementation-stages-v1.json)

## Dependency and ownership

WI-1086 is the implementation dependency. Shared amendment/status/admission/receipt/lifecycle and ledgers serialize; no second build or parallel shared edits. Independent Clock/type work requires later design review, real ownership isolation and normal admission.
