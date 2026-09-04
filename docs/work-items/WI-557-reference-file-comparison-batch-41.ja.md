---
author: AI Cockpit maintainers
title: "WI-557 — reference script 比較 batch 41"
description: "deferred の reference script 13 件を比較し、Rust-native の semantic boundary を記録する。"
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: canonical
workItemId: WI-557-reference-file-comparison-batch-41
lastVerifiedBy: WI-557-reference-file-comparison-batch-41
terminalArchive: .ai/work-items/archive/WI-557-reference-file-comparison-batch-41.contract.json
terminalVerification: .ai/evidence/WI-557-reference-file-comparison-batch-41.verification.json
terminalFinalization: .ai/decisions/WI-557-reference-file-comparison-batch-41.finalize.json
terminalDecision: .ai/decisions/WI-557-reference-file-comparison-batch-41.close.json
---

[English](WI-557-reference-file-comparison-batch-41.md) · [简体中文](WI-557-reference-file-comparison-batch-41.zh-CN.md)

# WI-557 — reference script 比較 batch 41

## Objective

pinned local reference commit
`fde3380f81fea5fd2e288f7a8849f737dc074060` の 13 script を一つずつ比較し、
portable な責務、Rust-native counterpart、意図した boundary を append-only ledger
に記録します。これは semantic conformance であり、source code や JSON wire format の
copy ではありません。

## Scope

`scripts/ai_issue_log.py`、`scripts/ai_linked_worktree_recovery.py`、
`scripts/ai_ownership.py`、`scripts/ai_performance_budget.py`、
`scripts/ai_project_profile.py`、`scripts/ai_purge.py`、
`scripts/ai_readiness_policy.py`、`scripts/ai_recovery_usability.py`、
`scripts/ai_review_readiness_policy.py`、`scripts/ai_risk_policy.py`、
`scripts/ai_rollback.py`、`scripts/ai_safety_gate.py`、
`scripts/ai_schema_migration.py` と、Contract が指定する target ledger、conformance
check、三言語の比較/parity page を含みます。

## Boundary

Python module、source test、source registry、source JSON wire format は reference
material のままです。shared Rust Runtime、repository-local Protocol、external
provider boundary、object repository はこの batch では変更しません。target に固定された
generic recovery scenario registry はないため、`ai_recovery_usability.py` は等価実装と
せず明示的に reference-only とします。

## Acceptance

- 指定した全 source path に `WI-557` の明示的な ledger record が 1 件ずつある。
- 各 record に classification、Rust counterpart、evidence-based reason があり、
  deferred または migrate gap が残らない。
- ledger regression と三言語の比較/parity page が 13 path と pinned commit で一致する。
- source implementation bytes、object repository、global Agent configuration を変更しない。
- reference inventory、documentation、governance-integrity、diff check が通過する。

## Verification

- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `git diff --check`
