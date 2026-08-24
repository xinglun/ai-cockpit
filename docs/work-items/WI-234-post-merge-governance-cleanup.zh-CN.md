---
author: AI Cockpit maintainers
title: "WI-234——合并后治理清理与 stale-close 防护"
workItemId: WI-234-post-merge-governance-cleanup
description: "闭合合并后的治理流程，防止 stale merged receipt，并让下一批从干净环境开始。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-234-post-merge-governance-cleanup
---

# WI-234——合并后治理清理与 stale-close 防护

## 意图

在下一批逐文件比对开始前闭合合并后的治理流程。保留不可变的失败与恢复历史，
然后清理废弃分支、worktree 和临时检出，使下一批从干净环境开始。

## 为什么需要本 Work Item

近期 hosted 失败暴露了两类重复流程缺口：

- 已审阅的 head 可能已经进入同步后的默认分支，但 pre-merge finalization
  receipt 仍记录为 `unmerged`；
- 合并后生成的 evidence 可能超出原始 release Contract scope，使真实合并无法
  关闭，只能建立 recovery Work Item。

本 Work Item 将它们固化为流程控制。既有失败 PR、Contract 和 evidence 保持
不可变；清理只记录处置结果，不改写历史。

## 范围

- 增加确定性的 `stale_awaiting_merge_close` governance-gate finding 和回归
  fixture。
- 三种语言的 parity ledger 同步登记 WI-222、WI-227、WI-230 以及本 Work Item
  的最终处置。
- 保留 WI-230 的 append-only 历史 transition，并通过当前 Work Item 绑定恢复。
  WI-222 继续作为 linked immutable history，不伪造第二条 successor edge。
- 在仓库外归档 WI-189/WI-193/WI-222/WI-223/WI-224/WI-225/WI-228 的精确脏
  worktree bytes 和分支 tip；记录 PR 处置后只删除这些明确废弃的 checkout 和 ref。

## 不在范围内

不改写 predecessor 的 Contract、Summary、Outcome、Events、verification、archive
或 hosted failure bytes。不修改全局 Agent/MCP 配置，也不修改用户根工作树中已有的
文件。

## 验收标准

1. 当 reviewed head 已在同步默认分支，而当前版本的 pre-merge receipt 仍为
   `unmerged` 时，gate 必须以稳定的 `stale_awaiting_merge_close` finding 拒绝。
2. 三语 parity ledger 内容一致，并引用精确 evidence/decision 路径。
3. 历史分支/worktree 必须先保存在带摘要的外部归档中，或在 PR 已关闭/取代后精确删除。
4. 根工作树已有的用户文件保持不变。
5. 在 finalize 前，已安装 Runtime 的 inspect/status/doctor 以及声明的治理和文档检查均通过。

## 恢复与清理规则

不可变 predecessor 只能使用 Runtime 支持的一条真实 successor edge 进行绑定。若需要第二条
edge，则在文档中保留历史链接并创建新的独立 Work Item，不伪造 receipt。清理采用 fail-closed：
删除 worktree 或分支前，先保存 archive 状态、未跟踪 bytes、分支 tip、PR 状态和 SHA-256 manifest。

## 参考

- [参考 parity ledger](../reference/reference-parity.zh-CN.md)
- [英文 parity ledger](../reference/reference-parity.md)
- [日文 parity ledger](../reference/reference-parity.ja.md)
- [Repository governance gate](../../tests/ci/governance_integrity_gate.py)
- [Gate 回归测试](../../tests/ci/governance_integrity_gate_test.sh)
