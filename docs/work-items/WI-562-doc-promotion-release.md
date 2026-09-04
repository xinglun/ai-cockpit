---
author: AI Cockpit maintainers
title: "WI-562 — terminal documentation promotion for WI-561"
description: "Promote the closed WI-561 release documentation projections."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-562-doc-promotion-release
lastVerifiedBy: WI-562-doc-promotion-release
---

[简体中文](WI-562-doc-promotion-release.zh-CN.md) · [日本語](WI-562-doc-promotion-release.ja.md)

# WI-562 — terminal documentation promotion for WI-561

## Objective

Promote the exact WI-561 Work Item page and reference-parity projections after
its verified close, using only immutable terminal records.

## Scope and boundary

The scope is limited to the three WI-561 language pages, the three matching
reference-parity pages, and these three language pages for this bounded
self-projection. The closed-Work-Item promotion helper is the only writer of
terminal status. Runtime behavior, object repositories, global Agent/MCP
configuration, source inventory semantics, and unrelated documentation remain
unchanged.

## Acceptance

- All WI-561 projections carry terminal archive, verification, finalization, and
  close references without changing governance facts.
- This Work Item is registered in all three parity pages with its pre-archive
  status until its own verified close.
- The closed-Work-Item promotion check, documentation acceptance, parity gate,
  and declared verification commands pass.
- No immutable receipt or unrelated projection is changed.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`
