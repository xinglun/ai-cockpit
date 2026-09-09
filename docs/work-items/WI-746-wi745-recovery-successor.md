---
author: AI Cockpit maintainers
title: "WI-746 — WI-745 parity-order recovery successor"
description: "Redeliver the bounded documentation recovery with parity registration committed before fresh verification evidence."
audience: [maintainer, reviewer, adopter]
workItemId: WI-746-wi745-recovery-successor
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-746-wi745-recovery-successor
---

[简体中文](WI-746-wi745-recovery-successor.zh-CN.md) · [日本語](WI-746-wi745-recovery-successor.ja.md)

# WI-746 — WI-745 parity-order recovery successor

## Intent

Preserve WI-745 as an immutable predecessor and redeliver its narrow
documentation-governance recovery with an explicit parity-registration-before-
verification boundary. This Work Item does not change WI-743 or WI-745
historical evidence.

## Boundary

This Work Item owns only the tri-language WI-746 documentation projection and
its three reference-parity rows. It does not change production code, Runtime
behavior, governance rules, WI-743 or WI-745 records, PR #718, or another
agent's Work Item.

## Evidence and lifecycle

- WI-745's recovery decision is the explicit predecessor binding.
- The documentation pages and parity rows are committed first.
- Fresh Runtime verification evidence is recorded only after that commit.
- The governed lifecycle is `start → preflight → checkpoint → verify →
  finish → archive → close`; terminal paths remain planned until finalization.

## Acceptance

Documentation acceptance, the parity status check, and Work Item status
consistency check must pass. The row-before-evidence ordering must be
reviewable in Git history, and no historical record or production behavior is
rewritten.
