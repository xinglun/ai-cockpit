---
author: AI Cockpit maintainers
title: "WI-560 — terminal documentation promotion for WI-559"
description: "Promote the closed WI-559 documentation projections and register this bounded self-projection."
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: canonical
workItemId: WI-560-doc-promotion-wi559
lastVerifiedBy: WI-560-doc-promotion-wi559
terminalArchive: .ai/work-items/archive/WI-560-doc-promotion-wi559.contract.json
terminalVerification: .ai/evidence/WI-560-doc-promotion-wi559.verification.json
terminalFinalization: .ai/decisions/WI-560-doc-promotion-wi559.finalize.json
terminalDecision: .ai/decisions/WI-560-doc-promotion-wi559.close.json
---

[简体中文](WI-560-doc-promotion-wi559.zh-CN.md) · [日本語](WI-560-doc-promotion-wi559.ja.md)

# WI-560 — terminal documentation promotion for WI-559

## Objective

Promote the exact WI-559 Work Item pages and reference-parity projections from
their post-close check findings, using only the immutable terminal records.

## Scope and boundary

The scope is limited to the three WI-559 language pages, the three matching
reference-parity pages, and these three language pages for this bounded
self-projection. The promotion helper is the only writer of terminal status.
Runtime behavior, object repositories, global Agent/MCP configuration, source
inventory semantics, and unrelated documentation remain unchanged.

## Acceptance

- All WI-559 projections carry terminal archive, verification, finalization,
  and close references without changing governance facts.
- This Work Item is registered in all three parity pages with the required
  pre-archive status until its own verified close.
- The closed-Work-Item promotion check, documentation acceptance, parity gate,
  and declared verification commands pass.
- No immutable receipt or unrelated projection is changed.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`
