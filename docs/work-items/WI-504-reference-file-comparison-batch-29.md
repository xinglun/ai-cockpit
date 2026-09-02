---
author: AI Cockpit maintainers
title: "WI-504 — reference documentation batch 29"
description: "Re-read five changed local reference documents and close any verified Rust reader-route omission without copying source implementation."
audience:
  - maintainer
  - reviewer
workItemId: WI-504-reference-file-comparison-batch-29
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-504-reference-file-comparison-batch-29
---

# WI-504 — reference documentation batch 29

[简体中文](WI-504-reference-file-comparison-batch-29.zh-CN.md) · [日本語](WI-504-reference-file-comparison-batch-29.ja.md)

## Goal

Compare the next five changed files in the pinned local reference checkout one
by one. Preserve portable governance meaning in the Rust-native reader routes,
and repair a concrete navigation omission when evidence shows one. This Work
Item does not copy source Python, Make, provider commands, source receipts, or
object/adopter repository state.

## Scope and file decisions

The reference commit is
`fde3380f81fea5fd2e288f7a8849f737dc074060`. Each path receives an explicit
inventory decision:

| Reference path | Decision | Rust boundary |
| --- | --- | --- |
| `docs/reference/repository-workflow.ja.md` | implemented-different-by-design | The Rust Japanese workflow already uses localized Runtime presentation without the removed `REPORT_LANGUAGE` argument and keeps explicit repository-scoped lifecycle and cleanup. |
| `docs/reference/troubleshooting.md` | implemented-different-by-design | The Rust tri-language troubleshooting route keeps general stop/recovery and evidence-preservation rules; provider handoff records remain external. |
| `docs/reference/verification-evidence-reuse.md` | implemented-different-by-design | The source no-change decision is specific to its Python/Make proposal; Rust's separately authorized reuse remains identity-bound and fail-closed. |
| `docs/reference/work-item-lifecycle-closure.md` | implemented-different-by-design | Rust-native closure, exact cleanup, and recovery routes retain the portable boundary; source hosted-governance/Make recovery details are not Runtime commands. |
| `docs/upgrade.md` | implemented-different-by-design | A minimal root compatibility pointer restores the reader route to the canonical Rust tri-language upgrade guide. |

## Acceptance

- All five paths are re-read at the pinned local commit and have non-deferred,
  evidence-backed inventory records with non-empty counterparts and reasons.
- The root `docs/upgrade.md` route exists and points to the canonical upgrade
  reference without duplicating source implementation or source claims.
- Tri-language comparison/parity documentation records the same five decisions;
  current counts are consistent and `migrate-gap` remains zero.
- No source implementation, provider configuration, global Agent/MCP setting,
  or object/adopter repository is changed.
- The declared conformance, documentation, Runtime verification, reviewed PR,
  merge, close, and exact cleanup checks pass.

## Verification

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

The ledger remains semantic/documentation parity, not source command, JSON-wire,
provider-state, or release-claim compatibility. The local reference checkout is
read through `AI_COCKPIT_REFERENCE_ROOT` and is never modified by this Work
Item.
