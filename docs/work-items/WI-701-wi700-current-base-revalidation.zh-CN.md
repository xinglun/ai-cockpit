---
author: AI Cockpit 维护者
title: “WI-701——WI-700 当前基线重验证”
description: “从当前远程默认基线重新验证观察上下文交付。”
audience: [contributor, maintainer, reviewer]
workItemId: WI-701-wi700-current-base-revalidation
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-701-wi700-current-base-revalidation
---

[English](WI-701-wi700-current-base-revalidation.md) · [日本語](WI-701-wi700-current-base-revalidation.ja.md)

# WI-701——WI-700 当前基线重验证

WI-701 是 WI-700 的新恢复 successor。它将保留的 observation-context 实现和前件
lineage 绑定到当前远程默认 revision，然后执行新的验证以及正常的 hosted 和 Runtime
终态 lifecycle。

## 边界

本 WI 不改写前件 archive、evidence 或 recovery decision，不引入新的代码语义、治理规则、
协议格式，也不修改其他 agent 的 Work Item。三语页面和 parity 行是同一恢复边界的投影。

## 验收

- 当前默认基线和前件 digest 明确绑定。
- 必需的 observation-context 场景与 locked workspace gate 通过。
- 在宣布终态前，必须有 hosted quality、provider finalization、archive、close 和准确清理证据。
