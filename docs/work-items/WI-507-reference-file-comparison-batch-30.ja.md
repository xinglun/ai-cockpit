---
author: AI Cockpit maintainers
title: "WI-507 — language-adaptation example reader boundary"
description: "維持された reference example README 5 件を比較し、application stack と source governance implementation を copy しない。"
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

[English](WI-507-reference-file-comparison-batch-30.md) · [简体中文](WI-507-reference-file-comparison-batch-30.zh-CN.md)

## Goal

維持された reference example README の次の 5 path を一つずつ読み、Rust
Runtime と adopter の境界を evidence-backed に記録します。これは semantic
comparison であり、source example、installer、Make bridge、SDK、application
stack の copy を求めるものではありません。

比較 Runtime: 公開済み `ai-cockpit` v0.2.60、binary SHA-256 は
`sha256:f04aa15868a6e3a590b109a7649c37d765cd2bb935213b9cd898f3ddec6b336d`。
source baseline は commit
`fde3380f81fea5fd2e288f7a8849f737dc074060` に固定します。

## File-by-file decisions

固定した source path は provider/application onboarding example なので
`reference-only` とします。portable な意味は owner が宣言する scope、
verification command、evidence、repository context に限られ、下記の
Rust-native route がその境界を保持します。

| Pinned reference path | Source SHA-256 | Rust boundary |
| --- | --- | --- |
| `examples/flutter/README.md` | `f9823e1b30e87e2a105869dbdaa03bfac9ed49f73524f9c7bac2326804afe8c7` | `docs/reference/flutter-fixture-adaptation.*`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`。Flutter/Dart installer、Make preset、coverage YAML、application code、source JSON は copy しません。 |
| `examples/go/README.md` | `ad36fe62949555e0e324c38ad2e6a89f71b6c0f4f4bbc2868973769c8e48dcac` | `docs/getting-started/adopter-configuration.md`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`。Go toolchain、Make、coverage、application example は adopter 責任です。 |
| `examples/java/README.md` | `e83eff645b0f7d21f42590197e88932bf2d106e124c053cff7a12b8470652b4a` | `docs/getting-started/examples/java.md`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`。Gradle/Spring/Android command と sample code は Runtime requirement ではありません。 |
| `examples/kotlin/README.md` | `7324bbf6472865ffc1a0563a3faa1a06d6dffe6be33ec2cc90d794ad197f0e8d` | `docs/getting-started/adopter-configuration.md`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`。Kotlin/Gradle command と coverage pattern は adopter/provider responsibility です。 |
| `examples/php/README.md` | `a25a87b0b0295677d15da8a5d7751ee3c278cae5946e95020ae2cd79c33dd04b` | `docs/getting-started/adopter-configuration.md`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`。Composer/PHPUnit/PHPStan command と application path は copy しません。 |

## Boundary and non-claims

これらの example は SDK availability、test execution、provider assurance、
enterprise approval、または該当 stack の support を証明しません。adopter は
自分の repository context で scope、command、authority、evidence を宣言します。
source Contract decision と source JSON wire shape は継承しません。

## Acceptance

- 5 つの pinned path を maintained local reference commit で読み、non-empty
  counterpart/reason を持つ non-deferred `reference-only` record を作成する。
- English、Simplified Chinese、日本語の comparison/parity page が同じ 5 path、
  boundary、current count を記録する。
- reference checkout、object/adopter repository、global Agent/MCP setting、
  unrelated Runtime behavior を変更しない。
- conformance、documentation、Runtime verification、reviewed PR、merge、close、
  exact cleanup check が通る。

## Verification

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

Local reference は `AI_COCKPIT_REFERENCE_ROOT` 経由で読み、変更しません。inventory
は semantic/documentation parity であり、source command、SDK、provider state、
JSON-wire compatibility ではありません。
