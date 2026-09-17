---
author: AI Cockpit maintainers
workItemId: WI-883-release-v0-2-94-current-main
title: 从当前 main 受治理地发布 v0.2.94
description: 只有在 Outcome、HCI、四方向收敛、Issue #851、接口发现和 Rust 工具链工作在同步 main 上完成验证后才发布 v0.2.94。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
predecessorWorkItemId: WI-871-release-v0-2-94-current-main
lastVerifiedBy: WI-883-release-v0-2-94-current-main
---

# WI-883——从当前 main 受治理地发布 v0.2.94

本 Work Item 是旧基线 WI-870、WI-871 发布尝试被作为不可变历史保留后的当前
main 发布路径。发布是最后一步：归档后的完整 Outcome 交付、HCI 对话事件、
四方向收敛证据、Issue #851 对应、接口发现试点、Rust 1.98.1 与锁定依赖更新、
候选验收、评审合并以及不可变下载制品验收全部完成后，才能修改 provider tag
或公开 Release。

发布必须保留 v0.2.93 作为 N-1 基线。性能收益未知或宿主展示确认未知时必须
保持未知，不能用 CI 通过或报告已生成推断为已完成。
