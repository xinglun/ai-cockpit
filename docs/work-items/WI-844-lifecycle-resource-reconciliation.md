---
author: AI Cockpit maintainers
title: "WI-844 — lifecycle resource reconciliation"
description: "Preserve historical successor bindings while reconciling Work Item lifecycle and exact resources."
audience: [maintainer, reviewer]
status: in_progress
authority: authorized
workItemId: WI-844-lifecycle-resource-reconciliation
---

[简体中文](WI-844-lifecycle-resource-reconciliation.zh-CN.md) · [日本語](WI-844-lifecycle-resource-reconciliation.ja.md)

# WI-844 — lifecycle resource reconciliation

## Intent and boundary

This Work Item reconciles active Work Item lifecycle state with the exact
remote branches and local worktrees that remain after reviewed merges. It
preserves historical Contract, Summary, Outcome, event, and evidence bytes;
unsupported or missing historical evidence remains explicitly classified.
Only an exact clean resource with a proven reviewed merge and terminal
lifecycle may be removed. Release tags, published Releases, product behavior,
and unrelated source changes are outside this boundary.

## Recovery boundary

An in-scope lifecycle defect is amended and revalidated here. A successor is
reserved for a different scope, authority, or base, an independent change, an
unsafe in-scope repair, immutable failed delivery, or explicit human direction.
An already-bound successor may supersede an amended predecessor only through
the immutable checkpoint Contract digest; an unbound or competing successor
remains fail-closed.

## Acceptance

- Every active Work Item has an evidence-backed disposition: closed, blocked
  with a concrete reason, or retained for current implementation.
- Historical records are preserved byte-for-byte and are never completed by
  fabricated fields or a rerun against the current repository snapshot.
- Only exact clean merged branches and worktrees are removed after lifecycle
  closure; all other resources retain a reason and remain recoverable.
- A predecessor Contract amendment does not invalidate an existing strictly
  bound successor's checkpoint digest or force historical product verification.

## Verification

- `cargo test --locked -p cockpit-repository --test recovery_decision`
- `cargo fmt --all -- --check`
- `git diff --check`
