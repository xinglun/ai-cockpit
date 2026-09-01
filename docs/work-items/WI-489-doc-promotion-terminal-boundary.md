---
author: AI Cockpit maintainers
title: "WI-489 — bounded terminal documentation promotion"
description: "Prevent closed documentation-promotion Work Items from creating an unbounded successor chain."
audience: [maintainer, reviewer, adopter]
workItemId: WI-489-doc-promotion-terminal-boundary
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-489-doc-promotion-terminal-boundary
---

# WI-489 — bounded terminal documentation promotion

This Work Item makes the terminal documentation projection explicit and
bounded. It keeps ordinary, malformed, or mixed scopes fail-closed while
allowing a documentation-promotion Work Item to remain terminal without
creating an endless successor solely for its own pages.

[简体中文](WI-489-doc-promotion-terminal-boundary.zh-CN.md) · [日本語](WI-489-doc-promotion-terminal-boundary.ja.md)

## Scope

- Add a validated self-terminal boundary to the documentation promotion
  helper and the tri-language status consistency checker.
- Add regression fixtures for ordinary promotion, malformed scopes, and the
  bounded terminal projection.
- Document the boundary in the English, Chinese, and Japanese workflows.

## Acceptance

- The boundary is derived from an exact documentation-only scope and cannot
  hide arbitrary drift or wildcard paths.
- Normal Work Items still require evidence-backed terminal promotion.
- All checks remain deterministic and do not rewrite immutable governance
  records or global Agent/MCP configuration.

## Verification

- `bash tests/docs/promote_closed_work_item_test.sh`
- `bash tests/docs/work_item_status_consistency_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `python3 tests/conformance/reference_file_inventory.py --check`
