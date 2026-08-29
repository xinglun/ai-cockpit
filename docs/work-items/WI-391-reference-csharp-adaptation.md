---
author: AI Cockpit maintainers
title: "WI-391 — C# adaptation example"
description: "Compare the pinned C# adaptation example without copying its installer or legacy wire format."
workItemId: WI-391-reference-csharp-adaptation
audience:
  - adopter
  - contributor
  - maintainer
authority: canonical
status: in_progress
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
lastVerifiedBy: WI-391-reference-csharp-adaptation
---

# WI-391 — C# adaptation example

[简体中文](WI-391-reference-csharp-adaptation.zh-CN.md) · [日本語](WI-391-reference-csharp-adaptation.ja.md)

## Intent and boundary

Compare the pinned `examples/csharp/README.md` section by section and document
the applicable C#/.NET adopter semantics in a Rust-native form. The goal is to
make the shared Runtime and repository-local responsibilities clear without
turning the source install script, Makefile, guard YAML, Python orchestration,
or legacy JSON examples into target requirements.

## Scope

- Add the tri-language C# adaptation reference and reference-index links.
- Record the installation, quality-gate, Contract, coverage, and guideline
  evidence mapping in the tri-language comparison and parity ledgers.
- Keep the pinned source commit and the source-vs-target semantic/non-wire
  boundary explicit.

## Out of scope

This Work Item does not add .NET tooling, a C# fixture, a second-technology
adopter acceptance, an installer implementation, a Makefile, a source guard
parser, Python checks, a provider integration, or a new Contract wire schema.

## Acceptance criteria

1. The four source sections (installation; quality gates and guards; Contract;
   `guidelinesCompliance`) each have a Rust-native mapping or an explicit
   external/non-applicable decision.
2. Source front matter is treated as descriptive metadata, not as target
   authority or a capability claim, and the source installer variables/flags
   are explicitly mapped to the different Rust installation boundary.
3. The adaptation page describes one shared immutable Runtime, explicit
   `attach --repo`, repository-local `.ai/`, explicit Agent adapter setup, and
   adopter/provider-owned `dotnet` checks.
4. The page states that source `contractVersion: 2`, `ai*` verification names,
   `Makefile.ai.stack`, and `guidelinesCompliance` are not direct Rust JSON-wire
   requirements; current Contract/evidence/decision boundaries are used.
5. English, Simplified Chinese, and Japanese pages, index links, inventory, and
   parity rows stay synchronized.
6. Documentation and conformance checks pass with fresh Runtime verification.

## Evidence boundary

This is documentation/semantic parity. It does not prove a C# adopter has been
run. Any future C# acceptance must use an immutable public Release, a distinct
repository context, and its own evidence/decision chain.
