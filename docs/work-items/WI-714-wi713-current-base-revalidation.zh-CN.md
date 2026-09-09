---
author: AI Cockpit maintainers
title: "WI-714——WI-713 P0-B 当前基线重新验证"
description: "从最新默认分支重新交付确定性的 Outcome 摘要和明确的完整证据查看。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: human:repository-owner
workItemId: WI-714-wi713-current-base-revalidation
lastVerifiedBy: WI-714-wi713-current-base-revalidation
---

[English](WI-714-wi713-current-base-revalidation.md) · [日本語](WI-714-wi713-current-base-revalidation.ja.md)

# WI-714——WI-713 P0-B 当前基线重新验证

## 意图

由于 PR #704 在前置项归档期间与推进后的默认基线冲突，从最新远程默认分支
重新交付 P0-B。默认人工交接包含四个确定性部分：结果、关键变化、剩余不确定性、
人的下一步；完整证据报告仍可显式查看。

## 边界

本 Work Item 只改变展示层，保持机器 JSON、验证规则、授权语义、退出码、持久化布局
和历史证据不变。CLI 与 MCP 共用同一份 repository renderer 事实，不宣称已测量的
认知收益或真实用户研究。

## 基线与恢复链

- 历史尝试基线：`origin/main`，提交 `94fec9e43ff0c046a236a100d881b032124e973e`。
- 前置项：WI-713，其 archive 和历史 verification 保持不可变。
- 恢复决定：`.ai/decisions/WI-713-wi703-current-base-revalidation.recovery.09a026473430b0e8855dabacf111b91686189e3de7c009b241c8a03ac886e5cc.json`。
- 历史证据：`.ai/evidence/WI-713-wi703-current-base-revalidation.verification.json`。
- 后继项：WI-715；由于仓库已将 WI-714 用于另一条已关闭 scope，WI-715 负责唯一编号的重新交付。
- WI-714 恢复决定：`.ai/decisions/WI-714-wi713-current-base-revalidation.recovery.3efe3143da8d84cb32db0877de59ee702b12034925d300b895f5449cb756d676.json`。

## 验收

- CLI/MCP 默认人工输出采用四段式读者优先摘要。
- `view: full` / `--view full` 保留审计所需的完整报告。
- 阻断项、人工决定、过期/无效证据和不确定性保持可见；只有非关键列表可以摘要化，
  且必须给出完整报告入口。
- 测试使用真实 Outcome 结构，覆盖多语言和历史状态；机器 JSON 不变。

## 当前状态

本 Work Item 作为已恢复的前置项保留。由于 WI-714 编号冲突，原尝试未在该编号下关闭；
WI-715 负责有界重新交付，且不重写这些历史记录。
