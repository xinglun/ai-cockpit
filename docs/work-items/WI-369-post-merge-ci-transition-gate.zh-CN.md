---
author: AI Cockpit maintainers
title: "WI-369——合并后的 CI 过渡门"
description: "区分经过 review 的 merge-to-close 过渡与历史遗留的未关闭 Work Item，同时不削弱质量门。"
workItemId: WI-369-post-merge-ci-transition-gate
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-369-post-merge-ci-transition-gate
terminalArchive: .ai/work-items/archive/WI-369-post-merge-ci-transition-gate.contract.json
terminalVerification: .ai/evidence/WI-369-post-merge-ci-transition-gate.verification.json
terminalFinalization: .ai/decisions/WI-369-post-merge-ci-transition-gate.finalize.d6e6c0bc91cdbdd880b1a8e9599e087d8003643969967bc0eef1156671d7ffa5.json
terminalDecision: .ai/decisions/WI-369-post-merge-ci-transition-gate.close.json
capabilityClaims:
  - governance_integrity
  - reference_parity
---

# WI-369——合并后的 CI 过渡门

[English](WI-369-post-merge-ci-transition-gate.md) · [日本語](WI-369-post-merge-ci-transition-gate.ja.md)

## 意图与边界

经过 review 的 merge 之后，默认分支 CI 会立即运行，而 provider finalization 和权威
close 回执在合并后的清理步骤中才会写入。这个 Work Item 消除由此产生的错误
`missing_terminal_decision`，但不会把缺少 close 降级为 advisory。

唯一允许的过渡是：真实 GitHub `push` 到配置的默认分支，`HEAD` 是精确的双父 merge，
并且该 merge 新增了该 Work Item 的归档 Contract。门报告
`awaiting_merge_close`；下一次普通默认分支提交仍然必须完成 finalization 和 close。

修改范围仅限仓库质量门、CI 调用说明、回归 fixture 和三语文档/parity 记录。Rust Runtime
生命周期语义、发布产物、provider API、全局 Agent/MCP 配置以及源 Python/Make/V1 runtime
均不在范围内。

## 验收

- 符合条件的 merge 被识别为明确的 `awaiting_merge_close` 过渡，不产生错误的
  `missing_terminal_decision` finding。
- 后续没有 close 的普通默认分支提交仍然 fail-closed。
- 直接提交、格式错误或无关的 merge、历史未关闭 Work Item、缺少 parity，以及缺失/矛盾的
  GitHub context 仍然阻断。
- 决定仅由 Git 历史和标准的不可变 GitHub context 确定，不引入 bypass flag 或进程级当前仓库。
- 回归测试和三语文档描述相同的有界过渡，并保留最终 finalize/close 要求。
- 在 merge、close 和精确清理 branch/worktree 前，完成已安装 Runtime 验证和可见的人类 Outcome。

## 验证记录

回归套件构造一个新增归档 Work Item 的 reviewed merge，验证允许的过渡，然后追加一次普通
提交并验证同一个未关闭 Work Item 会阻断质量门。现有负向 fixture 仍然保留在 gate 测试集中。

GitHub workflow 继承 `GITHUB_EVENT_NAME`、`GITHUB_REF`、`GITHUB_SHA`；这些只用于识别事件，
不能替代 Contract、evidence、PR 或 close 校验。
