---
author: Ray
title: "WI-353——Runtime recovery delivery 绑定"
workItemId: WI-353-runtime-recovery-delivery-binding
description: "在保持 WI-351 不可变历史的前提下，将恢复后的 Runtime 交付绑定到真实 reviewed PR。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: translation
canonical: docs/work-items/WI-353-runtime-recovery-delivery-binding.md
lastVerifiedBy: WI-353-runtime-recovery-delivery-binding
predecessor: WI-351-runtime-recovery-binding
terminalArchive: .ai/work-items/archive/WI-353-runtime-recovery-delivery-binding.archive.json
terminalVerification: .ai/evidence/WI-353-runtime-recovery-delivery-binding.verification.json
capabilityClaims:
  - recovery_delivery_binding
---

# WI-353——Runtime recovery delivery 绑定

[English](WI-353-runtime-recovery-delivery-binding.md) · [日本語](WI-353-runtime-recovery-delivery-binding.ja.md)

## 意图与边界

本 successor Work Item 保留 WI-351 的不可变 archive，并将恢复后的 Runtime 交付绑定到真实
reviewed GitHub PR #318。它在 finalization 前记录精确的 `main`/`origin` base、branch、
worktree 与 Runtime 自身的 evidence。

范围仅包括 recovery binding、fail-closed 回归覆盖，以及交付该修复所需的治理记录。Sentinel
业务代码、Provider 发现、交易决策、gate、execution、position sizing、全局配置，以及对
WI-351 历史的任何改写，都不在范围内。

## 验证与交付边界

- successor archive 前必须通过 locked workspace tests、formatting check 与 clippy。
- PR resource context 绑定到 [PR #318](https://github.com/xinglun/ai-cockpit/pull/318)，其 base
  为 `main`/`origin`，并使用专用 recovery worktree。
- Provider finalization、精确 branch/worktree 清理和结构化 close 必须等 reviewed PR merge
  后进行；merge 前不得报告为已完成。

前置 Work Item 的 archive 与 evidence 保持不可变；本 successor 承担交付与 finalization
边界，不改写前置 bytes。
