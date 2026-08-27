---
author: AI Cockpit maintainers
title: "WI-323 — reference documentation foundation"
workItemId: WI-323-reference-documentation-foundation
description: "Compare nine pinned reference documentation paths and record Rust-native adopter and Agent boundaries."
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-323-reference-documentation-foundation
---

# WI-323 — reference documentation foundation

## Intent and goal

Compare the next nine deferred paths from the pinned reference commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` one by one. Preserve useful
governance semantics for this repository and future adopter repositories while
keeping the shared Runtime external, repository state isolated, and all
commands explicitly bound by `--repo`.

The user-provided Cursor adopter feedback is an external observation for this
batch. Current Runtime behavior is checked against it rather than assumed:
stable lifecycle stdout JSON, visible human handoff/replay, repository entry
gates, and pre-start cleanliness are already implemented. Cursor panel display,
diagnostic remediation, close-gap conveniences, and optional controls
scaffolding remain separate product decisions.

## Compared files

- `docs/contributing/installation-document-maintenance.md`
- `docs/current/README.md`
- `docs/design/harden-work-item-pr-closure.md`
- `docs/distribution.md`
- `docs/enterprise-security-boundary.md`
- `docs/examples/trust-layer-demo.sh`
- `docs/features/human-benefit-report.md`
- `docs/features/human-benefit-report.zh-CN.md`
- `docs/features/human-benefit-report.ja.md`

Every path receives an inventory classification and a non-empty reason. Eight
are Rust-native `implemented-different-by-design`; the offline trust demo is
`reference-only`. No `migrate-gap` is hidden.

## Scope and boundaries

The batch updates the comparison inventory/generator and regression assertions,
tri-language reference comparison pages, tri-language Human Benefit Report
feature pages, and this tri-language Work Item record. It explicitly documents
the source Make/Python/installer/demo boundary, semantic rather than wire/byte
parity, and the object/adopter model of one shared Runtime with private
repository-local `.ai/` state.

It does not add Runtime commands, alter lifecycle semantics, copy source
Python/Make/YAML/JSON wire files, require `Makefile.ai`, modify global
Agent/MCP configuration, rewrite historical evidence, or publish a Release.

## Acceptance and verification

1. The nine pinned source paths are read and individually classified with
   evidence-backed counterparts or an explicit reference-only decision.
2. The generated inventory contains exactly nine WI-323 records:
   eight `implemented-different-by-design` and one `reference-only`, with
   zero deferred or migrate-gap records for this batch.
3. English, Simplified Chinese, and Japanese comparison and Human Benefit
   Report pages are semantically aligned and cross-link their language routes.
4. Human-facing output documents `work-item outcome --repo ...`, MCP
   `work_item_outcome`, lifecycle stdout JSON versus human handoff, the
   required report order, evidence-count semantics, stale/malformed stop
   behavior, and preservation of Contract-authored acceptance text.
5. The record does not claim that a CLI can expand a Cursor chat panel or that
   the target provides the source-only `implementation_approach_report`,
   Make/Python generators, or trust-demo authority.
6. The installed Runtime validates the current repository context and all
   declared checks pass; no unrelated repository bytes are changed.

## Evidence

The immutable source and target baseline are recorded in the active Contract.
The generated inventory, documentation acceptance output, diff check, and
Runtime verification receipt are the authoritative evidence for this batch.

[简体中文](WI-323-reference-documentation-foundation.zh-CN.md) ·
[日本語](WI-323-reference-documentation-foundation.ja.md)
