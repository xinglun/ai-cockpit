---
author: AI Cockpit maintainers
title: "WI-472——收尾上下文兼容性"
description: "在 finish 和 archive 前将 pending provider 收尾哨兵识别为 provisional。"
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-472-finalization-context-compatibility
status: in_progress
authority: authorized
lastVerifiedBy: WI-472-finalization-context-compatibility
---

# WI-472——收尾上下文兼容性

## 意图与边界

防止 `pending:<stable-reference>` 这样的 provider 占位值被误认为完整的资源收尾计划。
在绑定已审查的 provider 资源之前，Work Item 必须保持可恢复。本 Work Item 不改写 WI-471
或任何其他历史字节，也不操作对象工程。

## 范围

- 将 `pending:*` 与 `unknown` 收尾上下文识别为 provisional。
- 在现有 `finish`/`archive` 边界 fail closed，并在拒绝时保留 active 字节。
- 增加 protocol、repository 回归测试，并同步三语文档。

## 验收

1. pending provider 上下文不能通过 `finish` 或 `archive`。
2. 完整且已审查的上下文仍可通过现有生命周期测试。
3. 拒绝不会移动或改写 active Work Item 字节。
4. 英文、简体中文、日文测试与文档描述相同的 provisional 边界。
5. WI-471 保持不可变；修复发布后只能通过显式 successor receipt 恢复。

## 验证

- `cargo test --locked -p cockpit-protocol --test resource_finalization`
- `cargo test --locked -p cockpit-repository --test archive_integrity`
- `cargo test --locked --workspace`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`

## 恢复边界

provider PR 尚未知时，使用明确的 provisional context 并保持 Work Item active。在 verification、
finish、archive 前绑定准确的已审查 PR URL；不得编辑不可变 archived Contract 来替换 pending 哨兵。
