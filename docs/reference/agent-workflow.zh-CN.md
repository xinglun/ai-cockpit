---
author: AI Cockpit maintainers
title: "Agent 工作流与评审边界"
description: "未来 AI Cockpit Work Item 继承的仓库本地操作规则。"
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - agent_workflow_boundaries
---

# Agent 工作流与评审边界

本文是参考源操作规则在本工程中的适用投影，保留治理意图，但使用本
Rust Runtime 与本仓库的 Protocol 词汇。

## 继承的规则

- 从仓库发现的远端 default branch 最新提交开始工作，并在 Work Item
  Contract 中记录 remote、default branch 和 base revision。
- 每个 Work Item 使用一个 Contract、一个专用 branch/worktree 和一个 PR。
  只有 scope、evidence ownership、repository context 与串行投影均隔离且
  Runtime 判定兼容时，独立 Work Item 才能并行。
- 修改前阅读 `.ai/README.md` 与 `.ai/glossary.md`，查询 `inspect`、`status`、
  `doctor`；修改不得超出声明 scope；保留测试和证据；更新 Summary；执行
  Contract 声明的工程检查。
- 单独交付面向人的 Outcome，并以 `Outcome: 🟢`、`Outcome: 🟡` 或
  `Outcome: 🔴` 开头，包含 unknown、evidence、人工决定和下一步。Outcome
  缺失、仅折叠显示、过期、矛盾或格式错误时必须 fail closed，不得授权继续。
- 发现属于当前 Work Item 的问题时，先修复并 amend/revalidate 当前 Contract。
  只有 scope、authority 或 base 真正不同、变更独立、无法安全在当前范围修复、
  失败交付必须重新交付，或人明确指示时，才创建 successor。
- 安装和升级验收使用不可变的公开 Release tag 与下载 binary。合并后，closure
  必须核验归档证据、decision、合并 PR head、同步后的 default branch、干净
  worktree 和精确 branch 删除；任一步失败都保持可恢复的未闭合状态。

## 本工程的适配

参考源包含 `make ai-*` 命令和 `contractVersion: 2` 模板 Protocol；它们不是
本工程的命令或 schema 要求。本 Rust 工程使用已安装共享 Runtime 与显式流程：

```text
start → preflight → checkpoint → verify → finish → archive → close
```

每个 repository-bound 命令都带 `--repo`。Runtime 没有全局 current repository、
Work Item 或 project profile。Contract 条件保留其原始语言，只有面向人的表现层
负责本地化。

## 安全边界

规则保持语言中立并属于仓库本地。不得写入 secret 或机器凭据，不得修改用户全局
Agent/MCP 配置，也不得把 managed Agent prompt 当作治理 authority。不得把 V1
Runtime 代码、schema、installer 或模板实现复制进本仓库。
