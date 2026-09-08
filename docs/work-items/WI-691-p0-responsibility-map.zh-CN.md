---
author: AI Cockpit maintainers
title: “WI-691 — P0 架构职责关系图”
description: “为架构优化专项记录当前职责和依赖边界。”
audience: [contributor, maintainer, reviewer]
status: in_progress
authority: human:repository-owner
workItemId: WI-691-p0-responsibility-map
lastVerifiedBy: WI-691-p0-responsibility-map
---

[English](WI-691-p0-responsibility-map.md) · [日本語](WI-691-p0-responsibility-map.ja.md)

# WI-691 — P0 架构职责关系图

## 意图

在后续架构 Work Item 之前，以当前源码引用记录 observation、governance、lifecycle、
evidence、execution、persistence/recovery 以及 status/Outcome projection 的职责边界。

## 边界

仅修改文档。本 Work Item 不修改 Rust 源码、测试、公共 protocol、治理规则、`.ai/` 记录
或历史证据。地图基于最新远程默认分支 revision
`99f7d2323ffb59b1e3edd6c1c833b00f50fb9698`。

## 验收

- 英文、简体中文、日文地图引用当前 CLI/MCP 入口和 observation、Contract/policy/evidence、
  governance、lifecycle、execution、projection、persistence/recovery 调用链。
- 每项职责说明权威事实、允许的 I/O、校验/决策/展示所有者和可复用机制。
- 区分具体重复/职责混合与待调查问题；不把结构体或单文件 rename 推断为原子快照或多文件事务。
- P1-B、P2-A、P2-B、P2-C、P3 都有有界问题、目标边界、兼容风险和验证方式。
- 三种语言页面与 parity projection 保持一致，且不含代码或 Runtime 行为变化。

## 证据与验证

Runtime 必需证据是 locked workspace test。文档验收、parity status、`git diff --check` 和
hosted governance checks 是额外交付证据。最终状态必须来自 Runtime archive、verification、
finalization、close 记录，不能由本页面自行宣称。

## 后续问题

当前源码已经分离 Git snapshot、typed protocol facts、受限 evidence store、lifecycle lock、
bounded execution 和最终 human renderer。仍需调查端到端 observation context 所有权、多文件
lifecycle 操作的权威提交记录，以及 reusable verification 与 physical execution 的边界；本
Work Item 不实现这些变化。

