---
author: AI Cockpit maintainers
title: "WI-235——最终化恢复与清洁批次边界"
workItemId: WI-235-finalization-recovery
description: "恢复 WI-234 归档交付，并在验证与归档前绑定已审阅 PR context。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-235-finalization-recovery
---

# WI-235——最终化恢复与清洁批次边界

WI-235 是 PR #185 暴露的流程缺口的窄 successor：WI-234 在
`finalize-plan` 之前就归档，归档 Contract 因而保留 pending resource context，
治理 gate 正确拒绝了缺少 terminal decision 的状态。PR 失败记录和 WI-234 的全部
bytes 保持不可变。

本 successor 会在验证前绑定真实的已审阅 PR context，记录 recovery decision，
然后完成正常的 finalization boundary。同时证明下一批开始时不再遗留 WI-234/WI-235
的旧 worktree 或 branch。

## 验收边界

- `stale_awaiting_merge_close` 回归继续 fail closed。
- WI-234 通过精确 recovery receipt 标记为已恢复。
- `finalize-plan` 先于 verify、finish 和 archive。
- 并行 attach migration fixture 使用抗碰撞路径，确保完整 workspace 测试在并发执行时保持确定性。
- pre-merge finalization receipt、hosted checks、merge observation、精确清理与
  structured close 都绑定同一个 PR head。

## 参考

- [参考 parity ledger](../reference/reference-parity.zh-CN.md)
- [WI-234 不可变 Work Item](WI-234-post-merge-governance-cleanup.zh-CN.md)
- [治理 gate](../../tests/ci/governance_integrity_gate.py)
