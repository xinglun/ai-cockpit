---
author: AI Cockpit maintainers
title: "WI-465——已关闭 Work Item 文档晋级"
workItemId: WI-465-closed-work-item-doc-promotion
description: "在不改写不可变记录的前提下，将已关闭 Work Item 证据晋级到面向读者的文档。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-465-closed-work-item-doc-promotion
terminalArchive: .ai/work-items/archive/WI-465-closed-work-item-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-465-closed-work-item-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-465-closed-work-item-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-465-closed-work-item-doc-promotion.close.json
---

# WI-465——已关闭 Work Item 文档晋级

本 Work Item 修复 WI-464 恢复重试发现的关闭后文档投影缺口。只从 Runtime
不可变的 archive、verification、finalization 和 close 证据晋级，不改写这些
记录。

[English](WI-465-closed-work-item-doc-promotion.md) · [日本語](WI-465-closed-work-item-doc-promotion.ja.md)

## 范围

- 晋级 WI-464 重试的三语文档页面和 parity 行。
- 保持本 Work Item 自身的三语页面与 parity 注册，使关闭后的同一晋级检查
  不会再次产生文档债务。
- 保留 canonical gate manifest 中的已关闭 Work Item 检查及其 stale 投影回归测试。
- Runtime 行为、参考源字节、对象工程和不可变 `.ai` 证据不在范围内。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `bash tests/docs/promote_closed_work_item_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `cargo test --locked --workspace`

本页面的终态字段只会在本 Work Item 通过审查合并、归档、终结和关闭边界后，
由同一晋级流程写入。
