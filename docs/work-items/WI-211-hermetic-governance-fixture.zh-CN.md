---
author: AI Cockpit maintainers
title: "WI-211——治理 fixture 的事件上下文隔离"
description: "让发布 workflow 导出的 GitHub 事件变量不会污染治理回归 fixture。"
audience:
  - maintainer
  - reviewer
workItemId: WI-211-hermetic-governance-fixture
status: recovered
authority: canonical
lastVerifiedBy: WI-211-hermetic-governance-fixture
---

# WI-211——治理 fixture 的事件上下文隔离

发布 workflow 会把 GitHub 事件变量导出到整个 source quality job。此前治理回归测试
允许这些变量泄漏到普通 fixture，造成本地通过而 release-tag CI 失败。本 Work Item
让每个 fixture 都显式声明自己的事件上下文。

## 验收

1. `tests/ci/governance_integrity_gate_test.sh` 在普通环境和 release-tag 环境变量下都通过。
2. 普通 fixture 显式清除 release 上下文；真正的 `release-tag-*` fixture 仍使用严格的 tag 上下文。
3. 两种环境产生相同且确定性的 findings 与退出状态。
4. 不移动、改写或把不可变的 v0.2.26 发布历史作为源码 fallback。

## 不在范围内

本 Work Item 不改变 Runtime 治理语义、公开 Release 资产、参考源逐文件对比，或用户全局
Agent/MCP 配置。

## 验证

分别在没有 GitHub 事件变量的环境，以及设置
`GITHUB_EVENT_NAME=push`、`GITHUB_REF=refs/tags/<tag>` 和对应 `GITHUB_SHA` 的环境运行回归。
随后运行 repository gate manifest 与 workspace tests。

## 证据边界

本修复属于 source-test 证据，不会把失败的 v0.2.26 发布变成成功 Release；该历史保持不可变，
后续发布必须使用新 tag。
