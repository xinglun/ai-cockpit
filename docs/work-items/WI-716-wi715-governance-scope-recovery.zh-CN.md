---
author: AI Cockpit maintainers
title: "WI-716——WI-715 治理 scope 恢复"
description: "在不改写不可变历史的前提下，完成已合并 WI-715 交付的 scope-aware 治理关闭。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human:repository-owner
workItemId: WI-716-wi715-governance-scope-recovery
lastVerifiedBy: WI-716-wi715-governance-scope-recovery
terminalArchive: .ai/work-items/archive/WI-716-wi715-governance-scope-recovery.contract.json
terminalVerification: .ai/evidence/WI-716-wi715-governance-scope-recovery.verification.json
terminalFinalization: .ai/decisions/WI-716-wi715-governance-scope-recovery.finalize.json
terminalDecision: .ai/decisions/WI-716-wi715-governance-scope-recovery.close.json
---

[English](WI-716-wi715-governance-scope-recovery.md) · [日本語](WI-716-wi715-governance-scope-recovery.ja.md)

# WI-716——WI-715 治理 scope 恢复

## 意图

WI-715 合并 PR #707 后，Runtime 正确发现：原 Contract 已归档后补充的
parity 与 gate 修复触及了原 Contract 未声明的治理测试路径。本 Work Item
只记录有界的 successor 收尾，不重做已经交付的 P0-B 产品实现。

## 边界

本 Work Item 覆盖治理 parity/status 检查对完整 Work Item ID 的处理及其三语
文档投影。不改变 Outcome 行为、验证规则、退出码、机器 JSON、授权语义，
也不改写任何 WI-715 历史 archive/evidence bytes。

## 基线与恢复链路

- 当前远程默认基线：`origin/main`，revision `d1141480fb7a045979098480c3770d06002e2a87`。
- 前置项：WI-715；其已合并交付、finalization receipt 与 verification evidence 保持不可变。
- recovery 决定：`.ai/decisions/WI-715-wi713-p0b-redelivery.recovery.json`。
- 前置 finalization：`.ai/decisions/WI-715-wi713-p0b-redelivery.finalize.json`。
- PR #707：`https://github.com/xinglun/ai-cockpit/pull/707`。

## 验收

- governance gate、文档状态一致性、promotion helper 及其回归测试按完整
  Work Item ID 处理记录，不再把共享数字前缀的无关记录合并。
- 三语 parity 投影保留 WI-715 recovery 事实，并记录 successor 的证据与
  终态决定，不虚构产品收益或用户收益。
- 当前默认基线上的 `cargo test --locked --workspace`、文档验收和仓库治理
  检查通过。
- provider finalization、精确 cleanup、人工 close 以及 WI-715 历史 close
  均由 Runtime 生成证据表达。

## 当前状态

successor 已从合并后的默认基线激活。verification、reviewed delivery、
finalization、archive 与明确人工 close 仍待完成。
