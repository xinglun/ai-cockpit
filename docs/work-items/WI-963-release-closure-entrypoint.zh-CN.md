---
author: AI Cockpit maintainers
title: "WI-963 — 发布收尾入口"
description: "修复发布收尾 Work Item 的狭窄、fail-closed 入口 bootstrap 路径。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:sei-rinn
workItemId: WI-963-release-closure-entrypoint
lastVerifiedBy: WI-963-release-closure-entrypoint
---

[English](WI-963-release-closure-entrypoint.md) · [日本語](WI-963-release-closure-entrypoint.ja.md)

# WI-963 — 发布收尾入口

此后继将 WI-962 收窄为 fail-closed 的入口修正：当前 Work Item 的读者文档投影不能
阻断其第一个 checkpoint。授权、证据、finalization、archive 和 close 规则保持不变；
后续文档边界仍然可执行。
