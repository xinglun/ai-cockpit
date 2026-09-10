---
author: AI Cockpit maintainers
title: "WI-771 — WI-770 terminal documentation promotion"
description: "Promote the verified closed WI-770 documentation projections to their terminal state."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-771-wi770-doc-promotion
lastVerifiedBy: WI-771-wi770-doc-promotion
---

[简体中文](WI-771-wi770-doc-promotion.zh-CN.md) · [日本語](WI-771-wi770-doc-promotion.ja.md)

# WI-771 — WI-770 terminal documentation promotion

## Intent

Promote the verified closed `WI-770-performance-terminal-docs-recovery`
documentation and reference-parity projections without changing its historical
evidence or the Runtime.

## Boundary

This Work Item changes only the six WI-770 documentation projections and its
own governance documentation. Runtime behavior, performance implementation,
release behavior, version metadata, and historical WI-770
archive/evidence/finalization/close bytes are out of scope.

## Verification

The closed Work Item promotion helper must pass `--check-all`; terminal links
are projected only from the verified WI-770 archive, evidence, finalization,
and close records.
