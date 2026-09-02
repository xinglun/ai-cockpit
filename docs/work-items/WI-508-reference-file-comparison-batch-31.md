---
author: AI Cockpit maintainers
title: "WI-508 — stack-adaptation example reader boundary"
description: "Compare five maintained reference stack examples without copying source installers, toolchains, or application governance decisions."
audience:
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
workItemId: WI-508-reference-file-comparison-batch-31
sourceCommit: fde3380f81fea5fd2e288f7a8849f737dc074060
lastVerifiedBy: WI-508-reference-file-comparison-batch-31
terminalArchive: .ai/work-items/archive/WI-508-reference-file-comparison-batch-31.contract.json
terminalVerification: .ai/evidence/WI-508-reference-file-comparison-batch-31.verification.json
terminalFinalization: .ai/decisions/WI-508-reference-file-comparison-batch-31.finalize.json
terminalDecision: .ai/decisions/WI-508-reference-file-comparison-batch-31.close.json
---

# WI-508 — stack-adaptation example reader boundary

[简体中文](WI-508-reference-file-comparison-batch-31.zh-CN.md) · [日本語](WI-508-reference-file-comparison-batch-31.ja.md)

## Goal

Read the next five maintained reference stack-adaptation README files one by
one and record evidence-backed boundaries for the Rust Runtime and its
adopters. This is a semantic comparison, not a request to copy a source
installer, Make bridge, SDK, application example, or Contract wire shape.

The source baseline is pinned to commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`. Comparison and verification use
the installed published `ai-cockpit` v0.2.60, binary SHA-256
`sha256:f04aa15868a6e3a590b109a7649c37d765cd2bb935213b9cd898f3ddec6b336d`.

## File-by-file decisions

All five paths are `reference-only`: they are source/provider onboarding
material that demonstrates stack-specific installation, quality commands,
coverage patterns, and sample Contract/Summary prose. The portable meaning is
limited to owner-declared scope, commands, evidence, and repository context;
existing Rust-native routes already carry those boundaries.

| Pinned reference path | Source SHA-256 | Rust boundary |
| --- | --- | --- |
| `examples/python/README.md` | `80413e9611a2e03687733d13c433d9377c9cdaafd92b0d4d09b416da9c452d29` | `docs/reference/python-fixture-adaptation.*`, `docs/reference/contract-fields.md`, and `docs/reference/verification-route.md`; Python installer, Make, coverage, and sample Contract/Summary decisions remain adopter-owned. |
| `examples/ruby/README.md` | `7b8b799edfca2550e63a2493a92e0be98d8ad2a72d30e9b91f381a6aea344f28` | `docs/getting-started/adopter-configuration.md`, `docs/reference/contract-fields.md`, and `docs/reference/verification-route.md`; Bundler/RuboCop/RSpec or Rake commands, coverage, and application examples remain adopter/provider responsibilities. |
| `examples/rust/README.md` | `60e83d31510f13c79dd5af221608577b50d1d6dfb14e7c0465f8c7f477574149` | `docs/getting-started/adopter-configuration.md`, `docs/reference/contract-fields.md`, `docs/reference/verification-route.md`, and `docs/reference/ci-quality-gates.md`; Cargo commands, inline-test caveats, Make presets, and sample Contract/Summary decisions remain project-owned. |
| `examples/swift/README.md` | `9c5f39905973dfa5400db502750d7eaffe873e287a31d79dc9da691d5e851d6e` | `docs/reference/ios-swift-fixture-adaptation.*`, `docs/reference/contract-fields.md`, and `docs/reference/verification-route.md`; SwiftPM/Xcode commands, coverage, platform/signing assumptions, and sample decisions remain adopter/provider responsibilities. |
| `examples/typescript/README.md` | `036d52e200a13eabb47a7843ccca81b9ecf044aa6e789e51b6bb0af2643fd53f` | `docs/reference/typescript-fixture-adaptation.*`, `docs/reference/contract-fields.md`, and `docs/reference/verification-route.md`; npm/Node scripts, dependencies, fixture lifecycle, coverage, and sample decisions remain adopter/provider responsibilities. |

## Boundary and non-claims

These examples do not establish SDK availability, test execution, provider or
enterprise assurance, or support for the represented stack. An adopter must
declare its own scope, commands, authority, and evidence under its own
repository context. No source Contract decision, source installer, Make
preset, or source JSON wire shape is inherited.

## Acceptance

- Each pinned path is read at the maintained local reference commit and has a
  non-deferred `reference-only` inventory record with a non-empty counterpart
  list and reason.
- The inventory, comparison pages, parity pages, and this Work Item record
  contain the same five decisions and current counts with no `migrate-gap`.
- No reference checkout, object/adopter repository, global Agent/MCP setting,
  or unrelated Runtime behavior is changed.
- Conformance, documentation, Runtime verification, reviewed PR, merge, close,
  release, and exact cleanup checks pass.

## Verification

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

The local reference is read through `AI_COCKPIT_REFERENCE_ROOT` and is never
modified. This inventory is semantic/documentation parity, not source command,
SDK, provider-state, or JSON-wire compatibility.
