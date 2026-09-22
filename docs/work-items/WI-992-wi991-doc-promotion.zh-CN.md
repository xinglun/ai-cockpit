---
author: AI Cockpit maintainers
title: "WI-992——WI-991 终态文档投影"
description: "在 WI-991 尝试被替代后，完成修正后的终态文档投影。"
audience: [maintainer, reviewer, contributor]
status: implemented
authority: authorized
workItemId: WI-992-wi991-doc-promotion
lastVerifiedBy: WI-992-wi991-doc-promotion
terminalArchive: .ai/work-items/archive/WI-992-wi991-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-992-wi991-doc-promotion.verification.json
terminalDecision: .ai/decisions/WI-992-wi991-doc-promotion.close.json
---

[English](WI-992-wi991-doc-promotion.md) · [日本語](WI-992-wi991-doc-promotion.ja.md)

# WI-992——WI-991 终态文档投影

WI-992 是 WI-991 明确绑定的后继项，使用修正后的验证命令完成有边界的 WI-990
终态投影，同时保留 WI-991 的失败尝试和 retirement 事实。

## 边界

范围仅包括 WI-990、WI-991、WI-992 文档投影、三份 reference-parity 台账以及 Runtime
生成的 `.ai/` 证据。不修改源码行为、发布行为或既有生命周期事实。

## 验收

- 六份 WI-990 终态投影与不可变 archive、verification 和 close 证据一致。
- WI-991 的失败尝试、recovery 和 retirement 保持明确，不声称验证通过。
- 三份 WI-992 页面和 parity 行在关闭前保留本 Work Item 的有边界自投影。
- WI-990 单项投影、全仓 `--check-all`、parity 和状态一致性检查通过。
