---
author: AI Cockpit maintainers
title: "WI-482 — lifecycle, parallel, and trust-layer reference comparison"
description: "Re-read eight changed local reference paths and record explicit Rust-native parity decisions."
audience:
  - maintainer
  - reviewer
workItemId: WI-482-reference-file-comparison-batch-26
status: current
authority: canonical
lastVerifiedBy: WI-482-reference-file-comparison-batch-26
---

# WI-482 — lifecycle, parallel, and trust-layer reference comparison

## Goal

Compare the eight reference files changed at local reference commit
`fde3380f81fea5fd2e288f7a8849f737dc074060` after the previous batch, one file at
a time. Preserve the useful governance semantics in Rust-native routes without
copying the reference Python runtime, Make workflows, provider configuration,
or source-only documentation layout.

## Boundaries

The comparison is bound to Rust baseline `1f65a3b8bf09e54d4f9600fc5d64d8bbcb3ed62f`
and the published `ai-cockpit 0.2.57` binary (SHA256
`f03a13251a6fe57783528efbeae6ddd23bc2cc31dd2a1501d5421aac169a1d58`). The
object/adopter repositories, Runtime feature work, and global Agent/MCP
configuration are out of scope.

## File decisions

| Reference path | Decision | Rust-native counterpart |
| --- | --- | --- |
| `docs/operations/work-item-lifecycle.md` | implemented-different-by-design | `docs/reference/agent-workflow.md`, `docs/reference/outcome-report.md` |
| `docs/operations/work-item-lifecycle.zh-CN.md` | implemented-different-by-design | Chinese Agent workflow and Outcome routes |
| `docs/operations/work-item-lifecycle.ja.md` | implemented-different-by-design | Japanese Agent workflow and Outcome routes |
| `docs/reference/agent-parallel-work-items.md` | implemented-different-by-design | `docs/reference/cross-work-item-dedup.md`, `docs/reference/affected-verification.md`, `docs/reference/agent-workflow.md`, `AGENTS.md`, `.ai/README.md` |
| `docs/reference/ai-cockpit-work-item-lifecycle.md` | implemented-different-by-design | `docs/reference/agent-workflow.md`, `docs/reference/outcome-report.md`, `docs/reference/ci-quality-gates.md`, Runtime lifecycle |
| `docs/trust-layer.md` | implemented-different-by-design | `docs/philosophy.md`, `docs/security/enterprise-governance.md`, `docs/architecture.md`, `docs/capabilities.md` |
| `docs/trust-layer.zh-CN.md` | implemented-different-by-design | Chinese philosophy, enterprise-governance, architecture, and capabilities routes |
| `docs/trust-layer.ja.md` | implemented-different-by-design | Japanese philosophy, enterprise-governance, architecture, and capabilities routes |

The source changes narrow a short lifecycle page, move parallel/handoff detail
to a dedicated reference, remove a template-only quality-shard section, and
remove an obsolete `REPORT_LANGUAGE` argument. These are layout or source
workflow differences, not omissions in the Rust Runtime. Contract facts remain
in their authored language; localized presentation cannot alter governance
facts or create a human decision.

## Acceptance and verification

- The inventory records exactly these eight paths as
  `implemented-different-by-design`, preserving source-change provenance and
  previous classifications.
- The three comparison routes and three parity ledgers name every path and the
  no-copy boundary.
- `python3 tests/conformance/reference_file_inventory.py --check`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `cargo test --locked --workspace`
