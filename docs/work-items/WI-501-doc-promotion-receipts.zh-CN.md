---
author: AI Cockpit 维护者
title: "WI-501——WI-500 终态文档与收据晋升"
description: "将已关闭 WI-500 的恢复证据和生成收据晋升到经过评审的文档基线。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-501-doc-promotion-receipts
status: implemented
authority: human-authorized
lastVerifiedBy: WI-501-doc-promotion-receipts
terminalArchive: .ai/work-items/archive/WI-501-doc-promotion-receipts.contract.json
terminalVerification: .ai/evidence/WI-501-doc-promotion-receipts.verification.json
terminalFinalization: .ai/decisions/WI-501-doc-promotion-receipts.finalize.json
terminalDecision: .ai/decisions/WI-501-doc-promotion-receipts.close.json
---

# WI-501——WI-500 终态文档与收据晋升

[English](WI-501-doc-promotion-receipts.md) · [日本語](WI-501-doc-promotion-receipts.ja.md)

## 边界

本限定文档 Work Item 将已关闭 WI-500 的恢复证据，以及 Runtime 生成的恢复、
finalization 和 close 收据晋升到经过评审的主线文档基线。不重写历史字节，
也不改变 Runtime 行为。

## 范围

- 逐字节跟踪生成的 WI-496 与 WI-500 恢复/close 收据。
- 将 WI-500 三语页面和 parity 条目晋升为有证据支持的终态。
- 将本 Work Item 的三语页面和 parity 条目纳入同一有界 lifecycle，使 close
  后文档检查保持自终态。

## 不在范围内

Runtime 源码、测试、对象/adopter 工程、参考源实现、版本发布、全局
Agent/MCP 配置、源码 fallback binary 以及历史重写。

## 验收

- 五个 Runtime 生成的 WI-496/WI-500 收据逐字节复制并跟踪，未手动编辑。
- WI-500 三语页面和 parity 条目链接到 archive、verification、finalization
  与 close 证据。
- 本 Work Item 三语页面说明晋升边界和收据来源。
- 关闭后的 Work Item 晋升、文档、parity、状态一致性和 diff 检查通过。
- 评审 PR 合并，并记录精确的 branch/worktree 清理。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

收据由 Runtime 生成；纳入跟踪后保持不可变。
