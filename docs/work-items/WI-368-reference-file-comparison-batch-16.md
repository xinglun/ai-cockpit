---
author: AI Cockpit maintainers
title: "WI-368 — Reference file comparison batch 16"
description: "Eleven pinned reference files compared one by one with explicit Rust-native boundaries."
workItemId: WI-368-reference-file-comparison-batch-16
audience: [maintainer, reviewer]
status: implementation_active
authority: canonical
lastVerifiedBy: WI-368-reference-file-comparison-batch-16
capabilityClaims:
  - reference_parity
---

# WI-368 — Reference file comparison batch 16

[简体中文](WI-368-reference-file-comparison-batch-16.zh-CN.md) · [日本語](WI-368-reference-file-comparison-batch-16.ja.md)

## Intent and boundary

This Work Item compares the next eleven paths in the pinned reference
commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`, one file at a time. It
records whether each responsibility is represented by the Rust Runtime,
intentionally external, or historical reference material.

The target keeps one shared installed Runtime and explicit `--repo` contexts.
Source Python/Make/YAML orchestration, generated history, provider-global
configuration, source JSON wire compatibility, and public release creation
are out of scope. A semantic mapping is not a claim of identical commands or
fields.

## File-by-file decisions

| Pinned reference path | Classification | Bounded target decision |
| --- | --- | --- |
| `docs/reference/pre-release-documentation-alignment.md` | `reference-only` | Historical generated alignment evidence; current target documentation is checked by its own repository-local gates. |
| `docs/reference/pre-release-documentation-review.json` | `reference-only` | Historical five-strategy review record; source status and findings cannot authorize a target release. |
| `docs/reference/project-test-timing-baseline.json` | `implemented-different-by-design` | Map timing seeds to identity-bound Rust performance samples and advisory budgets; timing never lowers verification. |
| `docs/reference/provider-backed-governance-validation.md` | `implemented-different-by-design` | Keep provider configuration, branch protection, reviewer identity, and hosted controls as delegated evidence. |
| `docs/reference/real-absurd-injection-cases.md` | `implemented-different-by-design` | Preserve the semantic 15-case corpus and twelve named RAI cases through the canonical manifest and Rust tests. |
| `docs/reference/real-absurd-injection-cases.zh-CN.md` | `implemented-different-by-design` | Preserve the same Chinese semantic boundary; source prose is not Runtime authority. |
| `docs/reference/real-absurd-injection-cases.ja.md` | `implemented-different-by-design` | Preserve the same Japanese semantic boundary without claiming general language fluency. |
| `docs/reference/real-adopter-reference-validation.md` | `implemented-different-by-design` | Use immutable public Release adopter/upgrade acceptance with isolated repository, Runtime, lifecycle, and cleanup evidence. |
| `docs/reference/reference-impact-gate.md` | `reference-only` | The source static scanner/schema/Make commands are not shipped; operation-time policy remains a narrower declared-facts boundary. |
| `docs/reference/reference-impact-gate.zh-CN.md` | `reference-only` | Same explicit bounded gap in Chinese; no source scanner or provider claim is imported. |
| `docs/reference/reference-impact-gate.ja.md` | `reference-only` | Same explicit bounded gap in Japanese; no source scanner or provider claim is imported. |

The source reference-impact pages exposed an overclaim in the target Standard
profile. This batch corrects the three profile pages: Standard now requires
explicitly declared impact evidence and does not imply a static caller,
dynamic-reference, external-consumer, or monitoring scan. The existing
operation-time evaluator remains useful for declared operation/target/scope/
authority/freshness/trust/impact facts, but it is not a replacement for that
scanner.

The source real-absurdity language pages disagree about whether the named
scenario table contains twelve or fifteen cases. The target treats the
canonical manifest (`15` structured wording cases and `12` named RAI cases)
as the machine truth and keeps the discrepancy visible rather than guessing.

## Acceptance and verification

- Every pinned path appears exactly once in the inventory with a non-empty
  reason and no deferred or migrate-gap classification.
- Historical and provider records remain non-authoritative; timing/cost facts
  remain advisory; adopter evidence remains bound to an immutable public
  Release and isolated repository.
- The three adversarial language routes preserve identical deterministic
  semantics and the source-count discrepancy is explicit.
- Standard profile documentation no longer overclaims a reference-impact
  scanner; operation-time policy limitations are linked in all languages.
- Inventory, documentation metadata/links, parity, governance integrity, and
  targeted tests pass. No source Python/Make/V1 file or global Agent/MCP
  configuration is added.
- Installed v0.2.38 Runtime executes the repository-bound lifecycle and the
  final human Outcome is delivered visibly before merge/close and cleanup.

Pinned reference commit: `e5acb677da6621004d96f0ef353c58fe8d3acfbf`.
