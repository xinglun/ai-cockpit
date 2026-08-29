---
author: AI Cockpit maintainers
title: "WI-378 — reference documentation batch 17"
description: "Compare the next pinned reference documentation paths and publish bounded Rust-native, tri-language counterparts."
workItemId: WI-378-reference-documentation-batch-17
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-378-reference-documentation-batch-17
capabilityClaims: [reference_comparison, documentation_governance, adopter_readiness]
---

# WI-378 — reference documentation batch 17

[简体中文](WI-378-reference-documentation-batch-17.zh-CN.md) · [日本語](WI-378-reference-documentation-batch-17.ja.md)

## Intent

Compare the next ten paths in the pinned reference inventory and make their
reader-facing governance meaning available through the shared Rust Runtime
without copying source Python, Make, provider configuration, or historical
decisions.

## Compared paths and decisions

| Pinned path | Decision |
| --- | --- |
| `docs/reference/remediation-instruction-traceability.json` | `reference-only`; generated historical plan trace is not target authority. |
| `docs/reference/repository-workflow.ja.md` | Rust-native tri-language workflow documentation. |
| `docs/reference/schemas.md` | Rust-native tri-language record-family and validation map. |
| `docs/reference/test-architecture.md` | Rust-native tri-language layered test and evidence model. |
| `docs/reference/test-weakening-guard.{md,zh-CN.md,ja.md}` | Rust-native snapshot-derived weakening route and bounded policy. |
| `docs/reference/troubleshooting.{md,ja.md}` | Rust-native explicit-repository recovery and toolchain boundary. |
| `docs/reference/upgrade.ja.md` | Rust-native Runtime upgrade and repository-migration boundary. |

The source English `upgrade.md` remains in the deferred inventory and will be
compared in its own bounded batch; the target tri-language upgrade pages are
provided here so the selected Japanese route has a complete reader path.

## Boundary

This is semantic/documentation parity, not source JSON-wire, command, Python,
Make, or provider parity. Every adopter uses one installed Runtime with an
explicit `--repo`; repository facts, Work Items, evidence, and decisions remain
isolated. Documentation never invents authority, approval, assurance, or
verification evidence.

## Acceptance

- Every selected pinned path has a classification and a target counterpart or an explicit `reference-only` decision.
- English, Simplified Chinese, and Japanese reader routes agree on the same boundaries and links.
- Inventory and parity ledgers agree on source commit, Work Item, classifications, and zero `migrate-gap`.
- Documentation/conformance checks and installed v0.2.39 Runtime verification pass.
- Source-language Contract facts remain unchanged; semantic parity is not presented as wire compatibility.

## Verification plan

Run the repository-local inventory, documentation, conformance, governance,
and installed Runtime checks with an explicit repository context. The terminal
archive, verification, finalization, and close receipts will be added only by
the Runtime after verification.

