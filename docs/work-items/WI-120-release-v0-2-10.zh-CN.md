---
author: AI Cockpit maintainers
description: "发布 v0.2.10 并完成不可变 adopter 验收。"
audience:
  - adopter
  - maintainer
authority: canonical
lastVerifiedBy: documentation-acceptance
workItemId: WI-120-release-v0-2-10
title: 发布 v0.2.10 并完成不可变 adopter 验收
status: release-preparation
---

# WI-120 — 发布 v0.2.10 并完成不可变 adopter 验收

## 目标

发布包含 Contract 事前 Human Review gate 的首个公开 Runtime，并证明下载的
Release binary 可以在不使用源码 fallback 的情况下治理全新 adopter，且能从上一版本升级。

## 范围

- 将 workspace 与当前发布文档更新为 `v0.2.10`；
- 发布不可变公开制品并记录 Runtime identity；
- 在隔离目录执行 fresh-adopter 与 v0.2.9 → v0.2.10 N-1 验收；
- 安装公开 binary，并使用显式 repository context 验证当前工程。

## 边界

本 Work Item 不增加 Runtime 功能、不重写历史 evidence，也不修改全局 Agent/MCP 配置。
发布后验收可以报告失败，但不得改写已发布 Release truth。

## 验收

- version、文档和发布策略检查只把 `v0.2.10` 作为当前基线，同时明确保留历史引用；
- CI 与发布检查通过；
- fresh-adopter 保留 `first-adopter-smoke = not_ready`，并记录下载 binary digest、repository identity、
  evidence reuse、生命周期、隔离与清理 receipt；
- N-1 验收证明 v0.2.9 → v0.2.10 兼容路径；
- 安装的公开 binary 报告 `0.2.10`，当前工程的 inspect、status、doctor、Agent doctor 和 Outcome 检查通过。

## Evidence 与决定边界

Release 发布不等于 adopter 验收通过。公开 archive、manifest、checksum、验收 receipt 与 Runtime identity
必须可以分别验证。任何 yellow 或 red 状态都需要 Human decision；acceptance criteria 保持 Contract 原文，
不会被静默翻译成治理事实。
