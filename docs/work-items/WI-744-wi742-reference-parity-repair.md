---
author: AI Cockpit maintainers
title: "WI-744 — WI-742 reference-parity projection repair"
description: "Register the archived WI-742 coverage-document delivery in the tri-language reference-parity ledgers required by the docs governance gate."
audience: [maintainer, reviewer, adopter]
workItemId: WI-744-wi742-reference-parity-repair
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-744-wi742-reference-parity-repair
---

[简体中文](WI-744-wi742-reference-parity-repair.zh-CN.md) · [日本語](WI-744-wi742-reference-parity-repair.ja.md)

# WI-744 — WI-742 reference-parity projection repair

## Intent

Register the already archived WI-742 coverage-document delivery in the
English, Simplified Chinese, and Japanese reference-parity ledgers. The
successor exists because hosted docs governance identified the missing parity
projection after WI-742 was archived.

## Boundary

This is a documentation-only successor. It changes only the three
`docs/reference/reference-parity.*.md` ledgers and the required tri-language
Work Item projections. It does not change the WI-742 coverage content,
Runtime or production code, tests, scenario matrix, participant scope,
sections IV or V, onboarding track, or Outcome/performance/architecture/
contribution-experience tracks. WI-743 performance records remain outside
this Work Item.

## Acceptance and lifecycle

- Each parity ledger contains the archived WI-742 evidence binding and the
  pre-archive WI-744 lifecycle row with the future archive, verification,
  finalization, and close paths.
- `bash tests/docs/parity_status_check.sh` and
  `bash tests/docs/documentation_acceptance.sh` pass.
- The governed lifecycle is `start → preflight → checkpoint → verify →
  finish → archive → close`; no external participant confirmation is
  required.

## Evidence

- predecessor archive: `.ai/work-items/archive/WI-742-p1-invariant-7-doc-promotion.contract.json`
- predecessor verification: `.ai/evidence/WI-742-p1-invariant-7-doc-promotion.verification.json`
- successor recovery: `.ai/decisions/WI-742-p1-invariant-7-doc-promotion.recovery.93b3d1fdc6287fe50cdfac89827096a6b4b6a691c57a0f246aa15091167a6538.json`
- successor verification: `.ai/evidence/WI-744-wi742-reference-parity-repair.verification.json` (pending)
