---
author: AI Cockpit maintainers
title: "WI-334 — Evidence binding and reuse primitives"
workItemId: WI-334-evidence-binding-reuse
description: "Compare the pinned evidence-binding/reuse corpus and record the Rust semantic counterpart without copying Python/V1 wire."
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-334-evidence-binding-reuse
capabilityClaims:
  - reference_parity
  - evidence_reuse
---

# WI-334 — Evidence binding and reuse primitives

## Intent and boundary

This Work Item reads ten pinned source paths at
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` individually. The target already
has a Rust-native composite evidence model, so this batch records semantic
responsibility parity rather than copying source Python modules or JSON wire.

## File-by-file decision

All ten paths are `implemented-different-by-design`:

| Pinned source path | Rust counterpart | Decision |
| --- | --- | --- |
| `docs/reference/content-bound-evidence-reuse.md` | `crates/cockpit-evidence/src/lib.rs`; `tests/reuse.rs` | Content identity is one component of exact composite binding; reuse remains advisory. |
| `docs/reference/diff-bound-evidence-reuse.md` | `crates/cockpit-evidence/src/lib.rs`; `crates/cockpit-git/src/lib.rs` | Base/head and changed-path identity mismatch requires rerun. |
| `docs/reference/environment-bound-reuse.md` | `crates/cockpit-evidence/src/lib.rs`; `crates/cockpit-verification/src/lib.rs` | Explicit environment/toolchain/Runtime/profile identity is bound; process environment is not serialized wholesale. |
| `docs/reference/evidence-binding-foundation.md` | `crates/cockpit-evidence/src/lib.rs`; `crates/cockpit-repository/src/lib.rs` | Versioned receipt validation is strict and never bypasses governance or protected checks. |
| `scripts/ai_evidence_binding.py` | `crates/cockpit-evidence/src/lib.rs` | Typed structs and content-addressed receipt IDs replace the Python API. |
| `scripts/ai_diff_bound_reuse.py` | `crates/cockpit-evidence/src/lib.rs`; `crates/cockpit-git/src/lib.rs` | Typed diff identity replaces the Python helper. |
| `scripts/ai_environment_reuse.py` | `crates/cockpit-evidence/src/lib.rs`; `crates/cockpit-verification/src/lib.rs` | Explicit bounded inputs replace the source adapter; credentials are not read. |
| `tests/test_ai_evidence_binding.py` | `crates/cockpit-evidence/tests/reuse.rs`; `crates/cockpit-repository/tests/receipt_store.rs` | Strict schema, tamper, expiry, mismatch, failed/protected and rerun cases are covered natively. |
| `tests/test_ai_diff_bound_reuse.py` | `crates/cockpit-evidence/tests/reuse.rs`; `crates/cockpit-git/tests/repository.rs` | Clean/changed paths, canonical ordering, malformed paths and policy mismatch are covered. |
| `tests/test_ai_environment_reuse.py` | `crates/cockpit-evidence/tests/reuse.rs`; `crates/cockpit-verification/tests/execution.rs` | Environment/toolchain identity, stale/unknown receipts and protected execution are covered. |

The caller still owns governance, coverage, security and required-check gates.
Any result other than exact fresh reuse executes again. No source participant,
Python, Make or V1 artifact is introduced.

## Acceptance

- Inventory has exactly ten WI-334 records and none remain deferred or migrate-gap.
- The tri-language comparison and parity ledgers state the same semantic,
  non-wire boundary.
- Rust evidence/reuse tests and documentation/inventory checks pass.
- The installed Runtime produces bound verification evidence and the reviewed
  PR lifecycle is closed with exact branch/worktree cleanup.

[简体中文](WI-334-evidence-binding-reuse.zh-CN.md) · [日本語](WI-334-evidence-binding-reuse.ja.md)
