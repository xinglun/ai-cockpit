---
author: AI Cockpit maintainers
title: "WI-799——WI-798 根因修复"
description: "在恢复发布前修复已发现的协作、比较差异、范围、治理和恢复循环根因。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized-root-cause-repair-and-release
workItemId: WI-799-wi798-root-cause-repair
lastVerifiedBy: WI-799-wi798-root-cause-repair
terminalArchive: .ai/work-items/archive/WI-799-wi798-root-cause-repair.contract.json
terminalVerification: .ai/evidence/WI-799-wi798-root-cause-repair.verification.json
terminalFinalization: .ai/decisions/WI-799-wi798-root-cause-repair.finalize.json
terminalDecision: .ai/decisions/WI-799-wi798-root-cause-repair.close.json
predecessorWorkItemId: WI-798-collaboration-observability-performance
recoveryDecision: .ai/decisions/WI-798-collaboration-observability-performance.recovery.7f58122c0b99798e61e08bed7f3f3c3ccf8dd4da8dd8dffebd1b4f7be68fd96c.json
---

[English](WI-799-wi798-root-cause-repair.md) · [日本語](WI-799-wi798-root-cause-repair.ja.md)

# WI-799——WI-798 根因修复

## 恢复边界

WI-799 是 WI-798 的有界 successor。WI-798 保持不可变历史证据；本 Work
Item 只修复在重新验证其实现和发布路径时发现的根因，不改写 predecessor
记录，也不把技术重试伪装成无关的新 Work Item。

## 根因边界

本次修复覆盖结构化 CLI 失败报告、修订后的重新验证、治理与 parity 登记、
clean worktree 上已提交 PR comparison facts，以及共享 Contract scope glob
语义。comparison gate 分别绑定 Contract 基线、CI comparison 基线和实际提交的
HEAD 差异；Runtime 拥有的治理路径也在 Contract 中明确声明。

## 验收

- 在昂贵验证前发现顺序、范围、过期证据和治理登记失败。
- 失败验证输出结构化证据并返回非零退出码，不把失败伪装成完成证据。
- hosted 风格的 clean checkout 能观察提交后的 comparison diff，包括变更测试和治理记录。
- Contract 范围模式与 evaluator 使用一致语义，并用回归测试证明文件名 glob 不跨目录。
- 英文、简体中文和日文文档及 parity 行绑定相同 predecessor、证据和未来终态生命周期。
- 在恢复任何 provider CI 或发布动作前，reviewed branch 通过 strict 本地 route。

## 验证边界

- 当前 Contract：`.ai/work-items/active/WI-799-wi798-root-cause-repair.contract.json`
- verification：`.ai/evidence/WI-799-wi798-root-cause-repair.verification.json`
- 计划中的终态 archive：`.ai/work-items/archive/WI-799-wi798-root-cause-repair.contract.json`
- 计划中的终态 verification：`.ai/evidence/WI-799-wi798-root-cause-repair.verification.json`
- 计划中的 finalization：`.ai/decisions/WI-799-wi798-root-cause-repair.finalize.json`
- 计划中的 close：`.ai/decisions/WI-799-wi798-root-cause-repair.close.json`

三语页面和 parity 行保留相同事实。provider CI、审查、合并、发布以及公开
产物验收仍待各自证据，不提前宣称完成。
