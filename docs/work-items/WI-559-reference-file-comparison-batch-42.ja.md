---
author: AI Cockpit maintainers
title: "WI-559 — reference file 比較 batch 42"
description: "20 件の reference script を逐次比較し Rust の意味境界を記録する。"
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
authority: canonical
workItemId: WI-559-reference-file-comparison-batch-42
lastVerifiedBy: WI-559-reference-file-comparison-batch-42
---

[English](WI-559-reference-file-comparison-batch-42.md) · [简体中文](WI-559-reference-file-comparison-batch-42.zh-CN.md)

# WI-559 — reference file 比較 batch 42

状態：Implemented

## 目的

固定した local reference checkout の保守対象 script 20 件を Rust Runtime と一つずつ意味比較し、各 file の counterpart または bounded な reference-only 判断を記録する。

## 範囲と境界

machine ledger と三言語の comparison/parity page に `ai_onboard`、`ai_prepare_hosted_verification`、`ai_project_doctor`、`ai_projection_lease`、`ai_provider_merge_state_recovery`、`ai_quality_architecture`、`ai_resume_work_item`、`ai_start`、`ai_start_receipt`、`ai_task_event_log`、`ai_terminology`、`ai_trust_guards`、`ai_trust_schema`、`ai_uninstall_facts`、`ai_uninstall_proposal`、`ai_unknown_confirmation`、`ai_validate_java_runtime`、`ai_verification_context`、`ai_verification_policy`、`ai_verify` を登録する。

Python や shell implementation はコピーしない。Hosted snapshot 準備、Python AST architecture audit、Java runtime 選択は source/provider または adopter 固有である。Runtime behavior、object repository、global Agent/MCP configuration は変更しない。

## 結果

17 path は `implemented-different-by-design`、3 path は `reference-only`。ledger、source pin、Rust counterpart、三言語 page は一致する。各 object repository は shared Runtime、explicit repository binding、isolated Contract/evidence/knowledge、trust/lifecycle gate、human Outcome handoff を継承する。

## 検証

- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `git diff --check`
