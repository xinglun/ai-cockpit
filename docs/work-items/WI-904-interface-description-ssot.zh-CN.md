---
author: AI Cockpit maintainers
title: "WI-904 — Work-item outcome 接口描述单一事实源"
description: "从协议侧单一描述生成 CLI、MCP 和参考资料中的接口事实。"
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
workItemId: WI-904-interface-description-ssot
lastVerifiedBy: WI-904-interface-description-ssot
---

[English](WI-904-interface-description-ssot.md) · [日本語](WI-904-interface-description-ssot.ja.md)

# WI-904 — Work-item outcome 接口描述单一事实源

本 Work Item 移除 work-item outcome 发现路径中的重复接口事实。协议侧
描述作为 CLI、MCP 和标记参考资料区域的来源；参考页面中的人工解释仍由
人工维护。

## 验收

- CLI 解析、MCP 校验和生成的参考事实与协议侧描述一致。
- 发现路径确定且不读取仓库状态、不启动验证进程。
- Outcome 交付、授权和生命周期行为保持不变。
