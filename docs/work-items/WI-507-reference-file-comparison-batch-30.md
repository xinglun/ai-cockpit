---
author: AI Cockpit maintainers
title: "WI-507 — language-adaptation example reader boundary"
description: "Compare five maintained reference example README files without copying application stacks or source governance implementation."
audience:
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
workItemId: WI-507-reference-file-comparison-batch-30
sourceCommit: fde3380f81fea5fd2e288f7a8849f737dc074060
lastVerifiedBy: documentation-acceptance
---

# WI-507 — language-adaptation example reader boundary

[简体中文](WI-507-reference-file-comparison-batch-30.zh-CN.md) · [日本語](WI-507-reference-file-comparison-batch-30.ja.md)

## Goal

Read the next five maintained reference example README files one by one and
record evidence-backed boundaries for the Rust Runtime and its adopters. This
Work Item is a semantic comparison, not a request to copy a source example,
installer, Make bridge, SDK, or application stack.

Comparison Runtime: published `ai-cockpit` v0.2.60, binary SHA-256
`sha256:f04aa15868a6e3a590b109a7649c37d765cd2bb935213b9cd898f3ddec6b336d`.
The source baseline is pinned to commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`.

## File-by-file decisions

The pinned source files are recorded as `reference-only` because they are
provider/application onboarding examples. Their portable governance meaning is
limited to owner-declared scope, verification commands, evidence, and
repository context; those meanings are already documented by the Rust-native
routes below.

| Pinned reference path | Source digest (SHA-256) | Rust boundary |
| --- | --- | --- |
| `examples/flutter/README.md` | `f9823e1b30e87e2a105869dbdaa03bfac9ed49f73524f9c7bac2326804afe8c7` | `docs/reference/flutter-fixture-adaptation.*`, `docs/reference/contract-fields.md`, and `docs/reference/verification-route.md`; no Flutter/Dart installer, Make preset, coverage YAML, application code, or source JSON. |
| `examples/go/README.md` | `ad36fe62949555e0e324c38ad2e6a89f71b6c0f4f4bbc2868973769c8e48dcac` | `docs/getting-started/adopter-configuration.md`, `docs/reference/contract-fields.md`, and `docs/reference/verification-route.md`; Go toolchain, Make, coverage, and application examples remain adopter-owned. |
| `examples/java/README.md` | `e83eff645b0f7d21f42590197e88932bf2d106e124c053cff7a12b8470652b4a` | `docs/getting-started/examples/java.md`, `docs/reference/contract-fields.md`, and `docs/reference/verification-route.md`; Gradle/Spring/Android commands and sample code are not Runtime requirements. |
| `examples/kotlin/README.md` | `7324bbf6472865ffc1a0563a3faa1a06d6dffe6be33ec2cc90d794ad197f0e8d` | `docs/getting-started/adopter-configuration.md`, `docs/reference/contract-fields.md`, and `docs/reference/verification-route.md`; Kotlin/Gradle commands remain adopter/provider responsibilities. |
| `examples/php/README.md` | `a25a87b0b0295677d15da8a5d7751ee3c278cae5946e95020ae2cd79c33dd04b` | `docs/getting-started/adopter-configuration.md`, `docs/reference/contract-fields.md`, and `docs/reference/verification-route.md`; Composer/PHPUnit/PHPStan commands and application paths are not copied. |

## Boundary and non-claims

These examples do not establish SDK availability, test execution, provider
assurance, enterprise approval, or support for the represented language stack.
An adopter must declare its own scope, commands, authority, and evidence under
its own repository context. No source Contract decision or source JSON wire
shape is inherited.

## Acceptance

- Each of the five pinned paths is read at the maintained local reference
  commit and receives a non-deferred `reference-only` inventory record with a
  non-empty counterpart list and reason.
- The comparison and parity pages in English, Simplified Chinese, and
  Japanese record the same five paths, boundaries, and current counts.
- No reference checkout, object/adopter repository, global Agent/MCP setting,
  or unrelated Runtime behavior is changed.
- Conformance, documentation, Runtime verification, reviewed PR, merge, close,
  and exact cleanup checks pass.

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
modified. The inventory is semantic/documentation parity, not source command,
SDK, provider-state, or JSON-wire compatibility.
