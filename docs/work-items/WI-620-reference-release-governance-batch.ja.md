---
author: AI Cockpit maintainers
title: "WI-620 — reference installer, release, quality, and lifecycle parity"
description: "残りの source-changed reference test を一件ずつ比較し、Rust の対応を記録します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-620-reference-release-governance-batch
status: in-progress
authority: canonical
lastVerifiedBy: WI-620-reference-release-governance-batch
---

# WI-620 — reference installer, release, quality, and lifecycle parity

## Intent

残り 22 件の source-changed reference test path を一件ずつ再読し、evidence-backed
な Rust counterpart または明示的な boundary を記録します。目的は Runtime と
attached adopter の semantic coverage であり、source Python/Make、provider config、
source JSON wire の copy ではありません。

## Boundary と decision

Pinned local reference は
`fde3380f81fea5fd2e288f7a8849f737dc074060`、target comparison は
`cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd` です。21 path は
`implemented-different-by-design`、`tests/test_install_plan.py` は
`reference-only` です。Rust は immutable Release artifact と明示的な
`attach --repo` を使用し、source の interactive stack/provider wizard は再現しません。
この batch に `migrate-gap` はありません。

File-level ledger は
[reference-file-comparison.ja.md](../reference/reference-file-comparison.ja.md#wi-620--reference-installer-release-quality-and-lifecycle-test-parity)、
machine record は
[reference_file_inventory.json](../../tests/conformance/reference_file_inventory.json) です。
22 path は installed Runtime parity、installer、Make/CI、PR aggregate、project governance、
quality architecture/measurement、reference impact、release distribution/preflight/state/workflow、
start/archive、supply chain、published release projection、verification policy、Work Item
intelligence/lifecycle、workflow を対象にします。

## Adopter inheritance

Attached object/adopter は shared external Runtime、explicit repository context、isolated
Contract/evidence/knowledge、dynamic verification、fail-closed lifecycle、人間向け Outcome
boundary を継承します。source Python test、Make target、interactive stack installer、provider
policy value、fixture stack、source wire format は継承しません。これは semantic parity であり、
source implementation/wire compatibility ではありません。

## Verification

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 --target-commit cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd
bash tests/conformance/reference_file_inventory_test.sh
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/conformance/reference_inventory_docs_test.py
python3 tests/docs/reference_comparison_metadata_test.py
cargo test --locked --workspace
```

Contract acceptance は原文を保持します。固定 presentation のみ localize し、governance fact や
human decision を翻訳・創作しません。

See also: [English](WI-620-reference-release-governance-batch.md) ·
[中文](WI-620-reference-release-governance-batch.zh-CN.md)。
