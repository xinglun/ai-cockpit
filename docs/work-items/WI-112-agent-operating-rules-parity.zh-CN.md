---
author: AI Cockpit maintainers
title: "WI-112 Agent 操作规则对齐"
description: "为未来 Rust Work Item 固化参考源中适用的 Agent 工作流规则。"
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - agent_workflow_boundaries
---

# WI-112：Agent 操作规则对齐

## 目标

让未来 Work Item 继承参考源中有价值的 Agent 工作流、Outcome、评审、发布和安全
边界，同时保持本工程共享 Rust Runtime 与仓库本地状态模型。

## 范围

本 Work Item 更新 `AGENTS.md`、`.ai/README.md`、三语 Agent 工作流参考页、参考索引
及本 Work Item 记录，并把参考规则分类为继承、本工程 Rust 适配或模板专属排除。不修改
Runtime 代码、Protocol schema、全局 Agent/MCP 配置、打包或发布资产。

## 验收

- 明确 remote/default branch 和不可变公开 Release 边界。
- 为未来 Work Item 固化 Contract、glossary、scope、Summary、evidence、checks、
  Outcome、问题处理、并行兼容和合并后 closure 规则。
- 面向人的 Outcome 明确保留可见的 `🔴`、`🟡`、`🟢` 标记，并按 fail-closed 规则推进。
- 英文、中文、日文参考页与 Work Item 记录同步且链接有效。
- 明确排除参考源专属的 `make ai-*`、`contractVersion: 2` 和 V1 假设。

## 验证

```text
bash tests/docs/documentation_acceptance.sh
git diff --check
```

## Outcome

状态：**本地实现完成；Runtime-bound lifecycle 与文档检查通过。**
