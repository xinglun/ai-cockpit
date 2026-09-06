---
author: AI Cockpit maintainers
title: "WI-614 — First-record direct-merge recovery round-trip"
description: "Make the recovery plan for a historical direct merge without a PR apply as a complete, fail-closed receipt."
audience:
  - adopter
  - maintainer
  - reviewer
workItemId: WI-614-direct-merge-no-pr-apply
status: in_progress
authority: canonical
lastVerifiedBy: WI-614-direct-merge-no-pr-apply
---

# WI-614 — First-record direct-merge recovery round-trip

## Intent

Make `work-item finalize-recovery-plan --merge-commit <sha>` emit a complete,
typed first-record `direct_merge_no_pr` receipt. An owner only supplies the
human authorization fields; Git parents, repository identity, Contract
binding, Runtime identity, and historical low-assurance cleanup uncertainty
remain deterministic and auditable.

## Boundary

The plan never invents a pull request, upgrades historical assurance, or
rewrites `.ai/` history. Missing or contradictory merge facts continue to
fail closed. The attached object repository remains an external read-only
acceptance boundary.

## Verification

The Runtime lifecycle evidence covers the plan-to-apply round trip, strict
negative cases, workspace tests, clippy, documentation acceptance, governance
integrity, and reference inventory checks. Terminal evidence is recorded in
the archive and Runtime-generated receipts for this Work Item.

[简体中文](WI-614-direct-merge-no-pr-apply.zh-CN.md) · [日本語](WI-614-direct-merge-no-pr-apply.ja.md)
