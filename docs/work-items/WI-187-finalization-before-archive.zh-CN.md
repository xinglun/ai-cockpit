---
author: AI Cockpit maintainers
title: "WI-187——archive 前完成资源最终化计划"
description: "当前 Work Item 只有在显式绑定非 provisional 的资源最终化计划后才能 archive。"
audience:
  - maintainer
  - reviewer
workItemId: WI-187-finalization-before-archive
status: implemented
authority: canonical
lastVerifiedBy: WI-187-finalization-before-archive
---

# WI-187——archive 前完成资源最终化计划

WI-187 修复 lifecycle 的顺序缺口。`start` 会有意写入 provisional
`resourceContext`：它记录本地 branch 与 worktree 观察值，但
`baseBranch`、`baseRemote`、`provider` 和 `pullRequest` 仍为 `unknown`。
这个 context 不是资源最终化计划。

标准 `finish` 边界现在会在产生 `finish_ready` 之前拒绝缺失或 provisional 的
context；`archive` 也会在移动任何 active Contract、Summary、Outcome、report、
event 或 approach bytes 之前独立复查同一条件。操作者必须在 verification、
finish 与 archive 之前，通过 `work-item finalize-plan` 写入完整、已验证且
identity-bound 的 context。有效的非 provisional plan 仍可沿用既有成功
lifecycle 流程。

## 历史与恢复边界

WI-186 是本缺口的已观察 predecessor：公开 v0.2.23 Runtime 归档其记录时，
Contract 仍保留 `start` 写入的 provisional context。WI-187 不编辑、不规范化，
也不追溯提升这些历史 archive bytes。历史读取继续兼容 optional context；
显式 supersession 恢复路径继续逐 byte 保留 predecessor artifacts。该恢复例外
必须有独立且 identity-bound 的 recovery decision，不能让当前普通 Work Item
绕过 `finalize-plan`。

WI-187 是该已观察缺口的 bounded successor。安装 Runtime 会在
`.ai/decisions/` 下记录 `supersede` recovery receipt，把 WI-186 的精确
Contract、Summary、Outcome 与 events digests 严格绑定到 WI-187。该 receipt
只追加记录：它既不重新解释 WI-186 的结果，也不改写 WI-186 archive bundle
中的任何文件。

WI-187 的第一次执行本身也在该顺序被强制前进入了 `finish_ready`；Runtime
因此正确拒绝在 verification 后替换 provisional plan。这些精确记录会通过
digest-bound supersession 保留，`WI-190-finalization-plan-order` 则按正确顺序
重跑 lifecycle 并继续承载已验证实现。

回归测试覆盖 protocol provisional 判定、repository archive 拒绝与 active bytes
保持、CLI 拒绝与可恢复状态、有效 plan 后成功、历史 evidence 可读性，以及
superseded predecessor 的不可变恢复。本 Work Item 明确不修改共享 reference
parity 文件。

[English](WI-187-finalization-before-archive.md) ·
[日本語](WI-187-finalization-before-archive.ja.md)
