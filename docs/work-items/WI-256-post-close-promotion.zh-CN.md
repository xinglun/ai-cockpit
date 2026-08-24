---
author: AI Cockpit maintainers
title: "WI-256——Typed post-close 文档 promotion"
workItemId: WI-256-post-close-promotion
description: "使 close 后的文档 promotion 可重复、身份绑定并 fail closed。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-256-post-close-promotion
authority: canonical
---

# WI-256——Typed post-close 文档 promotion

WI-256 修复 WI-255 暴露的流程缺口：结构化 close 本身有效，但三语文档 projection 依赖容易
遗漏的手动命令。本 WI 增加 repository-owned typed plan/apply wrapper；不把 Markdown 行为移入
Runtime Core，也不重写任何 immutable `.ai` lifecycle bytes。

## 验收边界

- plan 绑定 repository identity、同步的 `origin/main`、approved close、sequence-2
  finalization、archive/evidence identity，以及六个准确受控文档路径的 before/after digest。
- stale、foreign、malformed、symlink、dirty、partial 或 unexpected state 在任何写入前
  fail closed；对 current plan 重复 apply 是 deterministic no-op。
- WI-255 英文、简体中文和日文 projection 变为 `Implemented`，不改变其 `.ai` archive、
  evidence、finalization 或 close bytes。
- AGENTS 与三语 workflow/command 文档要求
  `close → visible Outcome → post-close plan/apply → check-all → terminal CI`。
- wrapper、promoter、文档、manifest、governance、format、clippy 与 locked workspace
  checks 在安装 Runtime 下通过。

## 验证场景

Contract 覆盖 valid plan/apply/idempotent rerun、typed identity/staleness rejection、
dirty/unexpected/partial projection rejection 与可执行的 terminal-CI handoff。wrapper 测试使用
隔离 Git fixture，并断言 immutable `.ai` digest 保持不变。

## 参考

- [Agent workflow](../reference/agent-workflow.zh-CN.md)
- [Commands](../reference/commands.zh-CN.md)
- [Reference parity](../reference/reference-parity.zh-CN.md)
