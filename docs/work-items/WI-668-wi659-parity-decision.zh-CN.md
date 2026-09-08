---
author: AI Cockpit maintainers
title: "WI-668——WI-659 parity 决定链接修复"
description: "在三份 reference-parity ledger 中投影 WI-659 的精确 versioned supersede 决定，不改变历史记录。"
workItemId: WI-668-wi659-parity-decision
audience: [maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-668-wi659-parity-decision
capabilityClaims: [governance_integrity, reference_parity]
---

# WI-668——WI-659 parity 决定链接修复

[English](WI-668-wi659-parity-decision.md) · [日本語](WI-668-wi659-parity-decision.ja.md)

## 意图与边界

已关闭的 WI-659 predecessor 同时具有 canonical successor recovery 和按
digest 版本化的 supersede recovery。三份 reference-parity 投影必须展示精确的
terminal supersede 决定，治理门禁才能验证这条历史链。本 Work Item 只修改文档
链接；WI-659 的 archive、evidence、recovery 和 close 记录保持不可变。

## 范围

- 在三份 reference-parity ledger 的 WI-659 行加入精确的 versioned
  supersede 决定路径和 superseded close 路径。
- 保持英文、简体中文和日文投影语义一致。
- 添加本 Work Item 的三语文档对应页。

## 范围外

WI-659 的 archive、verification、recovery、close 或任何其他生成的 `.ai`
记录；WI-664/WI-665 文档；Runtime 或 Rust 代码；schema；测试；发布产物；以及
全局 Agent/MCP 配置。

## 验收

- 三份 WI-659 parity 行保留 archive、verification 和 canonical recovery 引用，
  并加入精确的 versioned supersede recovery 与 superseded close 决定路径。
- 治理完整性门禁不再报告 WI-659 的 `missing_parity_decision`。
- WI-659 的 archive、evidence、recovery 和 close 字节保持不变。

## 验证与终态记录

使用带明确 `--repo` 的已安装 Runtime、聚焦 parity 与文档检查、`git diff --check`
及仓库 hosted checks。评审合并后，记录 Runtime 声明的 archive、verification、
finalization 和 close 路径。
