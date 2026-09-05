---
author: AI Cockpit maintainers
title: "WI-593 — WI-592 documentation parity recovery"
description: "Redeliver the missing WI-592 parity registration without rewriting predecessor history."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-593-wi591-doc-parity-recovery
lastVerifiedBy: WI-593-wi591-doc-parity-recovery
---

[简体中文](WI-593-wi591-doc-parity-recovery.zh-CN.md) · [日本語](WI-593-wi591-doc-parity-recovery.ja.md)

# WI-593 — WI-592 documentation parity recovery

## Objective

Redeliver the missing three-language parity registration discovered by the
documentation governance gate after WI-592 was archived. Preserve WI-592's
immutable records and make the current documentation projection auditable.

## Boundary

The scope is limited to the three reference-parity files and the English,
Chinese, and Japanese WI-592/WI-593 documentation pages. Runtime behavior,
Cargo metadata, release artifacts, object repositories, global Agent/MCP
configuration, and historical `.ai` bytes are out of scope.

## Acceptance

1. All three parity files contain one consistent WI-592 recovery row and one
   pending WI-593 row with valid Work Item links and evidence boundaries.
2. WI-592's archived Contract, evidence, recovery decision, and event history
   remain byte-for-byte unchanged.
3. The three-language documentation pages have valid frontmatter and explain
   the append-only recovery relationship without claiming a false terminal state.
4. `python3 tests/docs/promote_closed_work_item.py --repo <repository>
   --check-all`, `tests/docs/documentation_acceptance.sh`, and
   `tests/docs/parity_status_check.sh` pass before terminal close.

## Verification

Run the documentation acceptance, parity, status-consistency, and locked
workspace checks declared by the current Contract with an explicit repository
context. Re-run the full documentation gate after the PR is merged.
