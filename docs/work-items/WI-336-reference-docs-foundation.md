---
author: AI Cockpit maintainers
title: "WI-336 — first five governance-documentation paths"
workItemId: WI-336-reference-docs-foundation
description: "Compare the first five deferred reference governance documents and record Rust-native boundaries without copying source tooling."
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-336-reference-docs-foundation
---

# WI-336 — first five governance-documentation paths

## Intent and boundary

Compare five deferred paths from the pinned reference commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` one file at a time. The goal is an
auditable Rust-native mapping for adopters, not a byte-for-byte copy of the
reference Python, Make, provider, or historical surfaces.

## File-level comparison

| Pinned reference path | Classification | Rust/adopter counterpart and boundary |
| --- | --- | --- |
| `docs/reference/cross-wi-integration.md` | `reference-only` | Per-Work-Item archive validation, `reference-parity`, and human Outcome are the target audit boundary. The source WI-04..WI-13 aggregate report and UI receipt are not Runtime commands. |
| `docs/reference/dependabot-intake.md` | `not-applicable` | Dependabot bot-branch intake is provider-specific. Generic delegated evidence and explicit Work Item source binding remain external/provider-owned. |
| `docs/reference/deprecated-assets-registry.json` | `reference-only` | Explicit Runtime lifecycle, immutable history, and exact resource finalization govern cleanup; no source registry or Make scan is shipped. |
| `docs/reference/deprecated-assets.md` | `reference-only` | Registry hygiene and obsolete command-chain guidance remain source documentation; Rust does not claim `check-deprecated-assets`. |
| `docs/reference/derived-artifacts.md` | `implemented-different-by-design` | Typed Contract/evidence/archive facts and status/Outcome projections are separated by the Runtime and documented in the Outcome/verification references; derived views cannot authorize later decisions. |

## Non-goals

This Work Item does not add a cross-WI report engine, Dependabot integration,
deprecated-asset deletion command, derived-artifact registry, or any source
Python/Make/V1 implementation. It does not rewrite immutable history or modify
global Agent/MCP configuration.

## Acceptance and verification

1. All five pinned paths have exactly one inventory record with an explicit
   classification, counterpart, and non-overclaiming reason.
2. The English, Simplified Chinese, and Japanese comparison/parity ledgers
   agree on the classifications and state the semantic/non-wire boundary.
3. Existing Rust facts/views and external provider boundaries are documented;
   unsupported source commands are not presented as available.
4. Inventory, documentation, parity, and locked workspace verification pass.

[简体中文](WI-336-reference-docs-foundation.zh-CN.md) ·
[日本語](WI-336-reference-docs-foundation.ja.md)
