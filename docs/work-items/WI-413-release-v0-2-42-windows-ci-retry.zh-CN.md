---
author: AI Cockpit maintainers
title: "WI-413——Windows CI 后的 v0.2.42 发布恢复"
workItemId: WI-413-release-v0-2-42-windows-ci-retry
description: "在不可变的 WI-412 交付被 Windows CI 拒绝后，恢复 v0.2.42 候选发布。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-413-release-v0-2-42-windows-ci-retry
---

# WI-413——Windows CI 后的 v0.2.42 发布恢复

这是 WI-412 的有界 recovery successor。前置 archive、verification、recovery
decision 和失败 PR 保持不可变。本 successor 只修复完全由可复用 receipt 满足的
验证计划中平台相关的 `execution_elapsed_ms` 投影，然后重新执行完整发布验证和
reviewed delivery 生命周期。

## 范围

- 零执行节点时准确报告 0 elapsed time，不改变 receipt identity、reuse 授权或
  fail-closed 行为。
- 保留继承的 v0.2.42 版本/发布/文档候选，并同步三种语言投影。
- 合并前通过 workspace、hosted quality、Windows-runtime 和 reference-oracle；
  adopter 验收仍属于发布后步骤。

## Recovery 边界

WI-412 和 PR #377 因 hosted Windows CI 失败而保留为历史 recovery evidence。
本 successor 是唯一 active delivery path，不改写前置 bytes，也不扩大参考源比对批次。

## 验证

使用带显式 repository 路径的已安装 Runtime。检查包括 locked workspace tests、fmt、
warning-denied Clippy、发布静态门、治理完整性、文档一致性，以及 hosted
quality/Windows/reference-oracle。

[English](WI-413-release-v0-2-42-windows-ci-retry.md) · [日本語](WI-413-release-v0-2-42-windows-ci-retry.ja.md)
