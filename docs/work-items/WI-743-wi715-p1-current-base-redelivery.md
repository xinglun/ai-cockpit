---
author: AI Cockpit maintainers
title: "WI-743 — WI-715 current-base performance decision redelivery"
description: "Revalidate the declined WI-715 large-history candidate from the latest default base without reviving its stale unmerged PR."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-743-wi715-p1-current-base-redelivery
lastVerifiedBy: WI-743-wi715-p1-current-base-redelivery
---

[简体中文](WI-743-wi715-p1-current-base-redelivery.zh-CN.md) · [日本語](WI-743-wi715-p1-current-base-redelivery.ja.md)

# WI-743 — WI-715 current-base performance decision redelivery

## Intent

Redeliver the WI-715 large-history status candidate decision from the latest
remote default base. The purpose is to preserve the evidence-backed decline,
its limitations, and the no-production-change boundary in a fresh, reviewable
Work Item; it is not to revive PR #708.

## Lineage and boundary

- Current base: `origin/main` at `838ae745511942cb55dd7ac30c319cc5bc95e74d`.
- Predecessor: `WI-715-p1-large-history-status`, whose archive, evidence,
  decisions, and PR #708 remain immutable audit history on the predecessor
  branch: <https://github.com/xinglun/ai-cockpit/pull/708>.
- This Work Item covers current-base verification, evidence binding, and the
  tri-language governance record. It does not change Runtime behavior,
  performance thresholds, measurement semantics, Outcome/schema behavior,
  exit codes, authorization, persistence, or another agent's work.

## Evidence-backed decision

The predecessor's current-base experiment was run at
`d1141480fb7a045979098480c3770d06002e2a87` on the same Runtime line. In the
forward order, warm status p50 changed by `-1.171%`; in the reverse order it
changed by `-0.302%`. The registered threshold is `5%` in both directions, so
the candidate remains declined. Both gates also failed closed because the
filesystem comparison key was unavailable. p99 and several phase/resource
metrics were unavailable and remain explicitly unavailable, not zero.

The later default-base commits contain governance, documentation, and test
changes but no production Runtime source change. That fact supports a
current-base semantic revalidation of the decline; it is not a new latency
benefit claim. The candidate implementation was not merged.

## Current state

The Runtime lifecycle is `checkpointed`; current-base verification, reviewed
PR delivery, finalization, archive, and human close remain pending. The
authoritative current evidence will be
`.ai/evidence/WI-743-wi715-p1-current-base-redelivery.verification.json` and
the external predecessor records listed by the Contract.

## Decision boundary

Decision: preserve `declined`. No latency, CPU, I/O, memory, or resident-MCP
benefit is claimed. A future optimization must start from a reviewed default
base and provide a trustworthy environment comparator plus phase-level
evidence before proposing production behavior.
