---
author: AI Cockpit maintainers
workItemId: WI-876-performance-current-proof
title: Current-version performance proof
description: Paired Runtime and development-cycle measurements for the four-direction convergence acceptance.
audience: [adopter, contributor, maintainer]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-876-performance-current-proof
terminalArchive: .ai/work-items/archive/WI-876-performance-current-proof.contract.json
terminalVerification: .ai/evidence/WI-876-performance-current-proof.verification.json
terminalDecision: .ai/decisions/WI-876-performance-current-proof.close.json
---

# WI-876 — Current-version performance proof

This Work Item establishes comparable evidence for Runtime latency and
development-cycle cost on one machine, toolchain, and scenario set. It does
not publish a release or change Runtime behavior.

## Acceptance boundary

- Release-grade percentile comparison requires at least 100 valid warm samples;
  99 remains diagnostic-only and is rejected by the gate.
- Paired measurements cover inspect, status, Outcome, verification planning and
  reuse, repository scale/history, single/multi-file changes, and invalid
  evidence with explicit unavailable reasons.
- Operation-scoped diagnostics bind applicable reads, hashes, parses, Git,
  subprocess, execution, and reuse counters to each external sample; overlap
  and diagnostics-on/off overhead are reported honestly.
- Contract-to-reviewable, verification-to-finish, and post-merge cleanup are
  reported separately with raw samples, p50/p95, agent operations, and
  preflight rejects.

The report will retain unknowns when the current Runtime or host cannot expose
an internal metric. Release publication remains outside this Work Item.
