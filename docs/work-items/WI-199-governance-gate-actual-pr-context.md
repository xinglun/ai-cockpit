---
author: AI Cockpit maintainers
title: "WI-199 — Governance gate actual PR context"
description: "Bind the detached-checkout governance gate correction to the actual reviewed branch and pull request before merge."
audience:
  - maintainer
  - reviewer
workItemId: WI-199-governance-gate-actual-pr-context
status: in_progress
authority: canonical
lastVerifiedBy: WI-199-governance-gate-actual-pr-context
---

# WI-199 — Governance gate actual PR context

WI-199 is the explicit successor to immutable WI-198. A post-archive
self-check found that WI-198 retained a stale branch name after the branch
rename. The default-branch discovery implementation remains unchanged; this
Work Item rebinds the reviewed PR #153 context to the actual branch and records
a strict pre-merge finalization receipt.

[简体中文](WI-199-governance-gate-actual-pr-context.zh-CN.md) ·
[日本語](WI-199-governance-gate-actual-pr-context.ja.md)
