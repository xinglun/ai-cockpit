---
author: AI Cockpit maintainers
title: "WI-270——参考源 Contract 语义批次"
workItemId: WI-270-reference-contract-batch
description: "逐文件比较固定参考源的 Contract 与治理语义首批范围。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-270-reference-contract-batch
authority: canonical
---

# WI-270——参考源 Contract 语义批次

## 意图

这是清理边界后的首个语义批次。逐文件比较固定参考源在 Contract、intent、scenario、
acceptance、parallel、decision 和 preflight 方面的行为。参考源仍然只是规格与行为语料；
不会复制参考 Runtime，也不会写入 provider 全局配置。

## 范围

首批固定为以下参考表面及其 Rust 对应文档/台账记录：

- `docs/concepts/decision-states.*`
- `docs/features/work-item-parallelism.*`
- `docs/reference/safe-parallel-verification.md`
- `docs/reference/work-item-intelligence-interface.md`
- `docs/reference/work-item-state-machine.md`
- `docs/reference/work-item-status-interface.md`
- `scripts/ai_acceptance_policy.py`
- `scripts/ai_check_scenario_coverage.py`
- `scripts/ai_check_work_item.py`
- `scripts/ai_decision_protocol.py`
- `scripts/ai_intent_policy.py`
- `scripts/ai_parallel_verification.py`
- `scripts/ai_preflight_review.py`
- `scripts/ai_scenario_policy.py`
- `scripts/ai_work_item_state.py`
- `tests/test_acceptance_policy.py`
- `tests/test_ai_parallel_verification.py`
- `tests/test_checkpoint_intent.py`
- `tests/test_contract_and_policy.py`
- `tests/test_intent_policy.py`
- `tests/test_parallel_lifecycle_contract.py`
- `tests/test_preflight_review.py`
- `tests/test_scenario_coverage_gate.py`

机器可读台账生成器 `tests/conformance/reference_file_inventory.py` 也在本批范围内，
确保重新生成台账时不会丢失本批分类。

每个路径必须且只能有一个台账分类，并记录 Rust counterpart 或外部边界、证据引用、明确的
gap/延期决定。没有对应物时不得静默宣称 parity。

## 验证

- 使用显式 `--repo` 的已安装 Runtime
- reference inventory 回归与治理完整性检查
- 三语文档验收
- 对有界实现修正运行对应 Rust 测试
- 输出包含状态、未知项、证据、决定和下一步的可见人类 Outcome

## 边界

本批不比较其余 720 条 deferred 路径，不实现新的技术栈 adopter，也不修改用户全局
Agent/MCP 配置。如果 gap 需要 Rust 代码修正，必须先 amend Contract，再在同一 WI 内修改并保留证据。
