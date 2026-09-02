---
author: AI Cockpit maintainers
title: "WI-502 — terminal documentation projection for WI-501"
description: "Promote WI-501's closed documentation and parity projections after the post-close gate identified a stale conditional status."
audience: [maintainer, reviewer, adopter]
workItemId: WI-502-doc-promotion-terminal
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-502-doc-promotion-terminal
---

# WI-502 — terminal documentation projection for WI-501

[简体中文](WI-502-doc-promotion-terminal.zh-CN.md) · [日本語](WI-502-doc-promotion-terminal.ja.md)

## Boundary

This narrow documentation Work Item consumes the mandatory post-close
promotion finding for WI-501. It updates only the six tri-language Work Item
and parity projections so a closed Work Item is represented as terminal in the
reader-facing baseline. It does not rewrite Runtime-generated evidence or
change Runtime behavior.

## Scope

- Promote the three WI-501 documentation pages to their evidence-backed
  terminal status.
- Promote the three reference parity rows from conditional to `Implemented`.
- Re-run the repository documentation and status-consistency gates after the
  projection is updated.

## Out of scope

Runtime source, tests, object/adopter repositories, reference-source
implementation, release publication, global Agent/MCP configuration, and
historical evidence or archive rewriting.

## Acceptance

- WI-501's English, Simplified Chinese, and Japanese pages contain terminal
  evidence paths and `status: implemented`.
- WI-501's three parity rows are `Implemented` and link to the exact terminal
  records.
- `promote_closed_work_item.py --repo <repo> --check-all` passes.
- Documentation, parity, and Work Item status-consistency checks pass.
- No generated evidence or historical bytes are edited or deleted.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

The helper is the source of the terminal projection; generated receipts remain
immutable.
