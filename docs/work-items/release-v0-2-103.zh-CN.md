---
author: AI Cockpit maintainers
workItemId: release-v0-2-103
title: verification receipt 与发布分发策略修复后的 v0.2.103 发布
description: 仅在已审查源码、不可变公开制品、采用者验收与精确资源清理均有证据后发布。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: release-v0-2-103
---

[English](release-v0-2-103.md) · [日本語](release-v0-2-103.ja.md)

# v0.2.103 发布

本 WI 准备发布包含有界 verification receipt 处理与仅 Apple Silicon Homebrew 分发策略的 Runtime。只有已审查合并、不可变 tag、公开 Release、下载制品检查、全新安装、升级和 provider 清理证据完整时，才可声明发布成功。

## 边界

- 发布包含 Sentinel 为 Issue #902 重放验收前所需的 Runtime 侧修复。
- Intel Homebrew 不再是受支持的分发或验收目标；独立 x86_64 macOS 压缩包仍受支持。
- 不修改 Sentinel 或其他采用者仓库。
- 没有直接证据时，不宣称性能已提升或第三方聊天窗口已展示 Outcome。
