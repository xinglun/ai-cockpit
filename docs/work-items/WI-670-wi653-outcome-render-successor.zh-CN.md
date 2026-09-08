---
author: AI Cockpit maintainers
title: "WI-670——WI-653 Outcome 展示层 successor"
description: "从最新默认分支重新交付 P1-A Outcome 展示边界，在渲染前组装仓库事实。"
audience: [contributor, maintainer, reviewer]
status: implemented
authority: human:repository-owner
workItemId: WI-670-wi653-outcome-render-successor
lastVerifiedBy: WI-670-wi653-outcome-render-successor
terminalArchive: .ai/work-items/archive/WI-670-wi653-outcome-render-successor.contract.json
terminalVerification: .ai/evidence/WI-670-wi653-outcome-render-successor.verification.json
terminalFinalization: .ai/decisions/WI-670-wi653-outcome-render-successor.finalize.json
terminalDecision: .ai/decisions/WI-670-wi653-outcome-render-successor.close.json
---

[English](WI-670-wi653-outcome-render-successor.md) · [日本語](WI-670-wi653-outcome-render-successor.ja.md)

# WI-670——WI-653 Outcome 展示层 successor

## 意图

原 WI-653 已归档但未合并，本 successor 从最新远程默认分支重新交付有界的
P1-A 架构变更。渲染器只接收组装好的 `OutcomeRenderInput`；仓库观察、人工决定
读取、生命周期状态和归档状态检查仍由 CLI/MCP 共用的组装用例负责。

## 边界

保持当前 Outcome trust 投影、机器 JSON、退出码、授权语义、Contract 原文边界和
治理事实不变。不修改 Outcome 文案或本地化，不建立第二套治理规则，不触碰其他
Work Item 的分支、工作树、归档或证据。当前主分支的 `outcome_report.rs` 集成测试
必须迁移到纯渲染输入边界，因此纳入本次范围。

## 改动与验证

- `outcome_render_input` 及 Runtime 绑定变体只组装一次已验证事实；
  `render_human_outcome` 只负责格式化。
- CLI、MCP、生命周期 handoff、阻断 handoff 和集成 fixture 均使用组装输入。
- 内存单元测试覆盖人工决定缺失、有效、无效、已归档未关闭和 superseded 历史事实，
  不需要仓库目录。
- 精确 successor head 必须通过 `cargo fmt --all -- --check`、严格 Clippy、工作区
  测试、文档验收和治理检查。

## 不在本次范围内

P0 职责图、P1-B 观察上下文重构、P2 生命周期/存储拆分、P2-B 状态类型重构、P2-C
多文件一致性和 P3 物理执行共享仍由独立 Work Item 负责。
