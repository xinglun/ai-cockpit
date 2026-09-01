---
author: AI Cockpit maintainers
title: "WI-481 — WI-480 terminal documentation promotion"
description: "Promote the WI-480 terminal documentation projections without rewriting immutable evidence."
audience: [maintainer, reviewer, adopter]
workItemId: WI-481-wi480-doc-promotion
status: in_progress
authority: authorized
lastVerifiedBy: WI-481-wi480-doc-promotion
---

# WI-481 — WI-480 terminal documentation promotion

This bounded Work Item promotes the verified and closed WI-480 lifecycle into
the tri-language Work Item and reference-parity projections. It does not alter
immutable Runtime evidence, archive records, or the reference inventory.

[简体中文](WI-481-wi480-doc-promotion.zh-CN.md) · [日本語](WI-481-wi480-doc-promotion.ja.md)

## Scope

- Promote the six WI-480 documentation projections using the repository helper.
- Keep the promotion deterministic and bound to the exact terminal records.
- Register this Work Item's own pages and parity row before archive.

## Out of scope

Runtime/Core implementation, release or adopter artifacts, reference-source
implementation parity beyond these projections, and immutable governance bytes.

## Acceptance

1. `promote_closed_work_item.py --repo <repo> --work-item WI-480-finalization-context-recovery --check` passes.
2. `promote_closed_work_item.py --repo <repo> --check-all` reports no stale projections after merge.
3. No Contract, Summary, Outcome, Evidence, Finalization, Close, Recovery, or reference-inventory bytes are rewritten.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-480-finalization-context-recovery --check`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/getting_started_semantic.sh`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/ci/governance_integrity_gate_test.sh`
- `python3 tests/ci/repository_gate_manifest_test.py`
- `cargo test --locked --workspace`
