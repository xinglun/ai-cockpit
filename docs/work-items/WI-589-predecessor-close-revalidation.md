---
author: AI Cockpit maintainers
title: "WI-589 — Predecessor close after Contract-amendment revalidation"
description: "Close-bound historical projection for an older provider finalization receipt after a successor revalidates an amended Contract."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-589-predecessor-close-revalidation
lastVerifiedBy: WI-589-predecessor-close-revalidation
---

[简体中文](WI-589-predecessor-close-revalidation.zh-CN.md) · [日本語](WI-589-predecessor-close-revalidation.ja.md)

# WI-589 — Predecessor close after Contract-amendment revalidation

## Objective

Allow an archived predecessor to close after a reviewed Contract amendment
has been revalidated by a terminal successor. The older provider finalization
receipt remains historical evidence: its bytes, path, digest, and sequence are
preserved and it is never relabeled as `direct_merge_no_pr`.

## Boundary

The compatibility lane is narrow and append-only. A successor must have
current verification, provider finalization, and an explicit human close. Only
then may the predecessor close bind the exact old finalization head as
`historical_low` revalidation. Missing, malformed, foreign, stale, or
contradictory lineage remains fail-closed. Direct-merge schema, adopter
scripts, object repositories, and reference-source implementation are outside
this Work Item.

## Acceptance

1. An old-runtime PR receipt can be projected as historical only after its
   Contract-amendment successor is terminal and repository-bound.
2. The predecessor close records the exact finalization path, digest, and
   sequence and preserves the original receipt bytes.
3. The close record distinguishes current successor revalidation from
   historical provider evidence and never invents a PR or direct-merge class.
4. Unresolved or tampered successor, archive, Contract, evidence, or receipt
   bindings fail closed without a partial close record.

## Verification

Run the focused recovery regression and the complete locked workspace suite.
The tri-language command reference documents the supported recovery path and
its fail-closed boundary.
