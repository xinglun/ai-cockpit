---
author: AI Cockpit maintainers
title: "WI-195 — Governance integrity recovery gate"
description: "Make the dynamic governance inventory recovery-aware and harden published adopter isolation receipts."
audience:
  - maintainer
  - reviewer
workItemId: WI-195-governance-recovery-gate
status: historical
authority: canonical
lastVerifiedBy: WI-196-governance-recovery-gate-retry
---

# WI-195 — Governance integrity recovery gate

This was the current-batch corrective Work Item that made the dynamic governance gate
accept a valid superseded predecessor as `recovered` history, while keeping
malformed, foreign, or missing recovery fail-closed. It also hardens the
published adopter and N-1 acceptance harnesses: source repository identity is
bound, every receipt write is checked, and temporary run roots are removed only
after identity-safe validation.

Recovery is not approval, verification, or merge authorization. The blocked
predecessor bytes remain immutable and red; the successor must independently
complete its Contract, evidence, hosted PR, and closure lifecycle.

An in-scope parity correction was discovered after finish evidence was written.
WI-195 remains immutable recovered history; the fresh delivery continues in
WI-196. The reference-source file-by-file comparison remains the next batch
after the corrected release and immutable public-artifact acceptance.

[简体中文](WI-195-governance-recovery-gate.zh-CN.md) ·
[日本語](WI-195-governance-recovery-gate.ja.md)
