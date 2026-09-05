---
author: AI Cockpit maintainers
title: "WI-587 — 参考源测试与夹具对比第 47 批"
description: "逐个比较下一批 20 个参考源测试/夹具路径，不复制源实现或线协议。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-587-reference-file-comparison-batch-47
lastVerifiedBy: WI-587-reference-file-comparison-batch-47
---

[English](WI-587-reference-file-comparison-batch-47.md) · [日本語](WI-587-reference-file-comparison-batch-47.ja.md)

# WI-587 — 参考源测试与夹具对比第 47 批

## 目标

在固定的本地参考源提交
`fde3380f81fea5fd2e288f7a8849f737dc074060` 上逐个复核 20 个维护中的
路径，记录有证据的语义决定。本批是责任对比，不是复制 Python、源文件或
JSON wire。

## 逐文件决定

以下 12 个路径的可迁移责任已由 Rust Runtime、原生测试或发布/ adopter
脚手架以不同设计实现：

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

以下 8 个路径是参考源专属输入，分类为 `reference-only`：

- `tests/conftest.py`
- `tests/fixtures/japanese-capability-corpus.json`
- `tests/fixtures/wizard/android.json`
- `tests/fixtures/wizard/ios.json`
- `tests/fixtures/wizard/monorepo.json`
- `tests/snapshots/wizard/kotlin.json`
- `tests/snapshots/wizard/mixed.json`
- `tests/snapshots/wizard/swift.json`

完整的对应文件、分类和不越界理由见
[`reference_file_inventory.json`](../../tests/conformance/reference_file_inventory.json)
及三语[参考源逐文件对比](../reference/reference-file-comparison.zh-CN.md)。

## 边界与对象工程继承

目标工程保留显式仓库上下文、荒诞输入 fail-closed、不可变发布/adopter
验收、归档/恢复完整性、串行生命周期门、严格 Contract/Summary、外部
handoff 身份和显式 onboarding。对象/adopter 工程从共享 Runtime 继承这些
边界与隔离能力；不继承 pytest 夹具、参与者/能力样本、栈工具链预设、
Python evaluator、源安装流程或源 JSON wire。

本批未发现 `migrate-gap`，也没有生成未经证据支持的治理决定。

## 验证

- `python3 tests/conformance/reference_file_inventory.py --check`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `python3 tests/docs/reference_comparison_metadata_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `cargo test --locked --workspace --all-targets --all-features -- --test-threads=1`
- `git diff --check`
