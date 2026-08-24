---
author: AI Cockpit maintainers
title: "WI-196——治理完整性恢复质量门重试"
description: "从全新 checkpoint 重新验证当前批次的恢复质量门与发布验收隔离。"
audience:
  - maintainer
  - reviewer
workItemId: WI-196-governance-recovery-gate-retry
status: recovered
authority: canonical
lastVerifiedBy: WI-196-governance-recovery-gate-retry
---

# WI-196——治理完整性恢复质量门重试

WI-196 是 WI-195 在 finish 后发现同范围修正后的显式 successor。它保持同一有界范围，
建立全新的 checkpoint，并重新执行恢复感知治理质量门、文档验收和公开 adopter 隔离回归。
前置 Work Item 保持为 recovered 历史，其 evidence 不会被当作当前验证复用。

本 Work Item 完成 review、合并、close，并使用修正后的不可变 public artifact 完成 Release
验收后，下一批才开始参考源工程的逐文件对比。

[English](WI-196-governance-recovery-gate-retry.md) ·
[日本語](WI-196-governance-recovery-gate-retry.ja.md)
