---
author: AI Cockpit maintainers
workItemId: WI-140-verification-semantics
title: Verification semantics と Artifact archive integrity
description: 独立した Verification truth の次元を定義し、生成 Artifact を孤立させず archive する。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-140-verification-semantics
---

# WI-140 — Verification semantics

Planner と性能作業の前に、`VerificationTier` と `EvidenceAssurance` を
独立した Governance dimension として定義します。あわせて、archive 後も
生成された implementation approach や parallel intelligence sidecar が
`active` に残る repository Artifact 問題を修正します。保持中の parallel slot
は明示的に解放するまで archive を阻止します。

Evidence：

- `.ai/evidence/WI-140-verification-semantics.verification.json`
- `.ai/work-items/archive/WI-140-verification-semantics.archive.json`
- `.ai/decisions/WI-140-verification-semantics.close.json`
