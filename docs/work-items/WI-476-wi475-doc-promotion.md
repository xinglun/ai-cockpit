---
author: AI Cockpit maintainers
title: "WI-476 — WI-475 terminal documentation promotion"
description: "Promote the closed WI-475 evidence into reader-facing projections without rewriting immutable records."
audience: [maintainer, reviewer, adopter]
workItemId: WI-476-wi475-doc-promotion
status: active
authority: authorized
lastVerifiedBy: WI-476-wi475-doc-promotion
---

# WI-476 — WI-475 terminal documentation promotion

## Intent and boundary

This bounded Work Item promotes the verified and closed WI-475 lifecycle into
the tri-language Work Item and reference-parity projections. It does not alter
immutable Runtime evidence, the reference inventory, Runtime code, or any
object repository.

[简体中文](WI-476-wi475-doc-promotion.zh-CN.md) · [日本語](WI-476-wi475-doc-promotion.ja.md)

## Scope

- Bind WI-475 archive, verification, finalization, and close records in all
  three Work Item pages and parity ledgers.
- Register this Work Item's own in-progress tri-language pages and parity rows
  before archive, then promote them only after verified close.
- Keep the closed-Work-Item promotion check repeatable and preserve historical
  evidence bytes exactly.

## Out of scope

Runtime/Core implementation, reference inventory classifications, release or
adopter scripts, object repositories, and global Agent/MCP configuration.

## Acceptance

1. `promote_closed_work_item.py --repo <repo> --work-item WI-475-reference-file-comparison-batch-25 --check` passes.
2. All six WI-475 projection files bind the exact archive, verification,
   finalization, and close evidence paths.
3. This Work Item has tri-language pages and pre-archive parity rows, and its
   post-close promotion is deterministic.
4. No Contract, archive, verification, finalization, close, or reference
   inventory bytes are rewritten.
5. English, Simplified Chinese, and Japanese pages remain semantically
   equivalent while preserving authored Contract language.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-475-reference-file-comparison-batch-25 --check`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/getting_started_semantic.sh`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/ci/governance_integrity_gate_test.sh`
- `python3 tests/ci/repository_gate_manifest_test.py`
- `cargo test --locked --workspace`

The terminal fields for this page are promoted only after reviewed merge,
archive, finalization, and close complete.
