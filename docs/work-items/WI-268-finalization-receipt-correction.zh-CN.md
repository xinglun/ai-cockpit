---
author: AI Cockpit maintainers
title: "WI-268——Finalization receipt 修正"
workItemId: WI-268-finalization-receipt-correction
description: "通过显式 successor 修正不可变的无效 pre-merge finalization receipt。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-268-finalization-receipt-correction
authority: canonical
---

# WI-268——Finalization receipt 修正

## 意图

WI-267 的生成 pre-merge receipt 使用了治理门拒绝的 worktree identity，因此保留为不可变恢复历史。本 successor 记录协议有效的 receipt，并在不重写 WI-267 的前提下，让三语 parity 文档明确恢复关系。

## 范围与证据边界

- 精确绑定 successor Contract、branch、worktree、PR、repository、Runtime 与归档 Contract identity。
- 保持 WI-267 的 archive、verification、无效 finalization 与 recovery bytes 不变。
- 更新三语 parity 文档与本 Work Item 文档，明确 predecessor 与 successor 关系。
- 在提升状态前完成 hosted review、verification、finalization、精确 cleanup 与结构化 close。

## 验证

- `cargo test --locked --workspace`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/work_item_status_consistency_test.sh`
- 使用显式 `--repo` 的已安装 Runtime lifecycle 与可见人工 Outcome

最终交付必须是可见的 `Outcome: 🟢`、`Outcome: 🟡` 或 `Outcome: 🔴`，并包含状态、未知项、证据、人工决定和下一步。
