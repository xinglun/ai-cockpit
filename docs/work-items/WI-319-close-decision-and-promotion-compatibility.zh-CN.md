---
author: AI Cockpit maintainers
title: "WI-319——close decision 与 promotion 兼容"
workItemId: WI-319-close-decision-and-promotion-compatibility
description: "让静态晋级和治理消费者与已安装 Runtime 的 close/finalization 绑定保持一致。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-319-close-decision-and-promotion-compatibility
---

# WI-319——close decision 与 promotion 兼容

## 意图与边界

已安装 Runtime 支持两个明确的正向 close decision（`approved` 与
`confirmed`），并可能在 `close` 前追加 deleted sequence-1 finalization transition。
静态文档与治理消费者必须识别这些当前记录，同时保留历史的 close 后
reconciliation 路径。本 Work Item 只修改这些消费者及其三语文档，不改写任何
不可变 Runtime 记录。

## 范围与验收

- promotion、status 和 governance 检查同时接受当前 sequence-1
  cleanup-before-close 记录与历史 root-bound reconciliation 形态，并继续拒绝
  predecessor、identity、path 或 digest 的任何不一致。
- `approved` 与明确的 `confirmed` 结构化 close decision 为正向决定；`rejected`
  永远不能把 Work Item 晋级为已实现。
- W317 的关闭投影在三语 Work Item 文档和 parity ledger 中如实体现，不改变其
  不可变 archive、verification、finalization 或 close bytes。
- 回归 fixture 覆盖两种 finalization 路径和 `confirmed` decision token；文档验收
  与 governance gate 继续保持严格。
- 本 Work Item 使用已安装 Runtime 的生命周期；只有 hosted checks 通过后才执行
  finalize、评审、合并、close 和精确清理。

## 验证

使用已安装 Runtime 并显式指定 repository context，运行 promotion、status-consistency、
governance-integrity、文档回归、locked workspace 测试，以及 reviewed branch 的 hosted checks。

## 不在范围内

Rust Runtime 生产代码、release/adopter harness、不可变 `.ai` archive/decision bytes、
全局 Agent/MCP 配置和无关的参考源比较批次均不在本次有界兼容修复内。

[English](WI-319-close-decision-and-promotion-compatibility.md) ·
[日本語](WI-319-close-decision-and-promotion-compatibility.ja.md)
