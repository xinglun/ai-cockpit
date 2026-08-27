---
author: AI Cockpit maintainers
title: "WI-338——治理文档基础干净交付"
workItemId: WI-338-reference-docs-foundation-clean
description: "重新交付前五个固定参考源治理文档的比较，并保留 WI-336/WI-337 不可变历史。"
audience: [maintainer, reviewer]
status: recovered
authority: canonical
lastVerifiedBy: WI-338-reference-docs-foundation-clean
---

# WI-338——治理文档基础干净交付

WI-338 是 WI-336 与 WI-337 的显式 successor 链。前两项失败的 lifecycle bytes
保持不可变历史。本次干净交付重新验证同五个固定路径、分类、Rust 对应物及 comparison/parity
台账中的语义/非 wire 边界。

不引入源 Python、Make、provider、历史工具或新的 Runtime 行为。首次验证前绑定 GitHub
resource context，最终 evidence 绑定仓库与快照。

验收：五条 inventory 记录明确；三种语言台账一致；文档/parity 检查与 locked workspace 验证通过。

[English](WI-338-reference-docs-foundation-clean.md) ·
[日本語](WI-338-reference-docs-foundation-clean.ja.md)
