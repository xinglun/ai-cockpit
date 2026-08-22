---
author: AI Cockpit maintainers
workItemId: WI-129-parity-gate
title: 强制参考源对齐完整性
description: 让文档门禁推导最新已实现 Work Item，而不是只依赖固定列表。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: documentation-acceptance
---

# WI-129——参考源对齐完整性

三语 parity 基线现在包含 WI-128。文档验收门禁还会从标记为
`status: implemented` 的 canonical 英文 Work Item 文档中推导最高数字 ID，并要求
该 ID 出现在每种 parity 语言中。这样新合并实现若遗漏文档会 fail-closed，而不再
依赖人工记得修改固定列表。

门禁保持只读，不推导治理事实，也不修改 Runtime、Contract、Summary 或 evidence。
