---
author: AI Cockpit maintainers
title: "WI-572 — installer と quality の reference 比較 batch 45"
description: "保守対象の reference path 20 件を比較し、bounded な Rust semantic decision を記録する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-572-reference-installer-quality-batch-45
lastVerifiedBy: WI-572-reference-installer-quality-batch-45
---

[English](WI-572-reference-installer-quality-batch-45.md) · [简体中文](WI-572-reference-installer-quality-batch-45.zh-CN.md)

# WI-572 — installer と quality の reference 比較 batch 45

## Objective

Pinned local reference checkout `fde3380f81fea5fd2e288f7a8849f737dc074060`
の次の保守対象 20 path を一つずつ再読し、Rust counterpart または bounded
source/provider-only decision を記録します。これは semantic comparison で
あり、source implementation や JSON wire の移行ではありません。

## Result

完全な path 単位の台帳は `tests/conformance/reference_file_inventory.json`
と tri-language comparison page にあります。19 件は
`implemented-different-by-design`（installer 9 件、quality 3 件、release/
quality runner・summary・publish projection・claim/quick-install の各責務）
です。`scripts/real_adopter_reference_validation.py` のみ
`reference-only` で、reference template 固有の七 project matrix であり、
portable Rust Runtime contract ではありません。

## Boundary と adopter 継承

shared Rust Runtime、明示的な `--repo` context、typed Agent/release/
verification service、dynamic quality route、分離された
Contract/evidence/knowledge、人間向け Outcome handoff が target capability
です。source Python、Make/provider orchestration、source wire、template 固有
stack matrix は copy しません。attached object/adopter repository は同じ
Runtime capability と boundary を継承しますが、source implementation は継承
しません。

## Verification

- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo . --report /tmp/ai-cockpit-governance-integrity.json`
- `git diff --check`
