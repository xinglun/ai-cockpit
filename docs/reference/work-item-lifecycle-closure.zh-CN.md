---
author: AI Cockpit maintainers
title: Work Item 生命周期关闭
description: 在归档、合并和精确清理后安全关闭经过评审的 Work Item。
audience: [adopter, maintainer, reviewer]
status: implemented
authority: translation
canonical: docs/reference/work-item-lifecycle-closure.md
lastVerifiedBy: WI-512-reference-docs-batch-33
---

# Work Item 生命周期关闭

[English](work-item-lifecycle-closure.md) · [简体中文](work-item-lifecycle-closure.zh-CN.md) · [日本語](work-item-lifecycle-closure.ja.md)

关闭是 `start → preflight → checkpoint → verify → finish → archive` 之后的最终交接，
不是删除分支的捷径。Runtime 必须证明经过评审的 PR、精确 Work Item head、归档 Contract/
Summary/evidence、已同步 base、干净 worktree 和远端分支不存在。

## 正常路径

```text
verify → finish/archive → push → reviewed PR 与 hosted checks → merge
→ finalize → finalize-verify → close → synchronize and clean
```

关闭会按顺序验证 PR 状态、branch/head identity、base fast-forward、archive/decision
receipt、干净 worktree 和远端分支不存在，之后才删除精确的本地 Work Item 分支。不能让
provider 自动删除分支以绕过证明。

`ready_on_base` 表示调用 checkout 干净且位于已同步默认分支；
`closed_but_current_worktree_detached` 表示已关闭但 base 由另一个已验证 worktree 持有，
应转到命令打印的 base worktree，不能把 detached checkout 当作 ready。

## 恢复与历史边界

任何缺失、过期、外部或矛盾事实都会 fail closed，并保留重试身份。provider 异常或 stacked-
PR 恢复使用独立显式 receipt，不改写不可变 archive，也不把 open PR 变成 merged。参考源的
`make` 命令和 Python 编排不是 Rust Runtime 命令；Rust 通过显式 `--repo` 和 repository-
local evidence 保留同一审查、归档、精确清理意图。

### 历史归档隔离

经过明确授权的 `supersede` 恢复可以关闭不可变的历史归档 predecessor，前提是旧版
Task Outcome Markdown 等可选产物与归档 manifest 摘要不一致。恢复 receipt 必须通过
`predecessorArchiveManifestDigest` 绑定归档 manifest 的精确字节。Runtime 不会改写产物或
manifest；close receipt 会记录 `historical_low` 的 `historicalArchiveIntegrity`，并将该
历史 Work Item 投影为黄色，而不是当前绿色验证。Contract/Summary/Outcome 必需字节、身份、
events 以及其他所有产物完整性检查仍然 fail closed。manifest 缺失、外部、格式错误、符号
链接或摘要不一致时，不能使用此隔离路径。

## Successor 与历史恢复

被阻塞的 predecessor 不会因为存在 corrective successor 就变成成功。其失败 evidence 保持不可变，并按历史记录投影。`work-item recover` 只接受绑定 identity 的 retry、successor 或 supersede receipt；receipt 必须写明 predecessor、适用时的 successor、repository、归档 Contract/Summary/Outcome digest、authority 和 reason。缺失、过期、外部或无关 receipt 都会 fail closed，不能掩盖无关的 Contract 或 Summary 错误。

旧版 resource-finalization 记录使用独立的只读路径：

```sh
ai-cockpit work-item finalize-recovery-plan --repo <path> --id <id>
ai-cockpit work-item finalize-recovery --repo <path> --id <id> --input <receipt.json>
```

对于没有 provider PR 的历史 direct merge，plan 可以包含真实 merge commit 和 parents。生成的 recovery 会明确标记为 historical/low-assurance，不会编造 PR 号，也不会重写旧 receipt。同样，历史共享 worktree 的 `retained` 收尾记录真实 resource disposition，不会仅为满足新 Runtime 而改成 `deleted`。

如果旧版共享主 worktree receipt 没有显式 `historical` 字段，`finalize-verify` 只有在确认
`provider=local`、Contract/receipt identity、主 checkout 拓扑和仍然保留的干净资源后，才会
执行同样窄化的只读投影；随后 `close` 可以使用这个 `historical_low` 结果。任何事实缺失或
矛盾都必须走显式 recovery plan/receipt；不会复制或改写 archive 或 predecessor bytes。

Provider-only 的 post-archive 或 stacked-PR 异常不是普通关闭捷径。它们需要 provider 提供独立、经人工授权的 append-only evidence 边界；Runtime 仍要求精确的 Work Item、repository、branch/head、archive 和 clean-base 绑定。开放或无法验证的 PR 永远不会被当成 merged，参考工程的 provider 专属 Make/Python recovery 命令也不是 Rust 命令。

最终状态同时是生命周期事实和交接：`ready_on_base` 表示调用 checkout 在已同步默认分支上且干净；`closed_but_current_worktree_detached` 表示关闭在其他 worktree 完成，下一项 Work Item 必须使用命令打印的 base worktree。
