---
author: AI Cockpit maintainers
title: "WI-517 — WI-516 terminal documentation promotion"
description: "immutable な governance record を書き換えず、closed WI-516 の reader-facing projection を昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-517-wi516-doc-promotion
lastVerifiedBy: WI-517-wi516-doc-promotion
---

[English](WI-517-wi516-doc-promotion.md) · [简体中文](WI-517-wi516-doc-promotion.zh-CN.md)

## Goal

closed WI-516 の Work Item と parity projection を conditional registration
から terminal な evidence-backed status に昇格します。helper は決定的に
動作し、WI-516 の Contract、Summary、Outcome、Events、verification、
finalization、close bytes を書き換えません。

## Scope

- `docs/work-items/WI-516-reference-file-comparison-batch-34.md`
- `docs/work-items/WI-516-reference-file-comparison-batch-34.zh-CN.md`
- `docs/work-items/WI-516-reference-file-comparison-batch-34.ja.md`
- `docs/reference/reference-parity.md`
- `docs/reference/reference-parity.zh-CN.md`
- `docs/reference/reference-parity.ja.md`
- WI-517 の三言語 reader record。

## Acceptance

- `promote_closed_work_item.py --repo <repo> --check-all` が WI-516 の stale
  projection を報告しない。
- WI-516 pages が `status: implemented` と正確な terminal evidence path を
  持ち、parity row が `Implemented` と archive Contract、verification、
  finalization、close record を参照する。
- documentation、parity、Work Item status consistency check が成功する。
- WI-516 の Runtime-generated record は byte-identical のまま保持する。
- Runtime source、reference source implementation、object/adopter repository、
  release publication、global Agent/MCP setting は変更しない。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `AI_COCKPIT_REFERENCE_ROOT=/Users/sei-rinn/dev/workspace_python/ai-cockpit-template bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

これは post-close の documentation projection のみを扱い、generated receipt
は immutable のままです。
