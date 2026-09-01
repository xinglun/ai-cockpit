---
author: AI Cockpit maintainers
title: "WI-476——WI-475 终态文档提升"
description: "在不改写不可变记录的前提下，将已关闭 WI-475 的证据提升到面向读者的投影。"
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-476-wi475-doc-promotion
status: active
authority: authorized
lastVerifiedBy: WI-476-wi475-doc-promotion
---

# WI-476——WI-475 终态文档提升

## 意图与边界

本 Work Item 只负责将已验证、已关闭的 WI-475 lifecycle 提升到三语
Work Item 与 reference-parity 投影。不会修改不可变 Runtime evidence、参考清单、
Runtime 代码或任何对象工程。

[English](WI-476-wi475-doc-promotion.md) · [日本語](WI-476-wi475-doc-promotion.ja.md)

## 范围

- 在三语 Work Item 页面和 parity ledger 中绑定 WI-475 的 archive、verification、
  finalization 与 close 记录。
- 在 archive 前登记本 Work Item 自身的三语页面与 parity 行，并仅在验证关闭后提升。
- 保持已关闭 Work Item 文档提升检查可重复，并逐字保留历史 evidence。

## 不在范围内

Runtime/Core 实现、参考清单分类、发布或 adopter 脚本、对象工程以及全局 Agent/MCP 配置。

## 验收标准

1. `promote_closed_work_item.py --repo <repo> --work-item WI-475-reference-file-comparison-batch-25 --check` 通过。
2. 六份 WI-475 投影文件都绑定准确的 archive、verification、finalization 和 close evidence 路径。
3. 本 Work Item 拥有三语页面和 archive 前 parity 行，关闭后的提升是确定性的。
4. 不重写 Contract、archive、verification、finalization、close 或参考清单 bytes。
5. 英文、简体中文、日文页面保持语义等价，并保留 Contract 的原始语言。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-475-reference-file-comparison-batch-25 --check`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/getting_started_semantic.sh`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/ci/governance_integrity_gate_test.sh`
- `python3 tests/ci/repository_gate_manifest_test.py`
- `cargo test --locked --workspace`

本页面的终态字段只会在 reviewed merge、archive、finalization 和 close 完成后提升。
