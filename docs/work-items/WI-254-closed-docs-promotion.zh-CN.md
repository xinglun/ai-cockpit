---
author: AI Cockpit maintainers
title: "WI-254——确定性的关闭后文档提升"
workItemId: WI-254-closed-docs-promotion
description: "从精确且不可变的 close evidence 提升受控 Work Item 文档字段，并把检查设为必需质量 gate。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-254-closed-docs-promotion
terminalArchive: .ai/work-items/archive/WI-254-closed-docs-promotion.contract.json
terminalVerification: .ai/evidence/WI-254-closed-docs-promotion.verification.json
terminalFinalization: .ai/decisions/WI-254-closed-docs-promotion.finalize.51a5fc0158258cc2ac3e6ce03e20355202530af433005e342ba59495c474aa3a.json
terminalDecision: .ai/decisions/WI-254-closed-docs-promotion.close.json
authority: canonical
---

# WI-254——确定性的关闭后文档提升

WI-254 是 WI-253 的 Runtime 记录 successor。recovery receipt 绑定 WI-253 的
canonical Contract、Summary、Outcome、events、archive、verification、sequence-2
finalization 与 close evidence；这些 lifecycle records 保持不可变。

## 验收边界

- `tests/docs/promote_closed_work_item.py` 在规划文档变更前，严格验证
  repository/Work Item identity、archive Contract 原始 digest、passing
  verification、线性 finalization chain、sequence-2 deleted receipt、merge
  identity 与结构化 approved close。
- 写入边界只包括三份精确 Work Item 文档中的 `status`、`lastVerifiedBy` 与四个
  `terminal*` frontmatter 字段，以及每份 reference-parity 文档中的唯一精确行。
  Contract 原语言正文与所有 `.ai` lifecycle truth 均不重写。
- `--check-all` 是受治理 closed Work Item 的强制 documentation/quality gate。
  identity 或 filesystem 输入无效时，会在写入任何文档之前 fail closed；非
  canonical 的 stale projection 也无法通过检查。
- 本变更用同一 helper 提升 WI-253；WI-254 关闭后，再从同步 default branch 的
  detached closure context 提升 WI-254。

## Lifecycle handoff

完整交付顺序是 `close → promote closed docs → terminal CI`。helper 是显式的
repository workflow command；Runtime Core 不声称会自动修改 Markdown。因此，
close 前 PR run 变绿不能替代终态 projection 与 default-branch terminal run。

## 参考

- [WI-253 predecessor](WI-253-docs-terminalization.zh-CN.md)
- [Agent workflow](../reference/agent-workflow.zh-CN.md)
- [Reference parity](../reference/reference-parity.zh-CN.md)
