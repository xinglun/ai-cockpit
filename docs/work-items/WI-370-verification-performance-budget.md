---
author: AI Cockpit maintainers
title: "WI-370 — Verification performance budget and exact reuse"
description: "Reduce redundant verification latency through dynamic, identity-bound reuse without weakening governance."
workItemId: WI-370-verification-performance-budget
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-370-verification-performance-budget
capabilityClaims: [verification_performance, evidence_integrity]
---

# WI-370 — Verification performance budget and exact reuse

[简体中文](WI-370-verification-performance-budget.zh-CN.md) · [日本語](WI-370-verification-performance-budget.ja.md)

## Intent and boundary

This Work Item reduces repeated verification latency for the current repository
and adopter repositories. A detected Work Item command may use the
profile-authorized dynamic path, but only an exact identity-bound receipt can be
reused. Explicit custom commands remain fresh. Changes in repository snapshot,
Contract, scope, command, stage, runner, Runtime, profile, toolchain,
dependency, or policy force a new execution or a policy-directed escalation.

Required and protected governance checks are never skipped, and unknown impact
never becomes green because of timing or cache state. The Rust Runtime remains a
single shared installation; adopter repositories inherit the same selection
rule while keeping their evidence and repository identities isolated.

## Verification and acceptance

- Selection reports executed, reused, escalated, and denied nodes with stable reasons.
- Reuse binds repository, profile, Runtime, command, scope, stage, runner, base,
  toolchain, dependency, and policy context.
- A reused result writes fresh Work Item evidence and cannot authorize another
  Work Item.
- Current-project and published-adopter measurements retain cold/warm timing and
  Runtime/repository identities.
- Three-language documentation states that performance is a cost optimization
  only; it does not weaken verification truth or required gates.

The archived Contract and verification evidence remain the machine-readable
authority. This page is the reader-facing Work Item projection; its final
terminal links are added only after provider merge and cleanup are verified.
