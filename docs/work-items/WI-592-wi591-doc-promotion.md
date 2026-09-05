---
author: AI Cockpit maintainers
title: "WI-592 — WI-591 documentation promotion"
description: "Promote the WI-591 release projections after verification and preserve the later parity recovery as immutable history."
audience: [maintainer, reviewer, adopter]
status: recovered
authority: canonical
workItemId: WI-592-wi591-doc-promotion
lastVerifiedBy: WI-592-wi591-doc-promotion
---

[简体中文](WI-592-wi591-doc-promotion.zh-CN.md) · [日本語](WI-592-wi591-doc-promotion.ja.md)

# WI-592 — WI-591 documentation promotion

## Objective

Promote the three-language WI-591 release and reference-parity projections
after its immutable archive and verification evidence were recorded. The CI
discovery that WI-592 itself lacked a parity registration is preserved as
immutable history and is redelivered by successor WI-593.

## Boundary

This record is a documentation projection only. Runtime behavior, release
artifacts, object repositories, global Agent/MCP configuration, and generated
archive/evidence/decision bytes are outside the boundary and remain unchanged.

## Acceptance

1. WI-591 release documentation is promoted consistently in English, Chinese,
   and Japanese from its terminal Runtime evidence.
2. The recovery boundary and successor WI-593 are recorded without rewriting
   the archived WI-592 bytes.
3. The documentation gate remains reproducible and reports any missing parity
   registration as a bounded successor task rather than silently changing history.

## Verification

Run `python3 tests/docs/promote_closed_work_item.py --repo <repository>
--check-all`, `tests/docs/documentation_acceptance.sh`, and
`tests/docs/parity_status_check.sh` with the explicit repository context.
