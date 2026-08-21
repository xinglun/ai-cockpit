---
author: AI Cockpit maintainers
title: "Work Item"
description: "本仓库的 repository-local 受治理实现生命周期。"
audience:
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - work_item_lifecycle
---

# Work Item

本仓库由已安装的 Rust `ai-cockpit` Runtime 治理，不安装 V1 template。每次修改都使用
repository-local `.ai/` Contract、evidence 和 human decision records。

每个 Work Item 使用一个 branch、base revision、change scope、evidence bundle 和 outcome，
不能只靠文字声称完成。必需章节包括 Intent 与 Goal、Scope 与 Out of Scope、Sources 与
Unknowns、Acceptance Criteria、Required Evidence、Base Revision、Changed Files、Verification、
Human Decisions 和 Outcome。面向读者或改变 Runtime 行为的 Work Item 必须同步英文、中文和日文。

## Runtime 命令

使用外部共享 Runtime，并始终显式指定 repository context：

```bash
ai-cockpit status --repo /path/to/ai-cockpit
ai-cockpit start --repo /path/to/ai-cockpit --id <id> \
  --intent "..." --goal "..." --scope "..." --authority authorized
ai-cockpit preflight --repo /path/to/ai-cockpit \
  --contract .ai/work-items/active/<id>.contract.json
ai-cockpit checkpoint --repo /path/to/ai-cockpit --id <id>
ai-cockpit verify --repo /path/to/ai-cockpit --work-item <id>
ai-cockpit finish --repo /path/to/ai-cockpit --id <id>
ai-cockpit archive --repo /path/to/ai-cockpit --id <id>
ai-cockpit close --repo /path/to/ai-cockpit --id <id> --human-decision approved
```

Runtime 是外部共享的，`.ai/` 属于 repository。不存在全局 current repository 或 Work Item。
Agent 路线见 `.ai/README.md`，标准术语见 `.ai/glossary.md`。
