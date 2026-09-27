---
author: AI Cockpit maintainers
workItemId: WI-1035-task8-governance-friction-remediation
title: Task 8 治理摩擦修复
description: 在一个串行 Work Item 中处理完整的 20 项 Task 8 治理摩擦清单，向 Agent 暴露可实际使用的协作能力，并在版本审查边界前验证行为。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1035-task8-governance-friction-remediation
---

[English](WI-1035-task8-governance-friction-remediation.md) · [日本語](WI-1035-task8-governance-friction-remediation.ja.md)

# WI-1035 — Task 8 治理摩擦修复

本 Work Item 承接 WI-1033 handoff，并在同一条实施主线上保留全部 20 项治理摩擦：原 handoff 的 17 项、独立复现的两项关闭投影与交付沟通问题，以及通用验证证据复用流程。权威验收标准、清单、实施阶段和证据计划以 Runtime 绑定的 Contract 及[Task 8 规格](WI-1034-task8-governance-friction-remediation/spec.md)和[实施计划](WI-1034-task8-governance-friction-remediation/implementation-plan.md)为准。

## 边界

- 所有实现串行使用一个 WI、分支、worktree 和仓库上下文；通过多个可审查提交组织，不堆成一个巨大提交。
- Agent 协作能力必须可发现且可实际使用。暴露完成后立即检查 manifest 与 MCP schema，并验收真实多进程、多 linked worktree 协作及单 WI 串行回退。
- 启动任何声明的验证前，先检查 Runtime freshness 与已有 receipt 绑定；复用完整且新鲜的证据，只重跑缺失或失效的检查，并将精确 PR-head hosted 证据与本地 receipt 分开。
- 验证、PR 审查、合并、清理和 Work Item 生命周期彼此分离。本任务停在版本发布之前，不创建 tag 或发布版本。
- 当前状态为进行中。本投影不声称任何验收项或验证结果已完成。

## 验收概要

全部 20 项 Contract 标准都必须关联到当前测试或真实验收证据、canonical CI gate 及 Runtime 绑定证据。缺少证据、候选身份过期或必需场景未验证时，仍视为阻塞。

Hosted 验证统一使用已登记到 gate manifest 的共享入口：按候选 Runtime 当前准入决定是否刷新 preflight，在执行验证前再次核对准入，并从绑定同一可执行文件的正式 receipt 生成 workspace coverage，不重复运行 package 检查。

`verify` 命令的 stdout 只是执行摘要。共享入口会将其单独保存，并把 Runtime 写入 `.ai/evidence/<WI>.verification.json` 的正式 envelope 按原始字节复制为 hosted receipt。Coverage 会验证两层 receipt 身份、完整 Cargo package/node 集合，并要求它与仓库 evidence 字节完全相同；畸形、错配或重新拼装的 receipt 一律 fail closed。

同版本候选 Runtime 重建后，如果旧 receipt 的仓库、Contract、版本及结构/摘要校验均有效、仅可执行文件 digest 改变，则将其标为 stale；它不能复用，也不需要冗余 retry 决策。共享入口遵从候选 Runtime 当前准入的下一动作；Runtime 版本变化或身份/证据无效仍走显式 fail-closed recovery 边界。
