---
title: "WI-605 — release API authentication for adopter acceptance"
description: "Keep staged and public adopter acceptance deterministic under repeated GitHub Release API access."
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: recovered
authority: canonical
workItemId: WI-605-release-api-auth
lastVerifiedBy: WI-605-release-api-auth
terminalArchive: .ai/work-items/archive/WI-605-release-api-auth.contract.json
terminalVerification: .ai/evidence/WI-605-release-api-auth.verification.json
terminalFinalization: .ai/decisions/WI-605-release-api-auth.finalize.json
terminalDecision: .ai/decisions/WI-605-release-api-auth.close.json
---

[简体中文](WI-605-release-api-auth.zh-CN.md) · [日本語](WI-605-release-api-auth.ja.md)

# WI-605 — release API authentication for adopter acceptance

## Objective

Make release adopter and N-1 acceptance resilient to GitHub API rate limits by
using the workflow token for release metadata requests while keeping artifact
downloads public and immutable.

## Boundary

The Work Item covers the two release harnesses, their static regression tests,
and the release workflow environment. Runtime governance semantics, release
artifact contents, installer behavior, and object repositories are outside the
boundary.

## Verification

Run the focused harness policy tests and the declared workspace verification.
Hosted checks must pass before merge; post-release acceptance must use only the
published artifact and preserve its isolation and cleanup evidence.
