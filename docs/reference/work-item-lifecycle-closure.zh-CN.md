---
author: AI Cockpit maintainers
title: Work Item 生命周期关闭
description: 在归档、合并和精确清理后安全关闭经过评审的 Work Item。
audience: [adopter, maintainer, reviewer]
status: implemented
authority: translation
canonical: docs/reference/work-item-lifecycle-closure.md
lastVerifiedBy: WI-379-reference-documentation-batch-18
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
