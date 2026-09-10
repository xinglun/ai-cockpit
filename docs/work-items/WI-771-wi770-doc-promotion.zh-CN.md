---
author: AI Cockpit maintainers
title: "WI-771——WI-770 终态文档 promotion"
description: "将已验证关闭的 WI-770 文档投影提升到终态。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: human:repository-owner
workItemId: WI-771-wi770-doc-promotion
lastVerifiedBy: WI-772-wi771-doc-promotion-recovery
recoveryDecision: .ai/decisions/WI-771-wi770-doc-promotion.recovery.json
---

[English](WI-771-wi770-doc-promotion.md) · [日本語](WI-771-wi770-doc-promotion.ja.md)

# WI-771——WI-770 终态文档 promotion

本 Work Item 将已验证关闭的 `WI-770-performance-terminal-docs-recovery` 文档和
reference-parity 投影提升到终态，不修改其历史 evidence 或 Runtime。WI-771 原始交付在
Runtime finish/archive snapshot 生成前已合并，因此由不可变 recovery decision 绑定本 successor
重验证。

范围仅包括 WI-770 的六个文档投影、WI-771 三语恢复投影及本 Work Item 自身治理记录；不修改
Runtime、性能实现、发布行为、版本元数据或 WI-770/WI-771 的历史
archive/evidence/finalization/close 字节。

验收：closed Work Item promotion helper 的 `--check-all` 通过后，才投影终态链接。WI-771 的恢复
投影必须绑定不可变 archive、verification 和 recovery decision；新鲜 verification、finalization
与 close 由 WI-772 负责。

## 恢复状态

WI-771 是不可变的 recovered 前置项。PR #755 已合并，但发生在 WI-771 finish/archive snapshot
之前，因此原 verification snapshot 不作为当前完成证据复用。追加式 recovery decision
`.ai/decisions/WI-771-wi770-doc-promotion.recovery.json` 将新鲜验证、finalization 和 close
边界交给 WI-772；不改变 Runtime 或性能行为。
