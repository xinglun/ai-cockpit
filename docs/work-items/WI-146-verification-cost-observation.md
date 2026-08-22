---
author: AI Cockpit maintainers
workItemId: WI-146-verification-cost-observation
title: Verification cost observation
description: Add advisory, identity-bound cost estimates and execution observations.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-146-verification-cost-observation
---

# WI-146 — Verification cost observation

This Work Item adds cost estimates and execution observations for single and
parallel verification. The projection is advisory only: it cannot change
policy-derived `VerificationTier`, `EvidenceAssurance`, protected gates, or
the governance result. Confidence is explicit (`complete`, `partial`, or
`unknown`), and unknown facts remain visible.

Implementation evidence: `.ai/evidence/WI-146-verification-cost-observation.verification.json`.
Closure decision: `.ai/decisions/WI-146-verification-cost-observation.close.json`.
