---
author: AI Cockpit maintainers
title: "WI-288 — Provider finalization correction"
workItemId: WI-288-provider-finalization-correction
description: "在实际 Provider PR 身份已知后重新交付 predecessor 实现，并保留不可变恢复链路。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-288-provider-finalization-correction
authority: canonical
---

# WI-288 — Provider finalization correction

## 目的

WI-287 的 Provider context 含有占位 PR URL，因此按 fail-closed 规则保留为
不可变历史。本 successor 不修改 predecessor bytes，也不增加 Runtime 功能；
只在实际 GitHub PR 身份确定后重新完成同一实现的交付。

## 边界

- 原样保留 WI-287 archive 与 recovery decision。
- 在 verify 之前把本 Contract 的 `resourceContext` 绑定到实际 PR。
- 使用已安装 Runtime 和 hosted checks 重新验证。
- 记录并验证 Provider finalization，以结构化决定 close，并只清理精确的
  merged branch/worktree。

## 对象工程能力一致性

本 successor 验证的仍是对象工程可获得的显式 repository context、fail-closed
未知项和人类可见 Outcome，不从本地记录推断 Provider approval。

## 验证

`cargo test --locked --workspace`、conformance/documentation acceptance、PR
hosted checks、Provider finalization verify，以及 close 后的 status/doctor。
