---
author: AI Cockpit maintainers
title: "WI-386 — reference documentation batch 19"
workItemId: WI-386-reference-documentation-batch-19
description: "Compare four pinned reference documents and record bounded Rust-native parity without copying historical authority."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-386-reference-documentation-batch-19
---

# WI-386 — reference documentation batch 19

## Intent and boundary

Compare `docs/review-final-evidence.md`, `docs/review-remediation-backlog.md`,
`docs/roadmap.md`, and `docs/security-boundaries.md` from pinned reference
commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`. Record an explicit
file-by-file decision in the inventory and all three parity ledgers.

The target is semantic/documentation parity, not source command, JSON-wire, or
provider-state compatibility. Historical review/backlog files remain
`reference-only`; current Rust-native documentation is the authority. Do not
copy source Python, Make orchestration, provider configuration, generated
GO/NO-GO claims, or historical/future roadmap milestones as shipped features.

## File decisions

| Pinned path | Decision | Maintained target boundary |
| --- | --- | --- |
| `docs/review-final-evidence.md` | `reference-only` | Fresh release/adopter evidence is produced by `docs/reference/final-replacement-acceptance.md`, `docs/reference/ci-release-evidence.md`, and repository-local Runtime records. |
| `docs/review-remediation-backlog.md` | `reference-only` | Current lifecycle and gate truth is maintained by `docs/reference/repository-workflow.md`, `docs/reference/governance-integrity-gate.md`, and the comparison ledger. |
| `docs/roadmap.md` | `implemented-different-by-design` | Mission, evidence governance, intent, human control, repository intelligence, and organization-policy direction are expressed in `docs/philosophy.md`, `docs/architecture.md`, and `docs/capabilities.md`; V1–V4 history is not a capability claim. |
| `docs/security-boundaries.md` | `implemented-different-by-design` | Content/authority separation, deterministic fail-closed handling, operation-time re-evaluation, adversarial limits, and external-control boundaries are expressed in the Rust-native security/reference docs. |

## Acceptance

- All four pinned source files are read and have one inventory classification,
  explicit counterparts, and a bounded reason; `migrate-gap` remains zero.
- English, Chinese, and Japanese comparison/parity ledgers describe the same
  four decisions and updated counts (`4262/294/1/4/47/511/0`).
- No source review backlog, roadmap history, security classifier code, Python,
  Make, provider configuration, or historical GO/NO-GO evidence is copied.
- The shared Runtime and explicit `--repo` model, including isolated adopter
  repository facts and evidence, is stated as the inheritance boundary.
- Documentation, inventory, governance, and installed Runtime lifecycle checks
  pass; no unrelated Runtime code or historical evidence is modified.

## Verification

Declared checks include the reference inventory documentation test, reference
inventory shell test, documentation/status consistency, governance integrity
gate, and the installed Runtime `inspect`, `status`, `doctor`, `preflight`,
`checkpoint`, `verify`, `finish`, `archive`, and `close` lifecycle with explicit
repository context.

[简体中文](WI-386-reference-documentation-batch-19.zh-CN.md) · [日本語](WI-386-reference-documentation-batch-19.ja.md)
