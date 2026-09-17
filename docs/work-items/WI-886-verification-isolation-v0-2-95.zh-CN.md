---
author: AI Cockpit maintainers
workItemId: WI-886-verification-isolation-v0-2-95
title: 修复发布验收隔离并发布 v0.2.95
description: 保留显式验证目标隔离，并在不可变的 v0.2.94 验收失败后证明候选版本和下载制品验收。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
predecessorWorkItemId: WI-883-release-v0-2-94-current-main
lastVerifiedBy: WI-886-verification-isolation-v0-2-95
---

# WI-886 — 修复发布验收隔离并发布 v0.2.95

本 successor 保留不可变的 v0.2.94 标签及其分阶段验收失败证据。修复验证环境边界，使调用者提供的隔离 `CARGO_TARGET_DIR` 生效，同时保持文档中的 HOME 回退路径和 `CARGO_INCREMENTAL=0` 不变。v0.2.95 只有在评审合并、候选验收和下载制品安装/升级验收完成后才是最终发布步骤。

结果必须分别报告隔离、清理、验证、发布身份和剩余未知项；不能把候选版本或源码检出误报为公开发布成功。
