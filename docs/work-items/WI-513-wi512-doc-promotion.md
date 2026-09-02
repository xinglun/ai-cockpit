---
author: AI Cockpit maintainers
title: "WI-513 — WI-512 terminal documentation promotion"
description: "Promote the closed WI-512 projection without rewriting immutable governance records."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-513-wi512-doc-promotion
lastVerifiedBy: WI-513-wi512-doc-promotion
---

[简体中文](WI-513-wi512-doc-promotion.zh-CN.md) · [日本語](WI-513-wi512-doc-promotion.ja.md)

## Goal

Promote the WI-512 parity projection from its pre-archive registration to a
terminal `Implemented` row after the closed evidence exists. The helper is
deterministic and must not rewrite WI-512 Contract, Summary, Outcome, Events,
verification, finalization, or close bytes.

## Scope

- `docs/reference/reference-parity.md`
- `docs/reference/reference-parity.zh-CN.md`
- `docs/reference/reference-parity.ja.md`
- These three-language WI-513 reader records.

## Acceptance

- `promote_closed_work_item.py --check-all` reports no stale WI-512 projection.
- Documentation, parity, and governance-integrity checks pass.
- The immutable WI-512 generated records remain byte-identical.
- No Runtime, source-reference, object repository, or global Agent/MCP setting changes.

## Boundary

This is a post-close documentation projection only. It does not change
governance facts, create a new approval, or copy reference implementation.
