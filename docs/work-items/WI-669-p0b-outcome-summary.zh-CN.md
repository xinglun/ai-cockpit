---
author: AI Cockpit maintainers
title: "WI-669——P0-B Outcome 摘要与完整证据查看"
description: "为 CLI 与 MCP 提供确定性的读者优先 Outcome 摘要，并保留明确的完整证据入口。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-669-p0b-outcome-summary
lastVerifiedBy: WI-669-p0b-outcome-summary
---

[English](WI-669-p0b-outcome-summary.md) · [日本語](WI-669-p0b-outcome-summary.ja.md)

# WI-669——P0-B Outcome 摘要与完整证据查看

## 意图

降低默认人工 Outcome 的阅读成本，同时保持验证、生命周期、人工决定、
证据和不确定性的校准边界。摘要由 CLI 与 MCP 共用的确定性规则生成，
完整审计视图仍可显式查看。

## 边界

本 Work Item 只改变展示层。机器 JSON、验证规则、授权语义、退出码、持久化
布局和历史证据保持不变；不宣称已测量的认知收益或真实用户研究结果。

## 证据

- Archive：`.ai/work-items/archive/WI-669-p0b-outcome-summary.contract.json`
- Verification：`.ai/evidence/WI-669-p0b-outcome-summary.verification.json`
- Hosted 交付：[PR #668](https://github.com/xinglun/ai-cockpit/pull/668)

## 当前状态

在 reviewed PR、provider finalization 和明确的人工 close 决定完成前，
该 Work Item 仍保持进行中。
