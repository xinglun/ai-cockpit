---
author: AI Cockpit maintainers
title: "WI-524 — recovery successor readiness entry-gate binding"
description: "Prevent a repository-wide pending-close blocker from being suppressed by an unproven recovery successor."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-524-recovery-successor-readiness
lastVerifiedBy: WI-524-recovery-successor-readiness
---

[简体中文](WI-524-recovery-successor-readiness.zh-CN.md) · [日本語](WI-524-recovery-successor-readiness.ja.md)

## Goal

Bind repository readiness to a validated recovery successor. A predecessor may
leave the repository entry gate only when its successor is repository-bound,
manifest-verified, verified, and explicitly closed.

## Scope

- Validate recovery successor lineage before suppressing an archived predecessor's `pending close` blocker.
- Keep missing, stale, foreign, malformed, symlinked, or still-open successors fail-closed.
- Add repository isolation regression coverage and tri-lingual workflow/parity documentation.
- Keep historical evidence immutable and do not modify object repositories or global Agent/MCP configuration.

## Acceptance

- A valid closed terminal successor clears only its own predecessor's pending-close blocker.
- Invalid or incomplete successor records remain blockers.
- Parallel repositories remain isolated and existing lifecycle behavior stays intact.
- Rust tests, documentation acceptance, governance integrity, and hosted CI pass.
- No generated evidence or historical archive bytes are hand-edited.

## Verification

```text
cargo test --locked -p cockpit-repository --test lifecycle_entry --test recovery_decision -- --test-threads=1
cargo clippy --workspace --all-targets --all-features -- -D warnings
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/docs/work_item_status_consistency.py --repo <repo>
bash tests/docs/documentation_acceptance.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
git diff --check
```
