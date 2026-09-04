---
author: AI Cockpit maintainers
title: "WI-563——参考源文件比对第 43 批"
description: "逐个比对 20 个维护中的参考脚本并记录有界的 Rust 语义决定。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-563-reference-file-comparison-batch-43
lastVerifiedBy: WI-563-reference-file-comparison-batch-43
---

[English](WI-563-reference-file-comparison-batch-43.md) · [日本語](WI-563-reference-file-comparison-batch-43.ja.md)

# WI-563——参考源文件比对第 43 批

## 目标

在固定本地参考 checkout `fde3380f81fea5fd2e288f7a8849f737dc074060` 上逐个阅读下一批 20 个维护中的文件，并记录明确的 Rust counterpart 或有界的 source/provider-only 决定。本任务是语义比对，不是复制实现或迁移 JSON wire。

## 范围与边界

范围包含 wizard I/O/localization、Work Item intelligence/benchmark/status、Bootstrap repository/wizard/write boundary，以及 CI、文档、治理、荒诞测试和 release checker。会更新机器台账、三语 comparison/parity 页面和本任务页面。

不复制 Python、Shell、Make、源 locale、provider credential、generated history 或源 JSON schema。不修改 Runtime 行为、对象工程或全局 Agent/MCP 配置。源专属 wizard、Bandit/coverage floor、deprecated-asset registry、benchmark report 和 provider distribution 行为保持明确边界，不静默宣称为 Rust capability。

## 比对结果

20 个路径中，14 个为 `implemented-different-by-design`，5 个为 `reference-only`，1 个为 `not-applicable`。台账、source pin、counterpart 列表以及中/英/日页面使用同一组路径。本批没有 `migrate-gap` 或可移植实现遗漏。对象工程继承 shared Runtime、显式 repository 绑定、隔离 Contract/evidence/knowledge、trust/生命周期门和可见 human Outcome；不继承源 Python 模块、provider policy 值或 source wire 格式。

## 验证

- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `git diff --check`
