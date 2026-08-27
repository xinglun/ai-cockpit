---
author: AI Cockpit maintainers
title: "WI-335——Provider finalization 修正"
workItemId: WI-335-provider-finalization-correction
description: "在验证前绑定真实 reviewed provider 身份，重新交付 WI-334 的有界 evidence-parity 文档。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-335-provider-finalization-correction
---

# WI-335——Provider finalization 修正

WI-334 作为不可变历史保留。其归档 Contract 在真实 PR 身份确定前记录了
占位 PR URL。本 successor 不改写前驱，也不增加 Runtime 行为；它记录恢复
关联，并在验证前绑定真实 provider PR，重新交付同一有界的 evidence-parity
文档。

## 边界

- 保留 WI-334 全部 archive、evidence 和 recovery bytes。
- 只在 reviewed PR 存在后记录 WI-335 provider context。
- 重新执行安装版 Runtime lifecycle 与托管检查。
- 完成准确的 branch/worktree 收尾，以结构化人工决定 close，并只删除准确的合并资源。

Cursor adopter 反馈仍是外部验证输入。稳定 stdout JSON、可重放 Outcome、
lifecycle 入口门禁和 verification 失效检查已经属于 Runtime 边界；IDE 聊天
自动发布仍由宿主 Adapter 负责。

## 验收

1. WI-334 前驱 bytes 与 repository identity 保持不变。
2. 三语 parity ledger 记录本次恢复，不使用猜测的 PR URL，并在创建后链接真实 provider PR。
3. active Contract 在验证前绑定真实 PR，所有 finalization receipt 与安装版 Runtime 和 repository 一致。
4. 托管检查与完整 lifecycle 产出可审计证据，随后清理准确 branch/worktree。

[English](WI-335-provider-finalization-correction.md) · [日本語](WI-335-provider-finalization-correction.ja.md)
