---
author: AI Cockpit maintainers
title: "WI-769——性能终态文档修正"
description: "修正三语性能 Work Item 报告中过期的终态表述，不改变证据或 Runtime 行为。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-769-performance-terminal-docs
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-769-performance-terminal-docs
---

[English](WI-769-performance-terminal-docs.md) · [日本語](WI-769-performance-terminal-docs.ja.md)

# WI-769——性能终态文档修正

## 意图与边界

本仅文档 Work Item 修正 WI-685、WI-692 和 WI-702 三语报告中面向人的过期终态表述。
不改变 Runtime 行为、性能测量、证据 bytes、治理规则、授权或其他 agent 的 Work Item。
WI-702 仍是已恢复的前置项；其 append-only recovery closure 由 WI-712 拥有并记录。

## 修正内容

- WI-685 现在说明其已记录的 `closed` 状态，并保留有界的请求内测量、不可用资源指标和显式的
  `user_visible_benefit_not_declared` unknown。
- WI-692 现在说明其已记录的 `closed` 状态，并保留因测量路径没有生产 caller 而不接入协调器的
  有证据决定；不声称生产性能收益。
- WI-702 现在说明它是没有独立绿色终态的历史前置记录，并将已完成的恢复边界指向 WI-712；
  前置 bytes 保持不可变，也不声称性能收益。

英文、简体中文和日文页面表达相同事实并保留现有证据链接。Parity projection 仅加入当前文档
Work Item，不重写前置记录。

## 验证边界

验收要求已声明的文档 acceptance、parity、status consistency、Runtime verification、评审后的
PR、archive、finalization、close 和 close 后 promotion 检查通过，并证明只有声明范围内的文档
projection 与 Runtime 生成的 WI-769 生命周期记录发生变化。任何不可用指标仍保持不可用；本 WI
不创建性能或面向用户的收益声明。

## 当前状态

Runtime 生命周期与终态证据记录在 WI-769 Contract 下。在评审后的 PR 和 close 完成前，本页保持
`in_progress`。
