---
author: AI Cockpit maintainers
title: "WI-665——WI-664 文档晋级"
description: "使用不可变终态证据晋级已关闭 WI-664 的文档投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-665-wi663-doc-promotion
lastVerifiedBy: WI-665-wi663-doc-promotion
---

[English](WI-665-wi663-doc-promotion.md) · [日本語](WI-665-wi663-doc-promotion.ja.md)

# WI-665——WI-664 文档晋级

## 意图

将三语 WI-664 Work Item 页面和共享 reference-parity 行与不可变的
archive、verification、finalization 和 close 记录同步。

## 边界

这是仅文档的投影变更，唯一目标是本 Work Item Contract 规定的九个 Markdown
文件。不可变的 `.ai` archive、evidence、recovery、finalization 和 close 记录只读；
不改变 Runtime、仓库行为、历史 Work Item 或其他 agent 的行为。

## 验收

- WI-664 页面及 parity 行在验证关闭后显示终态 `已实现`，并绑定精确终态证据路径。
- WI-665 作为本次有界文档投影的显式 prearchive 自注册保持可审计。
- 晋级、文档、parity、状态一致性、治理和 Hosted quality 检查在精确评审 head 上通过。
