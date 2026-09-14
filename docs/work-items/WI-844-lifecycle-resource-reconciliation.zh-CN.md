---
author: AI Cockpit 维护者
title: "WI-844——生命周期资源收敛"
description: "在收敛 Work Item 生命周期和精确资源时保留历史 successor 绑定。"
audience: [maintainer, reviewer]
status: in_progress
authority: authorized
workItemId: WI-844-lifecycle-resource-reconciliation
lastVerifiedBy: WI-844-lifecycle-resource-reconciliation
---

[English](WI-844-lifecycle-resource-reconciliation.md) · [日本語](WI-844-lifecycle-resource-reconciliation.ja.md)

# WI-844——生命周期资源收敛

## 意图与边界

本 Work Item 收敛 reviewed merge 后遗留的 Work Item 生命周期状态、远端分支和本地工作树。
历史 Contract、Summary、Outcome、事件和证据字节保持不变；历史证据不支持或缺失时必须明确分类。
只有精确匹配、干净、已证明 reviewed merge 且生命周期已结束的资源才可删除。Release tag、公开
Release、产品行为和无关源码不在范围内。

## 恢复边界

范围内的生命周期缺陷在本 Work Item 中 amend 并重新验证。只有 scope、authority 或 base 不同、
独立变更、无法安全进行的范围内修复、不可变失败交付或明确人工指示才使用 successor。
已经绑定的 successor 只能使用不可变 checkpoint Contract digest supersede 已追加 Contract 的前置 WI；
未绑定或竞争的 successor 仍必须 fail-closed。

## 验收

- 每个 active Work Item 都有有证据支持的处置：closed、有具体原因的 blocked，或保留为当前实现。
- 历史记录按字节保留，不能通过补造字段或针对当前仓库快照重跑来完成。
- 只有生命周期关闭后精确匹配且干净的已合并分支和工作树才会删除；其他资源保留原因并可恢复。
- 前置 Contract 的追加修改不会使严格绑定 successor 的 checkpoint digest 失效，也不会强制重跑历史产品验证。

## 验证

- `cargo test --locked -p cockpit-repository --test recovery_decision`
- `cargo fmt --all -- --check`
- `git diff --check`
