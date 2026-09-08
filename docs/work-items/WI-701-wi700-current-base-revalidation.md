---
author: AI Cockpit maintainers
title: "WI-701 — WI-700 current-base revalidation"
description: "Revalidate the observation-context delivery from the current remote default base."
audience: [contributor, maintainer, reviewer]
workItemId: WI-701-wi700-current-base-revalidation
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-701-wi700-current-base-revalidation
---

[简体中文](WI-701-wi700-current-base-revalidation.zh-CN.md) · [日本語](WI-701-wi700-current-base-revalidation.ja.md)

# WI-701 — WI-700 current-base revalidation

WI-701 is the fresh recovery successor for WI-700. It binds the preserved
observation-context implementation and predecessor lineage to the current
remote default revision, then runs fresh verification and the normal hosted and
Runtime terminal lifecycle.

## Boundary

This Work Item does not rewrite predecessor archives, evidence, or recovery
decisions. It adds no new code semantics, governance rules, protocol formats,
or changes to another agent's Work Item. The three-language pages and parity
rows are projections of the same recovery boundary.

## Acceptance

- The current default base and predecessor digests are explicit.
- Required observation-context scenarios and the locked workspace gate pass.
- Hosted quality, provider finalization, archive, close, and exact cleanup are
  evidenced before the Work Item is declared terminal.
