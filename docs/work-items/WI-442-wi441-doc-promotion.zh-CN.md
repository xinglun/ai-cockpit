---
author: AI Cockpit maintainers
title: "WI-442——WI-441 parity ledger 文档投影"
workItemId: WI-442-wi441-doc-promotion
description: "将已关闭 WI-441 的终态证据投影到三语 reference-parity ledger。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-442-wi441-doc-promotion
terminalArchive: .ai/work-items/archive/WI-442-wi441-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-442-wi441-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-442-wi441-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-442-wi441-doc-promotion.close.json
---

# WI-442——WI-441 parity ledger 文档投影

本 Work Item 将 WI-441 的不可变终态路径投影到英文、简体中文和日文
reference-parity ledger。不修改 Runtime 行为，也不重写 WI-441 的 evidence 字节。

[English](WI-442-wi441-doc-promotion.md) · [日本語](WI-442-wi441-doc-promotion.ja.md)

## 范围

- 更新三份 `docs/reference/reference-parity.*.md` ledger。
- 明确记录 archive、verification、finalization、close 路径。
- 保留仅使用本地参考源的边界。

## 验证边界

Runtime 验证命令为 `cargo test --locked --workspace`。只有
`python3 tests/docs/promote_closed_work_item.py --check-all` 报告 ledger 当前且
governance-integrity 门通过时，投影才算完成。
