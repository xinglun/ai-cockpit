---
author: AI Cockpit maintainers
title: "WI-568 — reference file comparison batch 44"
description: "次の保守対象 20 path を file 単位で比較し、bounded な Rust semantic decision を記録する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-568-reference-file-comparison-batch-44
lastVerifiedBy: WI-568-reference-file-comparison-batch-44
---

[English](WI-568-reference-file-comparison-batch-44.md) · [简体中文](WI-568-reference-file-comparison-batch-44.zh-CN.md)

# WI-568 — reference file comparison batch 44

## Objective

Pinned local reference checkout `fde3380f81fea5fd2e288f7a8849f737dc074060` の次の保守対象 20 path を一つずつ再読し、明示的な Rust counterpart または bounded な source/provider-only decision を記録します。これは semantic comparison であり、source implementation や JSON wire の移行ではありません。

## Result

17 path は `implemented-different-by-design`、3 path は source-template 固有の fixture/adoption driver として `reference-only` です。`migrate-gap` はありません。Rust は typed release/verification/agent boundary と immutable adopter acceptance を使い、Python module、source wire、stack matrix、provider configuration は copy しません。attached object/adopter repository は shared Runtime、explicit repository context、isolated Contract/evidence/knowledge、人間向け Outcome boundary を継承します。

## Scope boundary

Reference checkout、object repository、global Agent/MCP config、provider credential、source implementation は out of scope です。target の behavior omission が見つかった場合は Contract amendment を行い、この WI の範囲で安全に修正します。source 固有 behavior を Rust parity として暗黙に claim しません。

## Verification

- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `git diff --check`
