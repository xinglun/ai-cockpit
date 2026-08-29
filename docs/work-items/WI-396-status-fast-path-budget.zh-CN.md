---
workItemId: WI-396-status-fast-path-budget
title: "status 快速路径与严格性能预算"
author: AI Cockpit maintainers
description: "移除 clean snapshot 的一次可证明冗余 subprocess，同时保持性能声明有 identity 绑定且 fail-closed。"
type: implementation
audience: [adopter, contributor, maintainer, reviewer]
authority: human-authorized
status: implemented
lastVerifiedBy: WI-396-status-fast-path-budget
terminalArchive: .ai/work-items/archive/WI-396-status-fast-path-budget.contract.json
terminalVerification: .ai/evidence/WI-396-status-fast-path-budget.verification.json
terminalFinalization: .ai/decisions/WI-396-status-fast-path-budget.finalize.json
terminalDecision: .ai/decisions/WI-396-status-fast-path-budget.close.json
---

# WI-396——status 快速路径与严格性能预算

[English](WI-396-status-fast-path-budget.md) · [日本語](WI-396-status-fast-path-budget.ja.md)

## 意图

继续 WI-395 之后的 Rust 性能收敛。clean repository snapshot 已确定等价
diff 为空，因此 Runtime 可以跳过冗余 Git subprocess；dirty 或不确定输入仍
必须执行完整 patch 检查，并保持相同治理事实。

## 边界

基准边界是声明平台上的 release/installed Runtime。status `<50 ms` 和中型
observation `<100 ms` 仍是明确目标；未达标必须记录有界差距或失败预算，不得
通过削弱验证来隐藏。Runtime identity 与 repository identity 始终记录。

Runtime 仍是一份共享外部 binary；adopter 通过显式 `--repo` 绑定，每个
repository 保持独立 `.ai/` 状态。不引入全局 cache、current repository、
provider/enterprise 性能声明，也不复制参照源 installer/Make/Python/V1。

## 验证

运行锁定 workspace tests、Git snapshot 回归、性能 fixture、identity-bound
regression gate、文档门禁和 `git diff --check`。最终 Work Item evidence 必须
包含命令、样本/中位数、`gitCalls`、Runtime digest 和 repository identity。
