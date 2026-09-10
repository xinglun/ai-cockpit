---
author: AI Cockpit maintainers
title: "WI-771 — WI-770 terminal documentation promotion"
description: "Promote the verified closed WI-770 documentation projections to their terminal state."
audience: [maintainer, reviewer, adopter]
status: recovered
authority: human:repository-owner
workItemId: WI-771-wi770-doc-promotion
lastVerifiedBy: WI-772-wi771-doc-promotion-recovery
recoveryDecision: .ai/decisions/WI-771-wi770-doc-promotion.recovery.json
---

[简体中文](WI-771-wi770-doc-promotion.zh-CN.md) · [日本語](WI-771-wi770-doc-promotion.ja.md)

# WI-771 — WI-770 terminal documentation promotion

## Intent

Promote the verified closed `WI-770-performance-terminal-docs-recovery`
documentation and reference-parity projections without changing its historical
evidence or the Runtime. The original WI-771 delivery was merged before its
Runtime finish/archive snapshot; its immutable recovery decision therefore
binds this successor revalidation.

## Boundary

This Work Item changes only the six WI-770 documentation projections, the
three-language WI-771 recovery projection, and its own governance records.
Runtime behavior, performance implementation, release behavior, version
metadata, and historical WI-770/WI-771 archive/evidence/finalization/close
bytes are out of scope.

## Verification

The closed Work Item promotion helper must pass `--check-all`; terminal links
are projected only from the verified WI-770 archive, evidence, finalization,
and close records. The recovered WI-771 projection must bind its immutable
archive, verification, and recovery decision while its successor owns fresh
verification and closure.

## Recovery state

WI-771 is an immutable recovered predecessor. PR #755 is merged, but it was
merged before WI-771 finish/archive, so the original verification snapshot is
not reused as current completion evidence. The append-only recovery decision
`.ai/decisions/WI-771-wi770-doc-promotion.recovery.json` assigns fresh
verification, finalization, and close to WI-772. No Runtime or performance
behavior is changed.
