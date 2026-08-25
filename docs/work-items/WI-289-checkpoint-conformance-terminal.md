---
author: AI Cockpit maintainers
title: "WI-289 — Checkpoint conformance terminal recovery"
workItemId: WI-289-checkpoint-conformance-terminal
description: "Re-deliver the bounded checkpoint conformance batch after hosted documentation-truth rejection, without rewriting predecessor bytes."
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-289-checkpoint-conformance-terminal
authority: canonical
---

# WI-289 — Checkpoint conformance terminal recovery

## Purpose

WI-288 is preserved as immutable recovery history because hosted quality found
the recovered WI-287 documentation status still marked `in_progress` after
archive. This successor keeps the same bounded implementation and binds the
corrected tri-language documentation state before verification.

## Boundary

- Preserve WI-287 and WI-288 archives, evidence, recovery, and finalization bytes.
- Keep the Rust-native checkpoint and Agent-rule implementation unchanged.
- Correct the tri-language documentation/parity projection before archive.
- Bind a new provider PR before verification, then complete hosted checks,
  finalization, close, and exact resource cleanup.

## Object/adopter parity

The same installed Runtime, explicit repository context, fail-closed lifecycle,
and visible human Outcome must govern both this repository and a fresh adopter.

## Verification

Declared verification: `cargo test --locked --workspace`, conformance inventory,
documentation and governance integrity gates, hosted PR checks, provider
finalization verification, close, and post-close status/doctor checks.
