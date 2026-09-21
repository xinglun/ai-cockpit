---
author: AI Cockpit maintainers
title: "WI-958 — v0.2.103 Runtime 发布"
description: "仅在受治理的验证、精确清理和隔离产物验收后发布 v0.2.103。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:sei-rinn
workItemId: WI-958-v0-2-103-release
lastVerifiedBy: WI-958-v0-2-103-release
---

[English](WI-958-v0-2-103-release.md) · [日本語](WI-958-v0-2-103-release.ja.md)

# WI-958 — v0.2.103 Runtime 发布

本 Work Item 从已验证的 main 准备下一版 Runtime。发布是最后一步：必须先
绑定不可变发布身份、公开产物、隔离的安装与升级验收以及精确资源清理；随后才向
Sentinel #902 的所有者发送仅含事实的通知，请其确认下游 replay。本工作不修改
Sentinel，也不把第三方宿主中的对话展示表述为已确认。
