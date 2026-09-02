---
author: AI Cockpit maintainers
title: "WI-517 — WI-516 terminal documentation promotion"
description: "Promote the closed WI-516 reader-facing documentation projections without rewriting immutable governance records."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-517-wi516-doc-promotion
lastVerifiedBy: WI-517-wi516-doc-promotion
---

[简体中文](WI-517-wi516-doc-promotion.zh-CN.md) · [日本語](WI-517-wi516-doc-promotion.ja.md)

## Goal

Promote the closed WI-516 Work Item and parity projections from conditional
registration to terminal evidence-backed status. The helper is deterministic
and must not rewrite WI-516 Contract, Summary, Outcome, Events, verification,
finalization, or close bytes.

## Scope

- `docs/work-items/WI-516-reference-file-comparison-batch-34.md`
- `docs/work-items/WI-516-reference-file-comparison-batch-34.zh-CN.md`
- `docs/work-items/WI-516-reference-file-comparison-batch-34.ja.md`
- `docs/reference/reference-parity.md`
- `docs/reference/reference-parity.zh-CN.md`
- `docs/reference/reference-parity.ja.md`
- These three-language WI-517 reader records.

## Acceptance

- `promote_closed_work_item.py --repo <repo> --check-all` reports no stale
  WI-516 projection.
- WI-516 pages have `status: implemented` and exact terminal evidence paths;
  parity rows are `Implemented` and link the archived Contract, verification,
  finalization, and close records.
- Documentation, parity, and Work Item status-consistency checks pass.
- Immutable WI-516 Runtime-generated records remain byte-identical.
- No Runtime source, reference-source implementation, object/adopter
  repository, release publication, or global Agent/MCP setting changes.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `AI_COCKPIT_REFERENCE_ROOT=/Users/sei-rinn/dev/workspace_python/ai-cockpit-template bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

This is a post-close documentation projection only; generated receipts remain
immutable.
