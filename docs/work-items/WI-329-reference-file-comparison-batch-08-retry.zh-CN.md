---
author: AI Cockpit maintainers
title: "WI-329——参考源文件对比批次 08 的 CI 回归修复"
workItemId: WI-329-reference-file-comparison-batch-08-retry
description: "在不可变的 WI-328 hosted inventory 门失败后，从干净默认分支重新交付九文件批次。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implementation
authority: canonical
lastVerifiedBy: WI-329-reference-file-comparison-batch-08-retry
---

# WI-329——参考源文件对比批次 08 的 CI 回归修复

## 意图与边界

WI-328 的源文件对比和 inventory 已在 hosted quality 发现一个脆弱断言前归档：
conformance wrapper 要求 reference-only 原因中出现未来 WI 编号。WI-328 的已关闭
PR 和不可变记录保留为历史证据。本窄 successor 从同步后的默认分支重新执行同一批次，
让门检查稳定断言语义边界，而不是任意的 Work Item 编号。

Runtime 仍在仓库外部，每个仓库操作都显式带 `--repo`。不加入源 Python/Make 实现、通用
Session、全局 Agent/MCP 配置或 Runtime 行为。

## 修复与文件范围

| 路径 | 决定 |
| --- | --- |
| `tests/conformance/reference_file_inventory_test.sh` | 用稳定的语义短语断言替换脆弱的未来 WI 编号断言。 |
| `tests/conformance/reference_file_inventory.py` 与 `.json` | 保持四个 capability matrix/claim 路径为 `reference-only`，只描述未来专门 Work Item，不把本修复冒充其实现。 |
| `docs/reference/reference-file-comparison.*` | 保留 WI-328 九项分类并说明不可变 predecessor/successor 边界。 |
| `docs/reference/reference-parity.*` | 将 WI-328 标为已恢复，并在本次 verification 前登记 successor。 |
| `docs/work-items/WI-328-reference-file-comparison-batch-08.*` | 将未来 capability-claim 跟进改记为 WI-330，使 WI-329 专用于 CI 修复。 |
| `docs/work-items/WI-329-reference-file-comparison-batch-08-retry.*` | 三语记录本次有界修复和终态证据。 |

九个固定 source 路径保持与 WI-328 相同：五个为
`implemented-different-by-design`，四个明确为 `reference-only`。不复制或宣称源公共能力
矩阵/checker 是目标 gate；后续仍需专门的 capability-claim/evidence Work Item。

## Adopter 反馈边界

Cursor 报告是外部验证输入。稳定 lifecycle JSON、持久 human Outcome 重放、close-before-next
检查与 fail-closed start 仍是现有 Runtime 能力。自动 IDE 聊天发布、诊断修复、controls
脚手架、close-gap 便利命令和 Makefile 要求均不在本范围。

## 验收与证据

1. `tests/conformance/reference_file_inventory_test.sh` 使用固定 source commit 与 target baseline
   通过，并覆盖修复后的语义断言。
2. WI-328 inventory 保持五个 `implemented-different-by-design` 和四个 `reference-only`，没有
   deferred-next-batch 或 migrate-gap。
3. English、简体中文和日本語的 comparison/parity/Work Item 页面都说明 predecessor 恢复、
   语义门断言和未来 capability-claim 边界。
4. 不改写或静默删除 WI-328 历史 bytes，不加入源 Python/Make 执行或全局 Agent/MCP 配置。
5. 安装 Runtime 的 inspect/status/doctor/agent doctor、聚焦门、完整 workspace 检查、hosted CI、
   reviewed merge、finalization、close 以及精确 branch/worktree 清理全部通过。

[English](WI-329-reference-file-comparison-batch-08-retry.md) · [日本語](WI-329-reference-file-comparison-batch-08-retry.ja.md)
