---
author: AI Cockpit maintainers
title: "WI-772——WI-771 文档恢复重验证"
description: "在不可变 recovery decision 后，从当前默认分支重新验证已合并的 WI-771 文档投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-772-wi771-doc-promotion-recovery
lastVerifiedBy: WI-772-wi771-doc-promotion-recovery
recoveryDecision: .ai/decisions/WI-771-wi770-doc-promotion.recovery.json
---

[English](WI-772-wi771-doc-promotion-recovery.md) · [日本語](WI-772-wi771-doc-promotion-recovery.ja.md)

# WI-772——WI-771 文档恢复重验证

## 意图与边界

本有界 successor 根据不可变 recovery decision，从当前默认分支重新验证已合并的 WI-771
文档投影。保留 WI-771 的 archive、evidence、outcome、summary、events、recovery decision
和 PR #755 bytes；Runtime、性能、发布和版本行为均不在范围内。

## 验证边界

新鲜 Runtime 绑定 verification、finish、archive、本地 finalization、finalize-verify、close、
close 后 promotion 和精确清理均由本 successor 负责。不引入性能实现或收益声明。
