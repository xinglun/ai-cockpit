---
author: AI Cockpit maintainers
title: "WI-481——WI-480 终态文档提升"
description: "在不改写不可变证据的前提下，提升 WI-480 的终态文档投影。"
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-481-wi480-doc-promotion
status: in_progress
authority: authorized
lastVerifiedBy: WI-481-wi480-doc-promotion
---

# WI-481——WI-480 终态文档提升

本 Work Item 只将已验证、已关闭的 WI-480 lifecycle 提升到三语 Work Item
与 reference-parity 投影，不修改不可变 Runtime evidence、归档记录或参考清单。

[English](WI-481-wi480-doc-promotion.md) · [日本語](WI-481-wi480-doc-promotion.ja.md)

## 范围

- 使用仓库 helper 提升 WI-480 的六份文档投影。
- 保持提升过程确定性，并绑定准确的终态记录。
- 在 archive 前登记本 Work Item 自身的页面和 parity 行。

## 不在范围内

Runtime/Core 实现、发布或 adopter 产物、超出这些投影的参考源实现比对，以及不可变治理 bytes。

## 验收标准

1. `promote_closed_work_item.py --repo <repo> --work-item WI-480-finalization-context-recovery --check` 通过。
2. 合并后 `promote_closed_work_item.py --repo <repo> --check-all` 不报告 stale projection。
3. 不重写 Contract、Summary、Outcome、Evidence、Finalization、Close、Recovery 或参考清单 bytes。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-480-finalization-context-recovery --check`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/getting_started_semantic.sh`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/ci/governance_integrity_gate_test.sh`
- `python3 tests/ci/repository_gate_manifest_test.py`
- `cargo test --locked --workspace`
