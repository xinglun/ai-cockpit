---
author: AI Cockpit maintainers
title: "WI-786 — recovered parity-registration repair"
description: "Repair the evidence-bound tri-language parity projection required by the governance gate."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-successor-recovery
workItemId: WI-786-parity-registration-repair
lastVerifiedBy: WI-786-parity-registration-repair
---

[简体中文](WI-786-parity-registration-repair.zh-CN.md) · [日本語](WI-786-parity-registration-repair.ja.md)

# WI-786 — recovered parity-registration repair

## Intent and boundary

WI-786 is the explicit successor for WI-785. The hosted governance gate
identified that recovered parity rows must contain the actual hashed recovery
decision path, and that the WI-784 row must bind its verification evidence.

This Work Item changes only the three reference-parity documents and its own
tri-language planning pages. WI-785, WI-784, and earlier archive, evidence,
Outcome, event, and decision bytes remain immutable. Runtime source, product
behavior, release state, and unrelated documentation are out of scope.

## Acceptance mapping

- The English, Simplified Chinese, and Japanese rows for WI-781, WI-782, and
  WI-784 bind the actual recovery decision and required evidence paths.
- Existing WI-783 terminal and WI-785 recovery facts remain semantically
  equivalent across the three languages.
- The WI-786 planning pages stay pre-close and are promoted only from Runtime
  terminal evidence after verified close.

## Verification

`bash tests/docs/parity_status_check.sh`

`bash tests/docs/documentation_acceptance.sh`

`cargo test --locked --workspace`

The predecessor binding is
`.ai/decisions/WI-785-wi784-doc-promotion.recovery.json`.
