---
author: AI Cockpit maintainers
workItemId: WI-1037-task8-pr-lifecycle-gate
title: Task 8 PR 生命周期门禁修正
description: 仅修复阻碍 Task 8 收尾的 PR 生命周期门禁；合并并清理 Task 8 后进入 Task 9，在发布前停止。
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1037-task8-pr-lifecycle-gate
---

[English](WI-1037-task8-pr-lifecycle-gate.md) · [日本語](WI-1037-task8-pr-lifecycle-gate.ja.md)

# WI-1037 — Task 8 PR 生命周期门禁修正

此有界后继任务处理 [PR #997](https://github.com/xinglun/ai-cockpit/pull/997) 的 [CI run 36377627838](https://github.com/xinglun/ai-cockpit/actions/runs/36377627838) 所记录的合并前生命周期失败：普通 Work Item 必须等其 PR 合并后才能正式关闭，因此 CI 只能识别确实引入该归档的精确 PR 合并 checkout。

## 边界

- 仅当 `pull_request` merge ref、事件 base/head 与两个 Git 父提交完全对应，且该 diff 新增归档 Contract 时才允许等待正式关闭。
- 旧归档、身份不匹配或畸形事件、直接 push，以及后续仍未关闭的提交继续阻塞。
- 保留合并后的正式关闭要求；不得在集成前创建 close receipt。
- 不包含泛化治理重设计、Task 9 迁移、release、版本变更、tag 或发布。

## 验收

1. 精确 PR 新增的归档可以进入 `awaiting_merge_close`；事件与 Git 身份必须匹配。
2. 旧的未关闭归档以及不匹配或畸形的 PR 事件继续阻塞。
3. 临时窗口仅覆盖精确的默认分支合并；之后没有 close 的非合并提交必须阻塞。
4. 三语 WI-1036 parity 行指向其归档 Contract，并准确标为已归档、等待合并，不宣称完成。
5. 集成前聚焦检查和精确 head 的 hosted CI 通过。合并并清理 Task 8 后，仅当 Runtime 报告 `readyOnBase` 才开始 Task 9；发布前停止交由人工 review。

当前状态为进行中。必需场景、本地检查、hosted 检查和集成都待完成；本页不声称已通过。
