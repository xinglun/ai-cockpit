---
author: AI Cockpit maintainers
title: "WI-676——WI-675 文档状态修复"
description: "修复 WI-675 的终态文档投影，不改变 Runtime 行为或历史证据。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-676-wi675-doc-repair
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-676-wi675-doc-repair
---

[English](WI-676-wi675-doc-repair.md) · [日本語](WI-676-wi675-doc-repair.ja.md)

# WI-676——WI-675 文档状态修复

## 意图

修复阻塞仓库治理门禁的 WI-675 终态文档投影。不可变的 WI-675 archive、
verification、finalization 和 close 记录仍是唯一生命周期权威。

## 边界

这是仅限文档的纠偏 Work Item。变更限定为三个 WI-675 语言页面、三个
reference-parity 行和三个 WI-676 自注册页面。不修改生产代码、测试、治理规则
或历史 `.ai` 记录。

## 授权记录

2026-09-08，人类通过 `user-request` 明确授权在 provider 或生命周期中断后继续
受治理的工作。准确中断、范围和证据保存在 Contract 与 PR 中；这不伪造 GitHub
review。

## 验收与生命周期

- WI-675 页面和 parity 行显示由不可变 archive、verification、finalization 和
  close 记录支持的终态 `已实现`。
- WI-676 页面及预归档 parity 注册保持明确且绑定证据。
- 遵循 `start → preflight → checkpoint → verify → finish → archive → close`；
  保留 `user_visible_benefit_not_declared`。
- 精确审查 head 上的文档、parity、状态一致性和 closed-work-item 晋级检查通过。

## 证据

- archive：`.ai/work-items/archive/WI-675-wi670-doc-promotion.archive.json`
- verification：`.ai/evidence/WI-675-wi670-doc-promotion.verification.json`
- finalization：`.ai/decisions/WI-675-wi670-doc-promotion.finalize.json`
- close：`.ai/decisions/WI-675-wi670-doc-promotion.close.json`

