---
author: AI Cockpit maintainers
title: "WI-715 — WI-713 P0-B uniquely identified redelivery"
description: "Re-deliver the deterministic Outcome summary and explicit full evidence view from the latest default base under a unique Work Item identity."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-715-wi713-p0b-redelivery
lastVerifiedBy: WI-715-wi713-p0b-redelivery
---

[简体中文](WI-715-wi713-p0b-redelivery.zh-CN.md) · [日本語](WI-715-wi713-p0b-redelivery.ja.md)

# WI-715 — WI-713 P0-B uniquely identified redelivery

## Intent

Re-deliver P0-B from the current remote default branch after WI-714 could not
pass deterministic closed-Work-Item promotion because another closed scope
already used the WI-714 identifier. The default human handoff has four
deterministic sections: result, key changes, remaining uncertainty, and human
next step. The full evidence report remains explicitly available.

## Boundary

This Work Item is presentation-only. It preserves machine JSON, validation
rules, authorization semantics, exit codes, persistence layout, and historical
evidence. CLI and MCP use the same repository-renderer facts. It does not claim
measured cognitive benefit or a real user study.

## Base and recovery lineage

- Current remote/default base: `origin/main` at `7ada6cd0928a877fe2bc719689abfe99bd532d7e`.
- Predecessor: WI-714, preserved as a recovered predecessor; its recovery bytes remain immutable.
- Predecessor recovery decision: `.ai/decisions/WI-714-wi713-current-base-revalidation.recovery.3efe3143da8d84cb32db0877de59ee702b12034925d300b895f5449cb756d676.json`.
- Historical evidence: `.ai/evidence/WI-714-wi713-current-base-revalidation.verification.json`.

## Acceptance

- Default CLI/MCP human output uses the four reader-first sections.
- `view: full` / `--view full` retains the audit-oriented report.
- Critical blockers, human decisions, stale/invalid evidence, and uncertainty
  remain visible; only non-critical lists may be summarized with an explicit
  full-report route.
- Tests construct real Outcome structures and cover multilingual and historical
  cases. Machine JSON remains unchanged.

## Current state

The unique successor is active from the current default base. Verification,
hosted delivery, provider finalization, and explicit human close remain pending.
