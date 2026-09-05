---
author: AI Cockpit maintainers
title: "WI-587 — reference test/fixture 比較 batch 47"
description: "次の 20 件の maintained reference test/fixture path を source 実装や wire data をコピーせず比較する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-587-reference-file-comparison-batch-47
lastVerifiedBy: WI-587-reference-file-comparison-batch-47
---

[English](WI-587-reference-file-comparison-batch-47.md) · [简体中文](WI-587-reference-file-comparison-batch-47.zh-CN.md)

# WI-587 — Reference test/fixture 比較 batch 47

## Objective

固定した local reference commit
`fde3380f81fea5fd2e288f7a8849f737dc074060` の maintained path 20 件を一つずつ
再読し、evidence-backed な semantic decision を記録する。本 batch は責務の
比較であり、Python 実装、source file、JSON wire の migration ではない。

## Path ごとの決定

次の 12 path は typed Rust Runtime、native test、release/adopter harness が
portable な責務を異なる設計で実装している（`implemented-different-by-design`）。

- `tests/repository_fixture.py`
- `tests/test_absurd_capability_truth.py`
- `tests/test_adoption_e2e.py`
- `tests/test_adoption_evidence.py`
- `tests/test_adoption_ready.py`
- `tests/test_ai_archive_work_item.py`
- `tests/test_ai_check_serial_order.py`
- `tests/test_ai_check_summary.py`
- `tests/test_ai_check_work_item.py`
- `tests/test_ai_external_handoff.py`
- `tests/test_ai_onboard.py`
- `tests/test_ai_post_archive_recovery.py`

次の 8 path は source 固有 input のため `reference-only` である。

- `tests/conftest.py`
- `tests/fixtures/japanese-capability-corpus.json`
- `tests/fixtures/wizard/android.json`
- `tests/fixtures/wizard/ios.json`
- `tests/fixtures/wizard/monorepo.json`
- `tests/snapshots/wizard/kotlin.json`
- `tests/snapshots/wizard/mixed.json`
- `tests/snapshots/wizard/swift.json`

完全な counterpart、classification、non-overclaiming reason は
[`reference_file_inventory.json`](../../tests/conformance/reference_file_inventory.json)
と三言語の[逐 file 比較](../reference/reference-file-comparison.ja.md)に記録する。

## Boundary と adopter inheritance

target は explicit repository context、adversarial fail-closed、immutable
release/adopter acceptance、archive/recovery integrity、serial lifecycle gate、
typed Contract/Summary、external handoff identity、explicit onboarding を保持する。
attached object/adopter repository は shared Runtime から同じ boundary と isolation
を継承するが、pytest fixture、participant/capability corpus、stack toolchain preset、
Python evaluator、source installer workflow、source JSON wire は継承しない。

本 batch に `migrate-gap` はなく、evidence のない governance decision も生成していない。

## Verification

- `python3 tests/conformance/reference_file_inventory.py --check`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `python3 tests/docs/reference_comparison_metadata_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `cargo test --locked --workspace --all-targets --all-features -- --test-threads=1`
- `git diff --check`
