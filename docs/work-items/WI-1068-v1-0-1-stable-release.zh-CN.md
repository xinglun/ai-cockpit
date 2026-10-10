---
author: AI Cockpit maintainers
workItemId: WI-1068-v1-0-1-stable-release
title: v1.0.1 稳定版发布
description: 基于现有审查版本交付 v1.0.1 稳定版，并完成公开制品和安装验收。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1068-v1-0-1-stable-release
---

[English](WI-1068-v1-0-1-stable-release.md) · [日本語](WI-1068-v1-0-1-stable-release.ja.md)

# WI-1068——v1.0.1 稳定版发布

本 Work Item 将现有 v1.0.1 版本正式交付为稳定版，核验官方制品，并更新本机 CLI/MCP 安装。

## 边界

- 产品源码改动仍限于声明的 16 个路径。三语页面和 parity 行属于必需的 Work Item 治理投影。
- 真实的 v1.0.0 稳定版是 N-1 前序版本，完整 N-1 验收只在 x86_64 Linux 上执行。
- v1.0.1-rc.2 继续作为独立不可变预发布版保留。只有合并且 Runtime 新鲜准入发布后，才创建新的稳定版 tag；不得覆盖既有 tag、Release 或制品。
- 现有发布工作流必须先通过四目标候选安装冒烟和分阶段 N-1 验收，然后才能公开 Release。公开下载/安装验收和 macOS 本机 CLI/MCP 验收在发布后进行。

源码候选正在 PR #1022 中审查。CI、发布、公开验收、本机安装和清理均须等待各阶段的当前证据，不在此处预先声称完成。
