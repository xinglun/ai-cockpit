---
author: AI Cockpit maintainers
title: "WI-987——WI-986 文档晋级"
description: "在不改写不可变生命周期证据的前提下，晋级已关闭 WI-986 的文档投影。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: authorized
workItemId: WI-987-wi986-doc-promotion
lastVerifiedBy: WI-987-wi986-doc-promotion
---

[English](WI-987-wi986-doc-promotion.md) · [日本語](WI-987-wi986-doc-promotion.ja.md)

# WI-987——WI-986 文档晋级

这是一个有边界的文档 Work Item，用于晋级已关闭 WI-986 的文档投影。
只修改 WI-986 的三语页面和三条 reference-parity 行；不改写不可变的
`.ai` 生命周期证据。

## 验收

- promotion helper 报告 WI-986 没有过时投影。
- 英文、简体中文和日文页面保留一致的 archive、verification 和 close 终态事实。
- 全仓 `--check-all` 文档投影检查通过。
- 不修改源码、发布逻辑或不可变治理 receipt。
