---
author: AI Cockpit maintainers
title: "WI-290 — Checkpoint conformance final delivery"
workItemId: WI-290-checkpoint-conformance-final
description: "Re-deliver the bounded checkpoint conformance batch from the latest remote default base, without rewriting predecessor bytes."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-290-checkpoint-conformance-final
authority: canonical
---

# WI-290 — Checkpoint conformance final delivery

## Purpose

WI-287, WI-288, and WI-289 are preserved as immutable recovery history after
hosted gates found invalid delivery bindings. This successor keeps the same
bounded implementation, starts from the latest remote default base, and binds
complete tri-language lifecycle evidence before verification.

## Boundary

- Preserve WI-287, WI-288, and WI-289 archives, evidence, recovery, and finalization bytes.
- Keep the Rust-native checkpoint and Agent-rule implementation unchanged.
- Register complete tri-language documentation/parity lifecycle paths before archive.
- Bind a new provider PR before verification, then complete hosted checks,
  finalization, close, and exact resource cleanup.

## Object/adopter parity

The same installed Runtime, explicit repository context, fail-closed lifecycle,
and visible human Outcome must govern both this repository and a fresh adopter.

## Verification

Declared verification: `cargo test --locked --workspace`, conformance inventory,
documentation and governance integrity gates, hosted PR checks, provider
finalization verification, close, and post-close status/doctor checks.
