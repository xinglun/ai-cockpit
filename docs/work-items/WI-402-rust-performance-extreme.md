---
author: AI Cockpit maintainers
title: WI-402 — Rust Runtime performance extreme
description: Measure and reduce avoidable Rust Runtime cost without weakening governance truth.
workItemId: WI-402-rust-performance-extreme
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-402-rust-performance-extreme
terminalArchive: .ai/work-items/archive/WI-402-rust-performance-extreme.contract.json
terminalVerification: .ai/evidence/WI-402-rust-performance-extreme.verification.json
terminalFinalization: .ai/decisions/WI-402-rust-performance-extreme.finalize.json
terminalDecision: .ai/decisions/WI-402-rust-performance-extreme.close.json
---

# WI-402 — Rust Runtime performance extreme

This Work Item optimizes the shared Rust Runtime for both the Cockpit
repository and attached adopter repositories. It is a measured optimization,
not a change to governance semantics: verification strength, evidence
identity, fail-closed behavior, request-scoped repository context, and
deterministic human Outcome remain authoritative.

## Delivered boundary

- Exact verification reuse ignores shell/mise/Agent session bookkeeping but
  retains command and toolchain inputs such as `PATH`, `PWD`, `TMPDIR`,
  `CARGO_HOME`, and `RUSTFLAGS`.
- Source content identity excludes Runtime-generated `.ai/` receipts while
  retaining tracked source and non-`.ai` working-tree changes. A governance
  receipt therefore cannot invalidate its own reusable result; a source change
  still does.
- Reuse is only profile-authorized and identity-bound. Explicit custom
  commands remain fresh, and any mismatch executes the declared check.
- Regression tests cover session metadata, source-only content identity, and
  an exact first-run/second-run receipt reuse path.

## Object-project inheritance

The optimization is in the shared external binary, not copied into an adopter.
Each repository keeps its own `.ai/` evidence and receives the same rules only
after an upgrade to the published Runtime. Runtime version/digest and
repository identity remain part of every verification context.

## Verification

The Work Item evidence records targeted Rust tests, full workspace quality,
and release/adopter acceptance. Timing is advisory evidence; it cannot lower a
required Verification Tier or Evidence Assurance.

### Local measurement (advisory)

On 2026-08-29, macOS arm64, a ten-iteration run against the same tiny attached
repository compared the installed v0.2.40 binary with the candidate release
profile. Warm P95 elapsed time changed as follows: `inspect` 72.561 ms →
72.217 ms (-0.5%), `status` 95.573 ms → 94.500 ms (-1.1%), `doctor` 16.636 ms
→ 13.828 ms (-16.9%), and `observe` 73.057 ms → 71.957 ms (-1.5%). These are
local process-latency observations, not provider or enterprise guarantees;
the candidate was measured outside the repository and was not a public-release
acceptance artifact.
