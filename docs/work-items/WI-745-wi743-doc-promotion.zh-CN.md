---
author: AI Cockpit maintainers
title: "WI-745——WI-743 终态文档晋级"
description: "晋级已验证的 WI-743 终态文档并修复 hosted documentation-governance 投影。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-745-wi743-doc-promotion
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-745-wi743-doc-promotion
recoveryDecision: .ai/decisions/WI-745-wi743-doc-promotion.recovery.6ef6b1b0e804eaf246518a1839851b47b0de88cb9a18c73282f21fb0816d1e41.json
closeDecision: .ai/decisions/WI-745-wi743-doc-promotion.close.json
---

[English](WI-745-wi743-doc-promotion.md) · [日本語](WI-745-wi743-doc-promotion.ja.md)

# WI-745——WI-743 终态文档晋级

## 目的

晋级已验证的 WI-743 终态文档，并在三语 reference-parity 台账登记本
文档专用 successor。该 successor 修复 hosted documentation-governance
投影，不改变已关闭的 WI-743 记录或其已合并的性能交付。

## 边界

本 Work Item 仅修改六个 WI-743 文档投影，以及 archive 前 hosted gate
要求的三语 WI-745 Work Item/parity 投影。不修改生产代码、测试、治理
规则、WI-743 archive 或 decision 记录、PR #716、WI-744 或 WI-742 历史。

## 证据与生命周期

- 晋级内容的不可变来源是 WI-743 的终态 archive、verification、
  finalization 和 close 记录。
- PR #718 从已同步的 `origin/main` 基线交付本纯文档 successor。
- 生命周期为 `start → preflight → checkpoint → verify → finish →
  archive → close`；WI-745 已按 Runtime `supersede` 作为历史前置项关闭，后续归属
  WI-746。本页面不声明新的性能或用户可见收益。
- 当前 supersede 记录为 `.ai/decisions/WI-745-wi743-doc-promotion.recovery.6ef6b1b0e804eaf246518a1839851b47b0de88cb9a18c73282f21fb0816d1e41.json`，
  close 决策为 `.ai/decisions/WI-745-wi743-doc-promotion.close.json`。

## 验收

晋级 helper、文档验收、parity 状态检查和 Work Item 状态一致性检查必须
通过；不得改写历史证据，也不得引入生产行为或治理决定。
