---
author: AI Cockpit maintainers
workItemId: WI-126-status-outcome
title: 只读 Work Item 状态与面向人交接投影
description: 让 CLI 与 MCP 共享一份证据绑定的状态和 Outcome 投影。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: documentation-acceptance
---

# WI-126 — 只读状态与面向人交接

本 Work Item 增加请求级 `work-item status` 投影，并让 CLI/MCP 的面向人
Outcome 使用同一份校验后的来源。它不创建 scheduler、全局 current project
或第二套治理决定引擎。

已交付：生命周期、治理状态、活动健康、事实计数、阻塞项、evidence、风险、权限、
未知项、诊断和 source digest；CLI/MCP 首行直接输出 `Outcome: 🔴/🟡/🟢`；
Contract 原文按字节保留并明确标注；历史、缺失、过期、格式错误、foreign 和 symlink
证据保持非绿色且不被重写；三语 Contract 字段映射和 reference parity baseline 纳入 WI-125。

最终二十维 aggregator 与外部 assurance 仍是后续边界。Status 和 Outcome 是只读投影，
不是授权来源。
