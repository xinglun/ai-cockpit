---
author: AI Cockpit maintainers
title: "WI-563 — reference file 比較 batch 43"
description: "維持対象の reference script 20 件を比較し、Rust の意味境界を記録する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-563-reference-file-comparison-batch-43
lastVerifiedBy: WI-563-reference-file-comparison-batch-43
terminalArchive: .ai/work-items/archive/WI-563-reference-file-comparison-batch-43.contract.json
terminalVerification: .ai/evidence/WI-563-reference-file-comparison-batch-43.verification.json
terminalFinalization: .ai/decisions/WI-563-reference-file-comparison-batch-43.finalize.json
terminalDecision: .ai/decisions/WI-563-reference-file-comparison-batch-43.close.json
---

[English](WI-563-reference-file-comparison-batch-43.md) · [简体中文](WI-563-reference-file-comparison-batch-43.zh-CN.md)

# WI-563 — reference file 比較 batch 43

## 目的

Pinned local reference checkout `fde3380f81fea5fd2e288f7a8849f737dc074060` の次の維持対象 20 file を一つずつ読み、明示的な Rust counterpart または bounded な source/provider-only 判断を記録する。これは semantic comparison であり、implementation や JSON wire の copy ではない。

## 範囲と境界

対象は wizard I/O/localization、Work Item intelligence/benchmark/status、Bootstrap repository/wizard/write boundary、CI、documentation、governance、absurd-case、release checker である。machine ledger、三言語 comparison/parity page、Work Item page を更新する。

Python、Shell、Make、source locale、provider credential、generated history、source JSON schema は copy しない。Runtime behavior、object repository、global Agent/MCP configuration は変更しない。source 固有 wizard、Bandit/coverage floor、deprecated-asset registry、benchmark report、provider distribution behavior は明示的に境界を記録し、Rust capability だと暗黙に主張しない。

## 比較結果

20 path は `implemented-different-by-design` 14 件、`reference-only` 5 件、`not-applicable` 1 件である。ledger、source pin、counterpart list、英中日 page は同じ path set を使用する。本 batch に `migrate-gap` や portable implementation omission はない。attached object repository は shared Runtime、explicit repository binding、isolated Contract/evidence/knowledge、trust/lifecycle gate、visible human Outcome を継承するが、source Python module、provider policy value、source wire format は継承しない。

## 検証

- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `git diff --check`
