---
author: AI Cockpit maintainers
title: "WI-433——本地参考源绑定"
workItemId: WI-433-local-reference-source-binding
description: "将参考对比绑定到维护者管理的本地 checkout 与固定提交，不提供联网回退或源代码复制。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-433-local-reference-source-binding
---

# WI-433——本地参考源绑定

当前语义对比唯一使用 `AI_COCKPIT_REFERENCE_ROOT` 指向的本地 checkout，
精确提交记录在 `tests/conformance/reference-source.lock`。缺失、脏或提交
漂移都会 fail-closed；托管 CI 使用离线 corpus，不获取公开参考仓库。

历史 inventory 保持不可变，不会被静默重新建立。本 Work Item 是语义对齐与
治理边界文档，不复制参考 Runtime、Python 模块、Make 规则或工具链。

[English](WI-433-local-reference-source-binding.md) · [日本語](WI-433-local-reference-source-binding.ja.md)
