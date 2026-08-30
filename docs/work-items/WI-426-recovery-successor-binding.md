---
author: AI Cockpit maintainers
title: "WI-426 — Recovery successor binding compatibility"
description: Preserve strict successor lineage while safely recognizing terminal legacy successors.
workItemId: WI-426-recovery-successor-binding
audience: [contributor, maintainer, reviewer]
status: recovered
authority: human-authorized
lastVerifiedBy: WI-426-recovery-successor-binding
---

# WI-426 — Recovery successor binding compatibility

## Intent

Close the lifecycle gap where an immutable archived predecessor has a valid
successor recovery receipt but an older successor Contract lacks the newer
predecessor fields. New Runtime-created successors remain strictly bound.

## Scope

- Keep new successor Contracts bound to predecessor Work Item, Contract digest,
  recovery path, and repository identity.
- Permit only a terminal, fully evidenced legacy successor through an explicit
  compatibility path and mark the new append-only recovery receipt.
- Reject foreign, stale, malformed, symlinked, incomplete, or tampered records.
- Preserve predecessor bytes and document the boundary in three languages.

## Evidence boundary

Legacy compatibility is not a fresh green result. It requires a valid archive
manifest, verified strict evidence, and a confirmed structured close for the
successor. The compatibility marker is
`successorBindingMode: legacy_terminal_evidence`.

[中文](WI-426-recovery-successor-binding.zh-CN.md) · [日本語](WI-426-recovery-successor-binding.ja.md)
