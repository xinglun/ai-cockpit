---
author: AI Cockpit maintainers
title: "WI-839 — Runtime documentation projection preflight"
description: "Reject missing Work Item documentation projections before expensive verification and terminal close."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-839-runtime-doc-projection-preflight
lastVerifiedBy: WI-839-runtime-doc-projection-preflight
terminalArchive: .ai/work-items/archive/WI-839-runtime-doc-projection-preflight.contract.json
terminalVerification: .ai/evidence/WI-839-runtime-doc-projection-preflight.verification.json
terminalDecision: .ai/decisions/WI-839-runtime-doc-projection-preflight.close.json
---

[简体中文](WI-839-runtime-doc-projection-preflight.zh-CN.md) · [日本語](WI-839-runtime-doc-projection-preflight.ja.md)

# WI-839 — Runtime documentation projection preflight

## Intent and boundary

This Work Item moves the repository's tri-language Work Item projection
requirement to the earliest safe lifecycle boundaries. It covers Runtime
preflight, verification entry, close protection, regression tests, and the
operator command reference. Product behavior, release artifacts, historical
record rewriting, and unrelated Work Items are outside the boundary.

## Acceptance

- A repository using the tri-language parity convention rejects a new Work
  Item with a missing or malformed own page or parity row before verification.
- The rejection names the exact path and reason and does not write terminal
  lifecycle state.
- A repository without that convention keeps generic Work Item behavior.
- The command reference states that same-root failures use amend/revalidate
  and retry; a successor requires an explicitly different boundary and
  predecessor binding.

## Verification

- `cargo test -p cockpit-repository --test lifecycle_entry`
- `cargo test -p cockpit-repository --test agent_rule_parity`
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `git diff --check`
