---
author: AI Cockpit maintainers
title: "WI-559——参考源文件比对第 42 批"
description: "逐个比对 20 个参考脚本并记录 Rust 原生语义边界。"
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
authority: canonical
workItemId: WI-559-reference-file-comparison-batch-42
lastVerifiedBy: WI-559-reference-file-comparison-batch-42
---

[English](WI-559-reference-file-comparison-batch-42.md) · [日本語](WI-559-reference-file-comparison-batch-42.ja.md)

# WI-559——参考源文件比对第 42 批

状态：已实现

## 目标

针对固定本地参考 checkout 的下一批 20 个维护脚本逐个与 Rust Runtime 做语义比对，并为每个文件记录明确的 counterpart 或有界的 reference-only 决定。

## 范围与边界

机器台账以及三语 comparison/parity 页面登记了 `ai_onboard`、`ai_prepare_hosted_verification`、`ai_project_doctor`、`ai_projection_lease`、`ai_provider_merge_state_recovery`、`ai_quality_architecture`、`ai_resume_work_item`、`ai_start`、`ai_start_receipt`、`ai_task_event_log`、`ai_terminology`、`ai_trust_guards`、`ai_trust_schema`、`ai_uninstall_facts`、`ai_uninstall_proposal`、`ai_unknown_confirmation`、`ai_validate_java_runtime`、`ai_verification_context`、`ai_verification_policy`、`ai_verify`。

不复制 Python 或 shell 实现。Hosted snapshot 准备、Python AST 架构审计和 Java runtime 选择仍是 source/provider 或 adopter 专属能力。不修改 Runtime 行为、对象工程或全局 Agent/MCP 配置。

## 结果

17 个路径为 `implemented-different-by-design`，3 个为 `reference-only`。台账、source pin、Rust counterpart 列表和三语页面一致。对象工程继承 shared Runtime、显式 repository 绑定、隔离 Contract/evidence/knowledge、trust/生命周期门和 human Outcome handoff。

## 验证

- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `git diff --check`
