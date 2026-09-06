---
author: AI Cockpit maintainers
title: "WI-602——WI-601 终态文档晋级"
description: "补齐 WI-601 的终态对等文档投影，不改变治理事实。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-602-doc-promotion-wi601
lastVerifiedBy: WI-602-doc-promotion-wi601
---

[English](WI-602-doc-promotion-wi601.md) · [日本語](WI-602-doc-promotion-wi601.ja.md)

# WI-602——WI-601 终态文档晋级

## 目标

补齐已关闭 WI-601 的三语 Parity 行，使已关闭 Work Item 文档检查具有唯一、可
审计的终态投影。

## 边界

本 Work Item 只修改三份 reference-parity 页面及本 Work Item 的三语文档记录。
Runtime 行为、参考源脚手架或 wire 格式、对象工程、全局 Agent/MCP 配置，以及
生成的 evidence/decision 字节均不在范围内。

## 验收

1. 英文、中文、日文 Parity 表各有且只有一行 WI-601 终态记录，并包含不可变
   archive、verification、finalization、close 路径。
2. 三份 WI-602 页面在生成 evidence 前注册，并保留 Contract 的原始语言和人工
   所有边界。
3. 文档、metadata 和已关闭 Work Item 晋级检查通过，且不改变生成的治理事实。
4. 不复制参考源脚手架，不修改对象/adopter 工程。

## 验证

在显式 repository context 下运行文档验收、参考 metadata、已关闭 Work Item 晋级
检查，以及 Contract 声明的 locked workspace 验证命令。
