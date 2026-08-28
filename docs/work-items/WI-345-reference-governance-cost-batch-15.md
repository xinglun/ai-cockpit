---
author: AI Cockpit maintainers
title: "WI-345 — governance cost and performance documentation batch 15"
workItemId: WI-345-reference-governance-cost-batch-15
description: "Compare five pinned governance cost/complexity/performance documents and record bounded Rust counterparts without inventing source tooling."
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
capabilityClaims:
  - reference_parity
---

# WI-345 — governance cost and performance documentation batch 15

## Intent and boundary

This Work Item compares five pinned reference documents individually:
governance complexity (English and Japanese), governance cost metrics,
governance performance budgets, and profile/cost separation. The target must
preserve the useful governance boundary for an adopter without copying source
Python/Make maintenance tooling, inventing timing evidence, or turning cost
into authority.

The scope is limited to the inventory, tri-language comparison/parity pages,
the new reader-facing reference pages, and this Work Item record. Runtime code,
source scripts/guard files, global Agent/MCP configuration, immutable history,
and hard performance targets are out of scope.

## File-by-file decisions

| Pinned reference path | Classification | Bounded target decision |
| --- | --- | --- |
| `docs/reference/governance-complexity.ja.md` | `reference-only` | The target documents the boundary and keeps immutable archive/integrity rules, but does not claim the source Python/Make scanner, thresholds, or equivalent metrics. |
| `docs/reference/governance-complexity.md` | `reference-only` | `inspect`, `status`, `doctor`, and the repository integrity gate expose target facts; source complexity reporting remains non-portable maintenance material. |
| `docs/reference/governance-cost-metrics.md` | `implemented-different-by-design` | `diagnose` and typed verification cost estimates/observations provide identity-bound advisory facts. Source JSONL phase/wait aggregation and report wire shape are not Rust requirements. |
| `docs/reference/governance-performance-budget.md` | `implemented-different-by-design` | Identity-bound `PerformanceBaseline` samples and explicit regression budgets reject invalid/regressed measurements without skipping required verification or deriving P95/profile authority. |
| `docs/reference/governance-profile-cost-separation.md` | `implemented-different-by-design` | Light/standard/strict routes, operation/stage escalation, `VerificationTier`, `EvidenceAssurance`, and advisory cost remain orthogonal. |

This is semantic/documentation parity, not source command or JSON-wire parity.
The object/adopter boundary remains one shared Runtime with explicit `--repo`,
repository-local evidence, policy-owned route requirements, and no global
current project.

## Acceptance and verification

- Each of the five pinned paths occurs exactly once in the inventory with the
  listed classification and no deferred or migrate-gap record.
- English, Simplified Chinese, and Japanese reference/parity pages agree on
  the decisions and current inventory counts.
- Reader-facing pages state which source details are not available and do not
  invent CLI commands, profile decisions, metrics, or assurance.
- Cost/performance output is explicitly advisory; timing never substitutes for
  `VerificationTier`, `EvidenceAssurance`, policy, or protected checks.
- Inventory, documentation, governance, formatting, lint, and locked workspace
  verification pass.

The pinned source is `e5acb677da6621004d96f0ef353c58fe8d3acfbf`; the target base
is `747cf3d9f846aac52b2a592ec61a874511c18b81`.

[简体中文](WI-345-reference-governance-cost-batch-15.zh-CN.md) ·
[日本語](WI-345-reference-governance-cost-batch-15.ja.md)
