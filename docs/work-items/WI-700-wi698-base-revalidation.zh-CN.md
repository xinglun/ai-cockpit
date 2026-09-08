---
author: AI Cockpit 维护者
title: “WI-700——WI-698 基线重验证”
description: “保留不可变的 WI-698 交付，并将其重验证绑定到当时的默认基线。”
audience: [contributor, maintainer, reviewer]
workItemId: WI-700-wi698-base-revalidation
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-700-wi698-base-revalidation
---

[English](WI-700-wi698-base-revalidation.md) · [日本語](WI-700-wi698-base-revalidation.ja.md)

# WI-700——WI-698 基线重验证

WI-700 是 WI-698 的不可变恢复前件。其 Contract 和 verification 绑定了较旧的默认
分支，因此 WI-701 从当前默认基线重新验证同一有界交付，不改写 WI-698 或 WI-700 的
archive 与 evidence 字节。

## 恢复边界

- Archive：`.ai/work-items/archive/WI-700-wi698-base-revalidation.contract.json`
- 历史 verification：`.ai/evidence/WI-700-wi698-base-revalidation.verification.json`
- Recovery decision：`.ai/decisions/WI-700-wi698-base-revalidation.recovery.json`
- Successor：WI-701-wi700-current-base-revalidation

本恢复投影不引入新的实现语义、治理规则、协议格式或其他 agent 的分支变更。
