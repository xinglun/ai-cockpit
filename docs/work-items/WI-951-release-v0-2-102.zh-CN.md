---
author: AI Cockpit maintainers
workItemId: WI-951-release-v0-2-102
title: 清理完成及 verification snapshot lifecycle 修复后的 v0.2.102 发布
description: 仅在已审查源码、不可变公开制品、采用者验收与精确资源清理均有证据后发布。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-951-release-v0-2-102
---

[English](WI-951-release-v0-2-102.md) · [日本語](WI-951-release-v0-2-102.ja.md)

# WI-951 — v0.2.102 发布

本 WI 在已审查的直接合并 finalization 修复及仓库精确清理后发布 Runtime。只有不可变 tag、公开 Release、下载制品校验、全新安装、升级和 provider 清理证据完整时，才可声明发布成功。

## 边界

- 发布包含下游 acceptance replay 所需的 verification snapshot lifecycle 修复。
- 不修改 Sentinel 或其他采用者仓库。
- 没有直接证据时，不宣称性能已提升或第三方聊天窗口已展示 Outcome。
