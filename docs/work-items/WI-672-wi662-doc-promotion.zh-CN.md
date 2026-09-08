---
author: AI Cockpit maintainers
title: "WI-672——WI-662 终态文档晋级"
description: "将已关闭 WI-662 P0 可信性能基准证据晋级到受治理的三语文档投影。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-672-wi662-doc-promotion
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-672-wi662-doc-promotion
---

[English](WI-672-wi662-doc-promotion.md) · [日本語](WI-672-wi662-doc-promotion.ja.md)

# WI-672——WI-662 终态文档晋级

## 意图

将三语 WI-662 Work Item 页面和 reference-parity 行与已完成 P0 基准 Work Item
的不可变 archive、verification、finalization 和 close 记录同步。

## 边界

这是仅文档的投影变更。唯一目标是六个 WI-662 Markdown 投影文件和三个 WI-672
自注册页面。不可变 `.ai` 生命周期记录只读；不改变 Runtime 行为、基准证据、历史
记录或其他 agent 的 worktree。

## 授权记录

2026-09-08，人类通过 `user-request` 明确授权在 provider 权限中断后继续受治理的
工作。授权记录保存在 PR 评论和 WI-662 close receipt 中；这不伪造 GitHub review。
本 Contract 保留该边界，供未来 provider 或生命周期中断使用。

## 证据与验收

- WI-662 页面和 parity 行在验证关闭后显示终态 `已实现`，并绑定精确终态证据路径。
- P0 证据仍是测量结果和限制的来源；继续保留
  `user_visible_benefit_not_declared`，本次文档投影不宣称性能收益。
- 文档、parity、状态一致性、治理和 closed-work-item 晋级检查在精确评审 head
  上通过。

WI-662 终态证据：

- archive：`.ai/work-items/archive/WI-662-p0-benchmark-evidence.contract.json`
- verification：`.ai/evidence/WI-662-p0-benchmark-evidence.verification.json`
- finalization：`.ai/decisions/WI-662-p0-benchmark-evidence.finalize.json`
- close：`.ai/decisions/WI-662-p0-benchmark-evidence.close.json`
