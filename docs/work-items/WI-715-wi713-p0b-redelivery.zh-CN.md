---
author: AI Cockpit maintainers
title: "WI-715——WI-713 P0-B 唯一编号重新交付"
description: "从最新默认分支重新交付确定性的 Outcome 摘要和明确的完整证据查看，并使用唯一 Work Item 标识。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-715-wi713-p0b-redelivery
lastVerifiedBy: WI-715-wi713-p0b-redelivery
---

[English](WI-715-wi713-p0b-redelivery.md) · [日本語](WI-715-wi713-p0b-redelivery.ja.md)

# WI-715——WI-713 P0-B 唯一编号重新交付

## 意图

由于 WI-714 已被另一条已关闭 scope 使用，确定性的已关闭 Work Item 晋级无法唯一
匹配，因此从当前远程默认分支重新交付 P0-B。默认人工交接包含四个确定性部分：
结果、关键变化、剩余不确定性、人的下一步；完整证据报告仍可显式查看。

## 边界

本 Work Item 只改变展示层，保持机器 JSON、验证规则、授权语义、退出码、持久化布局
和历史证据不变。CLI 与 MCP 共用同一份 repository renderer 事实，不宣称已测量的
认知收益或真实用户研究。

## 基线与恢复链

- 当前远程/默认基线：`origin/main`，提交 `7ada6cd0928a877fe2bc719689abfe99bd532d7e`。
- 前置项：WI-714，作为已恢复前置项保留，其恢复记录保持不可变。
- 前置恢复决定：`.ai/decisions/WI-714-wi713-current-base-revalidation.recovery.3efe3143da8d84cb32db0877de59ee702b12034925d300b895f5449cb756d676.json`。
- 历史证据：`.ai/evidence/WI-714-wi713-current-base-revalidation.verification.json`。

## 验收

- CLI/MCP 默认人工输出采用四段式读者优先摘要。
- `view: full` / `--view full` 保留审计所需的完整报告。
- 阻断项、人工决定、过期/无效证据和不确定性保持可见；只有非关键列表可以摘要化，
  且必须给出完整报告入口。
- 测试使用真实 Outcome 结构，覆盖多语言和历史状态；机器 JSON 不变。

## 当前状态

唯一 successor 已从当前默认基线启动。验证、托管交付、provider finalization 和明确的
人工 close 决定仍待完成。
