---
workItemId: WI-390-reference-style-guide
title: "Reference Work Item style guide"
author: AI Cockpit maintainers
description: "Semantic comparison record for the pinned Work Item style guidance."
audience:
  - maintainer
  - reviewer
authority: canonical
status: implemented
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
lastVerifiedBy: WI-390-reference-style-guide
terminalArchive: .ai/work-items/archive/WI-390-reference-style-guide.contract.json
terminalVerification: .ai/evidence/WI-390-reference-style-guide.verification.json
terminalFinalization: .ai/decisions/WI-390-reference-style-guide.finalize.b0a9c123b5f157c327a4068001f478d05b6d39e152363bc167945e0dc83fe423.json
terminalDecision: .ai/decisions/WI-390-reference-style-guide.close.json
---

# WI-390 — Reference Work Item style guide

[简体中文](WI-390-reference-style-guide.zh-CN.md) · [日本語](WI-390-reference-style-guide.ja.md)

## Intent

Compare the pinned `docs/work-item-style-guide.md` one section at a time and
carry forward only its reader-facing governance semantics into the Rust-native
documentation. Installation and Runtime implementation are intentionally not
copied.

## Scope

- Pinned source: `docs/work-item-style-guide.md`
- Rust counterpart: `docs/reference/work-item-style-guide.*`
- Index/parity/inventory synchronization for this comparison

## Acceptance

- Outcome-first writing, explicit problem/boundary/non-goal guidance, observable
  acceptance, human-owned decisions, executable verification, proportional
  process, and documentation-before-schema are represented.
- The shared Runtime and explicit `--repo` repository isolation are explained;
  no installer commands or source runtime code are reproduced.
- Tri-language links and comparison records remain consistent.

## Verification boundary

This is semantic/documentation parity, not source command, JSON-wire, or
provider-state compatibility. Object/adopter repositories inherit the
reader-facing rules through their own `.ai/` and adapter while Contracts,
evidence, knowledge, and repository identity remain repository-scoped.

## Evidence

Terminal evidence is recorded by the Runtime under:

- `.ai/evidence/WI-390-reference-style-guide.verification.json`
- `.ai/work-items/archive/WI-390-reference-style-guide.*`
- `.ai/decisions/WI-390-reference-style-guide.close.json`
