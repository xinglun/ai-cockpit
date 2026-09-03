---
workItemId: WI-527-direct-merge-context
title: "WI-527 — direct-merge recovery context 兼容"
status: implemented
mode: code
author: AI Cockpit maintainers
description: "对保留归档 local resource context 的历史 direct-merge receipt 提供有界兼容。"
audience:
  - maintainer
  - adopter
authority: canonical
lastVerifiedBy: WI-527-direct-merge-context
terminalArchive: .ai/work-items/archive/WI-527-direct-merge-context.contract.json
terminalVerification: .ai/evidence/WI-527-direct-merge-context.verification.json
terminalFinalization: .ai/decisions/WI-527-direct-merge-context.finalize.f7bc389eb8064f2451fb5cbd0bb28785546030040c999d25e65f6e0adb5a7c85.json
terminalDecision: .ai/decisions/WI-527-direct-merge-context.close.json
---

# WI-527 — direct-merge recovery context 兼容

## 意图与边界

让归档 Contract 仍保留原始 local `resourceContext` 的仓库能够使用历史无
PR recovery 路径。Runtime 只在明确类型为 `direct_merge_no_pr`、
`historical_low` 的 receipt 中接受该 context，并继续绑定 repository、Work
Item、branch、worktree、base、真实 merge commit 和 parents。不编造 PR 号，
不修改对象工程。

## 实施

- 协议在窄化的历史场景接受未改变的归档 local context，并拒绝外部
  branch/worktree/base 身份。
- `finalize-recovery-plan` 输出身份一致的 historical context，Agent 无需猜测
  provider 或 URL。
- Rust protocol/repository 回归覆盖未改变 context、转换为 historical context
  两种形式以及真实 Git parent 绑定。

## 验收

公开 Runtime 必须接受诚实的首条 direct-merge 记录，并对 malformed、foreign、
stale、symlink、非祖先输入保持 fail-closed，同时保持历史 bytes 不变。必须通过
定向和 workspace 测试、文档检查及标准生命周期 evidence。

## 对象工程交接

`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator` 在本 WI 中仅作
只读对象。发布后由对象团队重新运行 `finalize-recovery-plan`，只应用公开版本输出的
suggested receipt。
