---
author: AI Cockpit maintainers
title: "WI-576 — WI-575 documentation-promotion retry"
description: "Re-deliver the WI-574 terminal documentation projection with a provable lifecycle order."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-576-wi575-doc-promotion-retry
lastVerifiedBy: WI-576-wi575-doc-promotion-retry
terminalArchive: .ai/work-items/archive/WI-576-wi575-doc-promotion-retry.contract.json
terminalVerification: .ai/evidence/WI-576-wi575-doc-promotion-retry.verification.json
terminalFinalization: .ai/decisions/WI-576-wi575-doc-promotion-retry.finalize.ff9b14cb37866d6e475e2dfc72c705bd289d494ae54790b2b5625c5292a94d42.json
terminalDecision: .ai/decisions/WI-576-wi575-doc-promotion-retry.close.json
---

[简体中文](WI-576-wi575-doc-promotion-retry.zh-CN.md) · [日本語](WI-576-wi575-doc-promotion-retry.ja.md)

# WI-576 — WI-575 documentation-promotion retry

## Objective

Re-deliver the WI-574 terminal documentation projection after PR #556 was
closed as an immutable failed delivery. The successor preserves that failure
as provider audit history and fixes only the lifecycle ordering: register the
tri-language parity row before archive, then review, merge, close, and promote.

## Scope and boundary

- Promote the WI-574 work-item pages and their three reference-parity rows.
- Maintain this successor's three-language pages and parity registration.
- Keep the closed PR #556 failure as external audit history; do not claim it
  merged and do not rewrite any WI-575 bytes.

Runtime behavior, the object repository, global Agent/MCP configuration,
reference-source implementation copying, release publication, and historical
governance bytes are outside this Work Item.

## Acceptance

1. The parity registration for WI-576 is committed before archive and remains
   `In progress` until verified close.
2. WI-574 pages are marked implemented only from validated terminal evidence.
3. The three parity ledgers contain exact terminal paths after close.
4. Documentation, governance, status, workspace, and diff checks pass.
5. No WI-575 or other historical governance record is rewritten.

## Verification

- `tests/docs/documentation_acceptance.sh`
- `tests/docs/parity_status_check.sh`
- `tests/docs/pending_parity_registry_test.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `cargo test --locked --workspace`
- `git diff --check`
