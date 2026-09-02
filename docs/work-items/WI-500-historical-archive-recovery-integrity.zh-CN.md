---
author: AI Cockpit 维护者
title: "WI-500——历史归档完整性恢复"
description: "为可选报告字节与清单不一致的不可变历史归档提供有界、可审计的恢复路径。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-500-historical-archive-recovery-integrity
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-500-historical-archive-recovery-integrity
canonical: docs/work-items/WI-500-historical-archive-recovery-integrity.zh-CN.md
---

# WI-500——历史归档完整性恢复

[English](WI-500-historical-archive-recovery-integrity.md) · [日本語](WI-500-historical-archive-recovery-integrity.ja.md)

## 边界

本 Work Item 为可选 `taskReportMarkdown` 字节与记录的 manifest digest
不一致的不可变历史归档增加有界、fail-closed 的恢复路径。必需的身份、
Contract、Summary、Outcome 及其他 artifact 绑定仍保持严格校验，绝不重写
前置项字节。

## 交付状态

实现已在专用分支归档并完成验证。评审 PR 合并并记录精确资源清理前，provider
finalization 与 close 仍待完成。
