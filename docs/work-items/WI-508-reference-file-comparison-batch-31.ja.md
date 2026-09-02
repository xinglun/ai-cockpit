---
author: AI Cockpit maintainers
title: "WI-508 — stack adaptation example reader boundary"
description: "維持された stack example 5 件を比較し、source installer、toolchain、application governance decision を copy しない。"
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

# WI-508 — stack adaptation example reader boundary

[English](WI-508-reference-file-comparison-batch-31.md) · [简体中文](WI-508-reference-file-comparison-batch-31.zh-CN.md)

## Goal

維持された reference stack-adaptation README の次の 5 path を一つずつ読み、Rust Runtime と adopter の境界を evidence-backed に記録します。これは semantic comparison であり、source installer、Make bridge、SDK、application example、Contract wire shape の copy を求めるものではありません。

source baseline は commit
`fde3380f81fea5fd2e288f7a8849f737dc074060` に固定します。比較と検証には公開済み `ai-cockpit` v0.2.60、binary SHA-256
`sha256:f04aa15868a6e3a590b109a7649c37d765cd2bb935213b9cd898f3ddec6b336d` を使用します。

## File-by-file decisions

5 path はすべて `reference-only` です。source/provider onboarding material として stack-specific install、quality command、coverage pattern、sample Contract/Summary prose を示すだけです。portable な意味は owner が宣言する scope、command、evidence、repository context に限られ、既存の Rust-native route がその境界を保持します。

| Pinned reference path | Source SHA-256 | Rust boundary |
| --- | --- | --- |
| `examples/python/README.md` | `80413e9611a2e03687733d13c433d9377c9cdaafd92b0d4d09b416da9c452d29` | `docs/reference/python-fixture-adaptation.*`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`。Python installer、Make、coverage、sample Contract/Summary decision は adopter 責任です。 |
| `examples/ruby/README.md` | `7b8b799edfca2550e63a2493a92e0be98d8ad2a72d30e9b91f381a6aea344f28` | `docs/getting-started/adopter-configuration.md`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`。Bundler/RuboCop/RSpec または Rake、coverage、application example は adopter/provider responsibility です。 |
| `examples/rust/README.md` | `60e83d31510f13c79dd5af221608577b50d1d6dfb14e7c0465f8c7f477574149` | `docs/getting-started/adopter-configuration.md`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`、`docs/reference/ci-quality-gates.md`。Cargo command、inline-test caveat、Make preset、sample Contract/Summary decision は project-owned です。 |
| `examples/swift/README.md` | `9c5f39905973dfa5400db502750d7eaffe873e287a31d79dc9da691d5e851d6e` | `docs/reference/ios-swift-fixture-adaptation.*`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`。SwiftPM/Xcode command、coverage、platform/signing assumption、sample decision は adopter/provider responsibility です。 |
| `examples/typescript/README.md` | `036d52e200a13eabb47a7843ccca81b9ecf044aa6e789e51b6bb0af2643fd53f` | `docs/reference/typescript-fixture-adaptation.*`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`。npm/Node script、dependency、fixture lifecycle、coverage、sample decision は adopter/provider responsibility です。 |

## Boundary and non-claims

これらの example は SDK availability、test execution、provider/enterprise assurance、または該当 stack の support を証明しません。adopter は自分の repository context で scope、command、authority、evidence を宣言します。source Contract decision、installer、Make preset、JSON wire shape は継承しません。

## Acceptance

- 5 つの pinned path を maintained local reference commit で読み、non-empty counterpart/reason を持つ non-deferred `reference-only` record を作成する。
- inventory、comparison page、parity page、Work Item page の English、Simplified Chinese、日本語が同じ 5 path、boundary、current count を記録する。
- reference checkout、object/adopter repository、global Agent/MCP setting、unrelated Runtime behavior を変更しない。
- conformance、documentation、Runtime verification、reviewed PR、merge、close、release、exact cleanup check が通る。

## Verification

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

Local reference は `AI_COCKPIT_REFERENCE_ROOT` 経由で読み、変更しません。inventory は semantic/documentation parity であり、source command、SDK、provider state、JSON-wire compatibility ではありません。
