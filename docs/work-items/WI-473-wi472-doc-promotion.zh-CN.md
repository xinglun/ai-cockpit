---
author: AI Cockpit maintainers
title: "WI-473——WI-472 终态文档提升"
description: "在发布前保持终态 Work Item 与 parity 投影完整。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-473-wi472-doc-promotion
status: implemented
authority: authorized
lastVerifiedBy: WI-473-wi472-doc-promotion
terminalArchive: .ai/work-items/archive/WI-473-wi472-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-473-wi472-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-473-wi472-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-473-wi472-doc-promotion.close.json
---

# WI-473——WI-472 终态文档提升

## 意图与边界

将已验证的 WI-472 生命周期提升到面向读者的文档，并让 recovery 与当前
Work Item 的 parity 登记可审计。本 Work Item 只修改文档投影；不可变的
`.ai` 记录、Runtime 代码、CI、发布产物和对象工程均不在范围内。

## 范围

- 在关闭后提升 WI-472 的英文、简体中文和日文页面。
- 在所有 parity 台账中保留 WI-471 的权威哈希 recovery receipt。
- 在归档/关闭前登记本 Work Item 及其终态路径。

## 验收

1. 三语 WI-472 页面和 parity 行绑定终态 receipt。
2. 三语 WI-473 页面和预归档 parity 行通过治理完整性门禁。
3. 文档与参考清单检查在干净分支通过。
4. 不修改不可变治理字节，也不修改对象工程。

## 验证

- `cargo test --locked --workspace`
- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`

## 恢复边界

如果投影不完整，保留不可变记录，通过显式 amendment 与 revalidation 修复
当前文档 Work Item。
