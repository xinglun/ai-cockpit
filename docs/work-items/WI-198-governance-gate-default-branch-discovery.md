---
author: AI Cockpit maintainers
title: "WI-198 — Governance gate default-branch discovery"
description: "Make pre-merge governance validation deterministic in detached pull-request checkouts without weakening identity checks."
audience:
  - maintainer
  - reviewer
workItemId: WI-198-governance-gate-default-branch-discovery
status: recovered
authority: canonical
lastVerifiedBy: WI-198-governance-gate-default-branch-discovery
---

# WI-198 — Governance gate default-branch discovery

WI-198 is the explicit successor to immutable WI-197. Hosted quality showed
that a detached pull-request merge checkout can lack both `origin/HEAD` and
event base-branch metadata. The gate now uses the Contract's immutable
`resourceContext.baseBranch` only as a narrow fallback, while retaining every
repository, PR, branch, worktree, evidence, runtime, and digest binding.

The regression covers both a valid metadata-free checkout and an externally
declared base-branch mismatch. WI-197 remains immutable and is linked through
its recovery receipt.

[简体中文](WI-198-governance-gate-default-branch-discovery.zh-CN.md) ·
[日本語](WI-198-governance-gate-default-branch-discovery.ja.md)
