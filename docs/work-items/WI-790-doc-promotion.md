---
author: AI Cockpit maintainers
title: "WI-790 — documentation promotion for WI-785 and WI-787"
description: "Repair the bounded tri-language documentation projections after verified Runtime recovery and close."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-documentation-promotion
workItemId: WI-790-doc-promotion
lastVerifiedBy: WI-790-doc-promotion
---

[简体中文](WI-790-doc-promotion.zh-CN.md) · [日本語](WI-790-doc-promotion.ja.md)

# WI-790 — documentation promotion for WI-785 and WI-787

## Intent and boundary

WI-790 is a bounded documentation Work Item. It promotes the terminal WI-787
projection and repairs the WI-785 recovered parity projection using immutable
Runtime records. It does not rewrite archive, evidence, finalization, close,
or recovery bytes, and does not change product behavior, Runtime behavior, or
release state.

## Acceptance

- WI-787's three language pages bind its actual terminal archive, verification,
  finalization, and close paths.
- WI-785's three language pages and parity rows identify the immutable attempt
  as recovered and bind the actual hashed supersede decision and verification.
- WI-790's own three pages and parity rows are registered before archive.
- Documentation, parity, governance-integrity, and status-consistency checks
  pass without rewriting immutable records.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-787-parity-finalization-recovery`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `cargo test --locked --workspace`

