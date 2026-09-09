---
author: AI Cockpit maintainers
title: "WI-743——WI-715 当前基线性能决定重新交付"
description: "从最新默认分支重新验证 WI-715 被拒绝的大历史候选，不恢复过期的未合并 PR。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-743-wi715-p1-current-base-redelivery
lastVerifiedBy: WI-743-wi715-p1-current-base-redelivery
---

[English](WI-743-wi715-p1-current-base-redelivery.md) · [日本語](WI-743-wi715-p1-current-base-redelivery.ja.md)

# WI-743——WI-715 当前基线性能决定重新交付

## 意图

从最新远程默认分支重新交付 WI-715 大历史 status 候选的决定，保留有证据支持的拒绝、限制和“不改变生产代码”的边界。这个 Work Item 不恢复 PR #708。

## 继承关系与边界

- 当前基线：`origin/main`，提交为 `838ae745511942cb55dd7ac30c319cc5bc95e74d`。
- 前置 Work Item：`WI-715-p1-large-history-status`。其 archive、evidence、decision 和 PR #708 仍作为不可变审计历史保留在前置分支：<https://github.com/xinglun/ai-cockpit/pull/708>。
- 本 Work Item 只覆盖当前基线验证、证据绑定和三语治理记录，不改变 Runtime 行为、性能阈值、测量语义、Outcome/schema、退出码、授权、持久化或其他 agent 的工作。

## 有证据支持的决定

前置项的当前基线实验在 `d1141480fb7a045979098480c3770d06002e2a87` 上运行，使用同一 Runtime 线。正向顺序的 warm status p50 变化为 `-1.171%`，反向顺序为 `-0.302%`。注册阈值是两个方向都达到 `5%`，因此候选仍为拒绝。两个 gate 还因 filesystem comparison key 不可用而 fail closed。p99 以及若干 phase/resource 指标不可用；它们被记录为不可用，不是零。

之后的默认分支提交只涉及治理、文档和测试，没有生产 Runtime 源码变化。这支持对该拒绝决定进行当前基线的语义重新验证，但不构成新的延迟收益声明。候选实现没有合并。

## 当前状态

Runtime lifecycle 为 `checkpointed`；当前基线验证、reviewed PR 交付、finalization、archive 和人工 close 仍待完成。当前权威验证记录将是 `.ai/evidence/WI-743-wi715-p1-current-base-redelivery.verification.json`，并与 Contract 列出的前置 external records 一起使用。

## 决定边界

决定：保留 `declined`。不声明延迟、CPU、I/O、内存或 resident-MCP 收益。未来优化必须从 reviewed 默认分支开始，并先提供可信的环境比较键和 phase 级证据，再提出生产行为变更。
