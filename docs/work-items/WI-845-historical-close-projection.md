---
author: AI Cockpit maintainers
title: "WI-845 — historical close projection"
description: "Project a valid append-only successor recovery without rewriting a noncanonical historical close."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-845-historical-close-projection
lastVerifiedBy: WI-845-historical-close-projection
---

[简体中文](WI-845-historical-close-projection.zh-CN.md) · [日本語](WI-845-historical-close-projection.ja.md)

# WI-845 — historical close projection

## Intent and boundary

This Work Item makes Runtime status projection recognize a valid append-only
successor recovery for an archived Work Item whose historical close decision
is noncanonical. The predecessor bytes remain immutable; the projection is
resolved only after the repository-bound recovery, successor, archive, and
close bindings are validated. Protocol schema, CLI surface, release and
adopter behavior, historical rewriting, and broad branch cleanup are outside
this boundary.

## Recovery boundary

An in-scope defect is amended and revalidated on this Work Item. A successor
is reserved for a genuinely different scope, authority, or base, an
independent change, an unsafe in-scope repair, immutable failed delivery, or
explicit human direction. Invalid, incomplete, foreign, stale, or tampered
recovery evidence remains blocking and never receives a normal closed
projection.

## Acceptance

- A valid repository-bound successor recovery with a terminal successor
  projects the predecessor as recovered and nonblocking without rewriting the
  predecessor close bytes.
- An unclosed, mismatched, malformed, foreign, or tampered successor recovery
  remains blocking and does not bypass close validation.
- Focused Rust regression tests prove the positive projection and fail-closed
  negative cases.
- The implementation performs no partial state writes and preserves the
  original historical close bytes.
- The PR contains the required pre-archive tri-language Work Item pages and
  exactly one parity row for this Work Item, so the repository quality gate
  can validate the change before expensive verification.

## Verification

- `cargo test --locked -p cockpit-repository --test status_projection`
- `cargo test --locked -p cockpit-repository --test recovery_decision`
- `cargo clippy --locked -p cockpit-repository --all-targets --all-features -- -D warnings`
- `cargo fmt --all -- --check`
- `git diff --check`
