---
author: AI Cockpit maintainers
title: "WI-436——已关闭文档投影晋级"
workItemId: WI-436-reference-doc-promotion
description: "在 WI-435 关闭后晋级三语文档投影。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-436-reference-doc-promotion
terminalArchive: .ai/work-items/archive/WI-436-reference-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-436-reference-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-436-reference-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-436-reference-doc-promotion.close.json
---

# WI-436——已关闭文档投影晋级

本仅文档 Work Item 使用仓库内的 closed Work Item 晋级脚本处理 WI-435。
语义参考源固定为维护者提供的本地
`/Users/sei-rinn/dev/workspace_python/ai-cockpit-template`，不访问公开参考
仓库，也不改变 Runtime 行为。

[English](WI-436-reference-doc-promotion.md) · [日本語](WI-436-reference-doc-promotion.ja.md)

## 范围

- 晋级 WI-435 的三语 Work Item 文档和三语 reference-parity 行。
- 只记录不可变的 archive、verification、finalization、close 路径。
- 不修改其他 Work Item 或历史字节。

## 验证

必须通过 `tests/docs/promote_closed_work_item.py --work-item
WI-435-reference-inventory-rebaseline-local`、`--check-all`、文档验收、parity
状态检查和 diff 检查。
