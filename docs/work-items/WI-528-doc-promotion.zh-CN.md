---
author: AI Cockpit maintainers
title: "WI-528——WI-526 终态文档晋级"
description: "在不改写 Runtime 证据的前提下晋级发布 Work Item 文档投影。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-528-doc-promotion
lastVerifiedBy: WI-528-doc-promotion
terminalArchive: .ai/work-items/archive/WI-528-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-528-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-528-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-528-doc-promotion.close.json
---

[English](WI-528-doc-promotion.md) · [日本語](WI-528-doc-promotion.ja.md)

## 目标

使 WI-526 发布页面和 parity 投影与不可变的 archive、verification、finalization
和 close 记录保持一致。

## 范围

- 晋级 WI-526 的读者文档和 reference-parity 行。
- 保留 Runtime 生成记录及对象工程不变。

## 验收

- WI-526 页面和 parity 行引用准确的终态证据路径。
- 文档与治理完整性检查通过。

## 验证

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
bash tests/docs/documentation_acceptance.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
```
