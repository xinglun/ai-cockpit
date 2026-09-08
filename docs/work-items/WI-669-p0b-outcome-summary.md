---
author: AI Cockpit maintainers
title: "WI-669 — P0-B Outcome summary and full evidence view"
description: "Provide a deterministic reader-first Outcome summary with an explicit full evidence view for CLI and MCP."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-669-p0b-outcome-summary
lastVerifiedBy: WI-669-p0b-outcome-summary
---

[简体中文](WI-669-p0b-outcome-summary.zh-CN.md) · [日本語](WI-669-p0b-outcome-summary.ja.md)

# WI-669 — P0-B Outcome summary and full evidence view

## Intent

Make the default human Outcome easier to scan while preserving calibrated
verification, lifecycle, human-decision, evidence, and uncertainty boundaries.
The summary is deterministic and shared by CLI and MCP; the full audit view
remains available explicitly.

## Boundary

This Work Item changes presentation only. Machine JSON, validation rules,
authorization semantics, exit codes, persistence layout, and historical
evidence remain unchanged. It does not claim a measured cognitive benefit or
real user study.

## Evidence

- Archive: `.ai/work-items/archive/WI-669-p0b-outcome-summary.contract.json`
- Verification: `.ai/evidence/WI-669-p0b-outcome-summary.verification.json`
- Hosted delivery: [PR #668](https://github.com/xinglun/ai-cockpit/pull/668)

## Current state

The Work Item remains in progress until the reviewed PR, provider finalization,
and explicit human close decision are complete.
