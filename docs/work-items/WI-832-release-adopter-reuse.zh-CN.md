---
author: AI Cockpit 维护者
title: "WI-832——adopter verification reuse"
description: "使 staged 与升级 adopter 验收与 Runtime 的 package 路由保持一致。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-832-release-adopter-reuse
lastVerifiedBy: WI-832-release-adopter-reuse
---

[English](WI-832-release-adopter-reuse.md) · [日本語](WI-832-release-adopter-reuse.ja.md)

# WI-832——adopter verification reuse

## 意图

adopter harness 必须证明成功验证可以复用。因此 profile confirmation 记录 Runtime
workspace package 路由实际执行的精确命令：`cargo test --locked --package adopter`。

## 边界

staged 与 N−1 升级验收使用相同的 package 级命令。回归检查保留首个 receipt，要求第二次
验证报告零个新进程，并不改变 Runtime 复用语义或发布身份。

## 验证

- adopter 与升级 harness 的静态检查通过。
- 真实 v0.2.92 staged candidate 运行中，第二次验证报告 `nodesReused: 1`、
  `processesSpawned: 0`。

## 不在范围内

Runtime 复用协议、产品构建、Release 标签与产物、历史 Work Item 及无关清理。
