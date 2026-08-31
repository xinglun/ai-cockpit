---
author: AI Cockpit maintainers
title: "WI-446 — 历史 finalization 恢复"
workItemId: WI-446-historical-finalization-recovery
description: "为旧 finalization 记录提供诚实、append-only 的恢复路径。"
audience: [maintainer, adopter, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-446-historical-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-446-historical-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-446-historical-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-446-historical-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-446-historical-finalization-recovery.close.json
---

# WI-446：历史 finalization 恢复

## 意图

为专用 linked worktree 和评审 PR 流程建立以前生成的旧 finalization 记录提供诚实、append-only 的兼容路径。Runtime 必须让对象工程完成历史收尾，但不能改写 predecessor receipt 或编造 PR。

## 范围

- 用 Runtime 绑定的 `historical_finalization_recovery` 记录分类旧共享主 worktree 的 `retained` receipt；
- 校验 repository、Work Item、Contract base、predecessor digest、Runtime 和人工授权绑定；
- 只有真实 merge commit、parents、base 和 Git 事实都匹配时，才接受无 PR 的完整 direct-merge receipt；
- 允许显式的低 assurance 历史 close，同时让新的 Work Item 继续受 deleted 资源门禁约束；
- 提供 `work-item finalize-recovery` 命令并同步三语文档。

## 非目标

不改写历史 bytes、不伪造 PR 编号、不削弱当前 Runtime identity 校验，也不自动迁移对象仓库。历史 assurance 始终是 `historical_low`，不是 provider assurance。

## 验收

仓库测试覆盖 shared-worktree recovery、foreign/tampered/symlink 拒绝、基于真实 Git parents 的 direct merge 验证，以及不改写 predecessor 的 close。命令和工作流参考说明兼容边界与人工授权的 recovery 命令。
