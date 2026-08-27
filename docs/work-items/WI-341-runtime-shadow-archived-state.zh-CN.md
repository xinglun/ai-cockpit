---
author: AI Cockpit maintainers
title: "WI-341——归档 PR 的 Runtime shadow"
workItemId: WI-341-runtime-shadow-archived-state
description: "仅在存在 active Contract 时运行 immutable Runtime shadow，同时保留归档 PR 的普通仓库门禁。"
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-341-runtime-shadow-archived-state
terminalArchive: .ai/work-items/archive/WI-341-runtime-shadow-archived-state.contract.json
terminalVerification: .ai/evidence/WI-341-runtime-shadow-archived-state.verification.json
terminalFinalization: .ai/decisions/WI-341-runtime-shadow-archived-state.finalize.cd2a636790b3f88c1ffc793bfee4a02e4d068f26788080b34472110e69deaf4e.json
terminalDecision: .ai/decisions/WI-341-runtime-shadow-archived-state.close.json
---

# WI-341——归档 PR 的 Runtime shadow

本 Work Item 使 public Runtime shadow 及其制品上传绑定到 active Contract。
因此，`finish` 与 `archive` 之后已没有 active Contract 的归档 PR，仍执行
普通仓库门禁，但不会因缺少 active Contract 被错误拒绝。

变更仅限于 workflow 条件、对应回归断言和同步参考文档；不改变 Runtime
Core、发布制品、adopter 验收或 provider 配置。

验收由 archive Contract 与 verification evidence 记录；已审查的 pull request
已合并，并在 close 之前完成精确的 branch/worktree 清理验证。

[English](WI-341-runtime-shadow-archived-state.md) ·
[日本語](WI-341-runtime-shadow-archived-state.ja.md)
