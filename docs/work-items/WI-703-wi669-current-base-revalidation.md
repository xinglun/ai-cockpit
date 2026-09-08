---
author: AI Cockpit maintainers
title: "WI-703 — WI-669 P0-B current-base revalidation"
description: "Re-deliver the deterministic Outcome summary and explicit full evidence view from the current default base."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-703-wi669-current-base-revalidation
lastVerifiedBy: WI-703-wi669-current-base-revalidation
---

[简体中文](WI-703-wi669-current-base-revalidation.zh-CN.md) · [日本語](WI-703-wi669-current-base-revalidation.ja.md)

# WI-703 — WI-669 P0-B current-base revalidation

## Intent

Re-deliver P0-B from the latest remote default branch after PR #668 became
conflicting when the predecessor was archived. The default human handoff has
four deterministic sections: result, key changes, remaining uncertainty, and
human next step. The full evidence report remains explicitly available.

## Boundary

This Work Item is presentation-only. It preserves machine JSON, validation
rules, authorization semantics, exit codes, persistence layout, and historical
evidence. CLI and MCP use the same repository-renderer facts. It does not claim
measured cognitive benefit or a real user study.

## Base and recovery lineage

- Remote/default base: `origin/main` at `83a3bb6b4d8072d2c273bec349581ce2441510cc`.
- Predecessor: WI-669, whose archive and historical verification remain immutable.
- Recovery decision: `.ai/decisions/WI-669-p0b-outcome-summary.recovery.19fd6109e7b826919cde18fb6bf3ec6138808dc8d4ad3579ae9cef8b5bcac23d.json`.
- Historical evidence: `.ai/evidence/WI-669-p0b-outcome-summary.verification.json`.

## Acceptance

- Default CLI/MCP human output uses the four reader-first sections.
- `view: full` / `--view full` retains the audit-oriented report.
- Critical blockers, human decisions, stale/invalid evidence, and uncertainty
  remain visible; only non-critical lists may be summarized with an explicit
  full-report route.
- Tests construct real Outcome structures and cover multilingual and historical
  cases. Machine JSON remains unchanged.

## Current state

Implementation is active on the recovered current-base branch. Verification,
hosted delivery, provider finalization, and explicit human close remain pending.
