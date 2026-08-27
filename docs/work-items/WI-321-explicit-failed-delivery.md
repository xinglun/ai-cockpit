---
author: AI Cockpit maintainers
title: "WI-321 — explicit failed-delivery recovery boundary"
workItemId: WI-321-explicit-failed-delivery
description: "Record a Runtime-bound successor for WI-313 without rewriting immutable failed-delivery history."
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-321-explicit-failed-delivery
---

# WI-321 — explicit failed-delivery recovery boundary

## Intent and boundary

WI-313 is an immutable failed delivery: PR #277 did not merge and its retry
receipt has no terminal decision or successor. This Work Item records a
Runtime-generated successor receipt so the governance gate cannot leave that
history orphaned or silently project it as a completed implementation.

The predecessor remains historical truth. This Work Item does not rewrite or
delete its Contract, Summary, Outcome, Events, archive, verification, retry
receipt, branch, or PR records. It does not claim that WI-313's implementation
was merged.

## Scope and acceptance

- The Runtime-generated WI-313 successor receipt is bound to this Work Item,
  repository identity, predecessor digests, Runtime identity, and explicit
  human authority.
- The governance integrity gate has a deterministic regression proving that an
  orphaned retry without a successor cannot be a terminal success, while an
  explicit successor is accepted as `Recovered`.
- English, Simplified Chinese, and Japanese Work Item/parity projections state
  the failed unmerged PR boundary and use the recovery receipt as evidence.
- Existing historical bytes and global Agent/MCP configuration remain
  untouched.

## Verification

Run the orphaned-retry and recovery-chain static regressions, documentation
acceptance, the locked workspace test suite, and hosted CI on the reviewed
branch. All repository-bound Runtime commands use the explicit repository
path; source-build fallback is not release evidence.

## Related history

- WI-313: immutable failed PR #277 delivery, now explicitly recovered by this
  successor.
- WI-314 and WI-315: separate recovery chains that remain unchanged.

[简体中文](WI-321-explicit-failed-delivery.zh-CN.md) ·
[日本語](WI-321-explicit-failed-delivery.ja.md)
