---
author: AI Cockpit maintainers
title: "WI-278——pre-close 三语 Work Item 文档门"
workItemId: WI-278-preclose-docs-gate
description: "对 parity/documentation 投影缺失失败关闭，同时保持普通代码路径轻量。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-278-preclose-docs-gate
authority: canonical
---

# WI-278——pre-close 三语 Work Item 文档门

本 Work Item 修复 WI-277 之后发现的流程缺口：hosted governance 可能在 close 之前通过，
但 close 后的文档 promotion 才发现 Work Item 文档缺失。静态门根据 Contract 声明的 parity
所有权动态选择策略，检查英文、日文和中文的 regular、非 symlink 投影，并在不改写 `.ai`
历史记录的前提下修复当前周期遗漏。

普通代码 Work Item 不会仅因增加此门就被要求创建文档。同一规则会通过 repository-bound CI
门继承到 adopter repository。
