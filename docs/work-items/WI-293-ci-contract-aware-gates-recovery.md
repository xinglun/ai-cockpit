---
author: AI Cockpit maintainers
title: "WI-293 — CI Contract-aware quality gate recovery"
workItemId: WI-293-ci-contract-aware-gates-recovery
description: "Re-deliver the bounded CI Contract-aware gate from the latest remote default base with parity registered before verification."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-293-ci-contract-aware-gates-recovery
authority: canonical
---

# WI-293 — CI Contract-aware quality gate recovery

## Purpose

WI-291 is preserved as immutable recovery history after hosted quality rejected
its late parity projection. This successor delivers the same bounded Rust gate
from the latest remote default branch and registers all tri-language parity and
Work Item documentation before verification evidence is created.

## Boundary

- Preserve WI-291 archive, evidence, blocked finalization, and recovery bytes.
- Keep Rust as the Contract gate authority while retaining Python/Cargo shadow
  checks; do not remove the existing CI policy in this batch.
- Bind the actual provider PR before final verification, then complete hosted
  checks, finalization, close, and exact branch/worktree cleanup.

## Object/adopter parity

The same installed Runtime, explicit `--repo` context, fail-closed evidence,
and visible human Outcome must govern this repository and a fresh adopter.

## Verification

Declared verification: `cargo test --locked --workspace`, CI/conformance and
documentation gates, hosted PR checks, provider finalization verification,
close, and post-close status/doctor checks.

