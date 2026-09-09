---
author: AI Cockpit maintainers
title: "WI-714 — WI-713 P0-B current-base revalidation"
description: "Re-deliver the deterministic Outcome summary and explicit full evidence view from the latest default base."
audience: [maintainer, reviewer, adopter]
status: recovered
authority: human:repository-owner
workItemId: WI-714-wi713-current-base-revalidation
lastVerifiedBy: WI-714-wi713-current-base-revalidation
---

[简体中文](WI-714-wi713-current-base-revalidation.zh-CN.md) · [日本語](WI-714-wi713-current-base-revalidation.ja.md)

# WI-714 — WI-713 P0-B current-base revalidation

## Intent

Re-deliver P0-B from the latest remote default branch after PR #704 became
conflicting while its predecessor was archived. The default human handoff has
four deterministic sections: result, key changes, remaining uncertainty, and
human next step. The full evidence report remains explicitly available.

## Boundary

This Work Item is presentation-only. It preserves machine JSON, validation
rules, authorization semantics, exit codes, persistence layout, and historical
evidence. CLI and MCP use the same repository-renderer facts. It does not claim
measured cognitive benefit or a real user study.

## Base and recovery lineage

- Historical attempted base: `origin/main` at `94fec9e43ff0c046a236a100d881b032124e973e`.
- Predecessor: WI-713, whose archive and historical verification remain immutable.
- Recovery decision: `.ai/decisions/WI-713-wi703-current-base-revalidation.recovery.09a026473430b0e8855dabacf111b91686189e3de7c009b241c8a03ac886e5cc.json`.
- Historical evidence: `.ai/evidence/WI-713-wi703-current-base-revalidation.verification.json`.
- Successor: WI-715, which owns the uniquely identified redelivery after the repository already used WI-714 for a different closed scope.
- WI-714 recovery decision: `.ai/decisions/WI-714-wi713-current-base-revalidation.recovery.3efe3143da8d84cb32db0877de59ee702b12034925d300b895f5449cb756d676.json`.

This page preserves immutable recovery history for the superseded predecessor; it does not claim that the predecessor delivered the redelivery.

## Acceptance

- Default CLI/MCP human output uses the four reader-first sections.
- `view: full` / `--view full` retains the audit-oriented report.
- Critical blockers, human decisions, stale/invalid evidence, and uncertainty
  remain visible; only non-critical lists may be summarized with an explicit
  full-report route.
- Tests construct real Outcome structures and cover multilingual and historical
  cases. Machine JSON remains unchanged.

## Current state

This Work Item is preserved as a recovered predecessor. Its attempted delivery
was not closed under the colliding WI-714 identifier; WI-715 owns the bounded
redelivery without rewriting these historical records.
