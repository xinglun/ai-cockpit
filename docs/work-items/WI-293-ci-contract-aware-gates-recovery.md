---
author: AI Cockpit maintainers
title: "WI-293 — CI Contract-aware quality gate recovery"
workItemId: WI-293-ci-contract-aware-gates-recovery
description: "Re-deliver the bounded CI Contract-aware gate from the latest remote default base with parity registered before verification."
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-294-lifecycle-recovery-state-machine
authority: canonical
---

# WI-293 — CI Contract-aware quality gate recovery

## Purpose

WI-293 is preserved as immutable recovered history. Its merged CI gate delivery
is recorded by PR #253, and the post-merge lifecycle recovery defect is owned by
the narrow successor WI-294; neither Work Item rewrites the predecessor bytes.

## Boundary

- Preserve WI-293 archive, evidence, blocked finalization, and recovery bytes.
- Keep Rust as the Contract gate authority while retaining Python/Cargo shadow
  checks; do not remove the existing CI policy in this batch.
- Bind the actual provider PR before final verification, then complete hosted
  checks, finalization, close, and exact branch/worktree cleanup.

## Object/adopter parity

The same installed Runtime, explicit `--repo` context, fail-closed evidence,
and visible human Outcome governed the merged delivery. WI-294 records the
recovery boundary discovered during closure.

## Verification

Declared verification: `cargo test --locked --workspace`, CI/conformance and
documentation gates, hosted PR checks, provider finalization verification,
close, and post-close status/doctor checks.
