---
workItemId: WI-531-historical-direct-merge-apply
title: "WI-531——bundled historical direct-merge 应用"
status: implemented
mode: code
author: AI Cockpit maintainers
description: "将真实 bundled merge parent 与不可变 Contract base 作为两个可审计事实分别绑定。"
audience:
  - maintainer
  - adopter
authority: canonical
lastVerifiedBy: WI-531-historical-direct-merge-apply
terminalArchive: .ai/work-items/archive/WI-531-historical-direct-merge-apply.contract.json
terminalVerification: .ai/evidence/WI-531-historical-direct-merge-apply.verification.json
terminalFinalization: .ai/decisions/WI-531-historical-direct-merge-apply.finalize.json
terminalDecision: .ai/decisions/WI-531-historical-direct-merge-apply.close.json
---

# WI-531——bundled historical direct-merge 应用

## 目标与边界

让发布版 Runtime 支持没有 Pull Request 的历史 bundled merge。新增可选的
`historical.contractBaseRevision`，使 `pullRequest.baseRevision` 继续绑定真实
merge commit 第一 parent；同时报告 resource-context 的具体不匹配类别，并证明
只读 plan 可以在不改写历史 bytes 的情况下完成。不得修改对象工程或虚构 PR。

## 验收

- 即使 bundled merge parent 不同于 Contract base，完整 plan receipt 也能作为第一条
  canonical direct-merge record 被接受。
- 缺失或外部的 Contract base、context、repository、Work Item、Git parent、Runtime
  事实仍然 fail-closed，并指出可操作字段。
- 英文、中文、日文命令文档说明确定性事实与人工填写字段的边界。
- Protocol/repository 测试覆盖保留旧 context、生成 historical context、bundled base 漂移、
  malformed 输入和拒绝时不写入。

## 兼容性

新字段可选，默认不存在；当 merge base 已等于 Contract base 时，旧 receipt 仍可读取。
只有 `direct_merge_no_pr`/`historical_low` receipt 绑定准确的归档 Contract digest 且显式
携带 Contract base 时，才允许两者不一致。

## 对象工程交接

`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator` 对本 Work Item 只读。
发布后由对象工程团队重新运行 `finalize-recovery-plan --merge-commit <sha>`，保留两个 base
字段，只应用生成的 receipt。若仍出现 `resourceContext.<field>`，请反馈该字段，不要手改 `.ai/`。
