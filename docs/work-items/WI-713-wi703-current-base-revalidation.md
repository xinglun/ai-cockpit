---
author: AI Cockpit maintainers
title: "WI-713 — WI-703 P0-B current-base revalidation"
description: "Re-deliver the deterministic Outcome summary and explicit full evidence view from the latest default base."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-713-wi703-current-base-revalidation
lastVerifiedBy: WI-713-wi703-current-base-revalidation
---

[简体中文](WI-713-wi703-current-base-revalidation.zh-CN.md) · [日本語](WI-713-wi703-current-base-revalidation.ja.md)

# WI-713 — WI-703 P0-B current-base revalidation

## Intent

Re-deliver P0-B from the latest remote default branch after PR #703 became
conflicting while its predecessor was archived. The default human handoff has
four deterministic sections: result, key changes, remaining uncertainty, and
human next step. The full evidence report remains explicitly available.

## Boundary

This Work Item is presentation-only. It preserves machine JSON, validation
rules, authorization semantics, exit codes, persistence layout, and historical
evidence. CLI and MCP use the same repository-renderer facts. It does not claim
measured cognitive benefit or a real user study.

## Base and recovery lineage

- Remote/default base: `origin/main` at `00700d88647246622729bca2d623eafd779eb1f3`.
- Predecessor: WI-703, whose archive and historical verification remain immutable.
- Recovery decision: `.ai/decisions/WI-703-wi669-current-base-revalidation.recovery.json`.
- Historical evidence: `.ai/evidence/WI-703-wi669-current-base-revalidation.verification.json`.

## Acceptance

- Default CLI/MCP human output uses the four reader-first sections.
- `view: full` / `--view full` retains the audit-oriented report.
- Critical blockers, human decisions, stale/invalid evidence, and uncertainty
  remain visible; only non-critical lists may be summarized with an explicit
  full-report route.
- Tests construct real Outcome structures and cover multilingual and historical
  cases. Machine JSON remains unchanged.

## Current state

Implementation is active on the recovered latest-base branch. Verification,
hosted delivery, provider finalization, and explicit human close remain pending.
