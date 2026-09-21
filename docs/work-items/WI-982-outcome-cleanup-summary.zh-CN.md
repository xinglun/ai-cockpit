---
author: AI Cockpit maintainers
title: "WI-982 — Outcome 清理摘要"
description: "在共享的人类 Outcome 正文中暴露已验证的清理计数、精确资源身份、保留原因、证据引用和下一步动作。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:user
workItemId: WI-982-outcome-cleanup-summary
lastVerifiedBy: WI-982-outcome-cleanup-summary
---

[English](WI-982-outcome-cleanup-summary.md) · [日本語](WI-982-outcome-cleanup-summary.ja.md)

# WI-982 — Outcome 清理摘要

本 Work Item 改善从已验证 resource-finalization receipt 生成的人类可读清理
摘要。共享投影报告已删除、保留和未知的数量，列出 pull request、branch 和
worktree 的精确身份，保留 receipt 绑定的原因、证据引用以及现有可执行下一步。
CLI 与 MCP 继续消费同一份完整正文；未单独提供证据的 provider 显示确认仍保持未知。

旧版 finalization JSON 通过带默认值的新增可选字段保持可读。本 Work Item 不改变
授权、验证、发布或 provider 显示语义。
