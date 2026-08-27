---
author: AI Cockpit maintainers
title: "WI-313 — post-close finalization reconciliation"
workItemId: WI-313-post-close-finalization-reconciliation
description: "Enforce cleanup-before-close and provide a narrowly bound recovery path for immutable legacy close records."
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-313-post-close-finalization-reconciliation
---

# WI-313 — post-close finalization reconciliation

## Intent and boundary

W312 exposed a real ordering defect: an older Runtime could close a Work Item
while its provider finalization remained `retained`, then the closed-document
promotion gate correctly refused to claim terminal cleanup. This Work Item
fixes the Runtime boundary and preserves the historical bytes. New Work Items
must clean provider resources before close; only an immutable legacy close may
receive one bound deleted transition afterward.

## Scope and acceptance

The Rust protocol/repository lifecycle rejects retained, blocked, or unknown
finalization at close and accepts a post-close transition only when it binds
the closed root digest, Work Item/repository identity, next sequence, and exact
deleted branch/worktree state. The close and original finalization bytes remain
unchanged. The documentation promotion gate and tri-language workflow explain
the normal and legacy paths and reject every unbound or incomplete exception.

## Verification

Targeted Rust finalization tests, the closed-document promotion fixture,
formatting, linting, workspace tests, and the repository documentation gates
are required. Hosted CI and the installed Runtime identity are recorded in the
final evidence; no source-build fallback is a release acceptance substitute.
