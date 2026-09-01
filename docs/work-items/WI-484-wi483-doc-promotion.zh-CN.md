---
author: AI Cockpit maintainers
title: "WI-484——WI-483 终态文档晋级"
description: "在不重写不可变证据的前提下晋级 WI-483 的终态文档投影。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-484-wi483-doc-promotion
status: implemented
authority: canonical
lastVerifiedBy: WI-484-wi483-doc-promotion
terminalArchive: .ai/work-items/archive/WI-484-wi483-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-484-wi483-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-484-wi483-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-484-wi483-doc-promotion.close.json
---

# WI-484——WI-483 终态文档晋级

本 Work Item 将已验证并关闭的 WI-483 生命周期晋级到三语 Work Item 与
reference-parity 投影。不会修改不可变 Runtime 证据、归档记录或参考源语义。

[English](WI-484-wi483-doc-promotion.md) · [日本語](WI-484-wi483-doc-promotion.ja.md)

## 范围

- 使用仓库辅助脚本晋级 WI-483 的三个文档投影。
- 保持晋级过程确定性，并绑定到准确的终态记录。
- 在归档前登记本 Work Item 自身页面与 parity 行。

## 不在范围内

Runtime/Core 实现、发布或 adopter 产物、新的参考源比对路径，以及不可变治理字节。

## 验收

1. 三个 WI-483 投影包含由证据支持的终态元数据。
2. 三个 reference-parity 行将 WI-483 标记为已实现并链接同一组终态证据。
3. 本 Work Item 具有三语文档和归档前 parity 行。
4. 关闭后 `promote_closed_work_item.py --check-all` 通过。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `git diff --check`
