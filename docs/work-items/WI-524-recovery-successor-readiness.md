---
author: AI Cockpit maintainers
title: "WI-524 — recovery successor readiness entry-gate binding"
description: "Require a validated recovery successor before suppressing an archived predecessor blocker."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-524-recovery-successor-readiness
lastVerifiedBy: WI-524-recovery-successor-readiness
terminalArchive: .ai/work-items/archive/WI-524-recovery-successor-readiness.contract.json
terminalVerification: .ai/evidence/WI-524-recovery-successor-readiness.verification.json
terminalFinalization: .ai/decisions/WI-524-recovery-successor-readiness.finalize.cab9a20e63481aea75e8801ff86a94cec5ddc4c99fe9602500b43537567272c6.json
terminalDecision: .ai/decisions/WI-524-recovery-successor-readiness.close.json
---

[简体中文](WI-524-recovery-successor-readiness.zh-CN.md) · [日本語](WI-524-recovery-successor-readiness.ja.md)

## Goal

Bind repository readiness to a recovery successor that is repository-bound,
manifest-verified, verified, and explicitly closed.

## Scope

- Validate recovery successor lineage before suppressing an archived predecessor's `pending close` blocker.
- Keep missing, stale, foreign, malformed, symlinked, or open successors fail-closed.
- Add isolation regression coverage and tri-lingual workflow/parity projections.
- Preserve historical evidence and do not modify object repositories or global configuration.

## Acceptance

- Only a valid closed terminal successor clears its own predecessor blocker.
- Invalid or incomplete successors remain blockers; repository isolation remains intact.
- Rust tests, documentation/governance checks, and hosted CI pass.

## Verification

```text
cargo test --locked -p cockpit-repository --test lifecycle_entry --test recovery_decision -- --test-threads=1
cargo clippy --workspace --all-targets --all-features -- -D warnings
python3 tests/ci/governance_integrity_gate.py --repo <repo>
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
```
