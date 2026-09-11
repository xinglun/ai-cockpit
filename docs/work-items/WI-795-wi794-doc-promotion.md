---
author: AI Cockpit maintainers
title: "WI-795 — WI-794 closed documentation promotion"
description: "Promote the closed WI-794 Runtime evidence into a bounded, tri-language documentation projection."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-documentation-promotion
workItemId: WI-795-wi794-doc-promotion
lastVerifiedBy: WI-795-wi794-doc-promotion
---

[简体中文](WI-795-wi794-doc-promotion.zh-CN.md) · [日本語](WI-795-wi794-doc-promotion.ja.md)

# WI-795 — WI-794 closed documentation promotion

## Intent and boundary

WI-795 is the bounded post-close documentation Work Item for WI-794. It
projects the immutable WI-794 Contract, verification, finalization, and close
records into the English, Simplified Chinese, and Japanese Work Item pages and
reference-parity tables.

It does not change Runtime source, product behavior, release state, governance
rules, or immutable WI-794 records.

## Scope

- Promote the WI-794 terminal documentation projection in three languages.
- Keep this Work Item's own three planning pages and parity rows registered
  before close so the projection remains bounded after close.
- Keep the promotion helper and documentation acceptance checks reproducible.

## Acceptance

- WI-794's three language pages bind its actual terminal archive, verification,
  finalization, and close paths.
- WI-795's own three language pages and parity rows are registered before
  archive and are promoted only after verified close.
- Documentation, parity, governance-integrity, and status-consistency checks
  pass without rewriting immutable records.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-794-release-v0-2-90-closure --check`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
