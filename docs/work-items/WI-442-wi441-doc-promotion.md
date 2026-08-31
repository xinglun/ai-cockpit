---
author: AI Cockpit maintainers
title: "WI-442 — WI-441 parity-ledger projection"
workItemId: WI-442-wi441-doc-promotion
description: "Promote the closed WI-441 terminal evidence into the three reference-parity ledgers."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-442-wi441-doc-promotion
terminalArchive: .ai/work-items/archive/WI-442-wi441-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-442-wi441-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-442-wi441-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-442-wi441-doc-promotion.close.json
---

# WI-442 — WI-441 parity-ledger projection

This Work Item promotes immutable WI-441 terminal paths into the three
reference-parity ledgers. It does not change Runtime behavior or rewrite
WI-441 evidence bytes.

[简体中文](WI-442-wi441-doc-promotion.zh-CN.md) · [日本語](WI-442-wi441-doc-promotion.ja.md)

## Scope

- Update the three `docs/reference/reference-parity.*.md` ledgers.
- Keep archive, verification, finalization, and close paths explicit.
- Preserve the local-only reference-source boundary.

## Verification boundary

Runtime verification is `cargo test --locked --workspace`. The projection is
accepted only when `python3 tests/docs/promote_closed_work_item.py --check-all`
reports a current ledger and the governance-integrity gate passes.
