---
author: AI Cockpit maintainers
title: "WI-458——WI-457 文档 promotion"
workItemId: WI-458-wi457-doc-promotion
description: "将已关闭的 WI-457 生命周期提升为所需的三语文档投影。"
audience: [adopter, maintainer, reviewer]
status: in progress
authority: human-authorized
lastVerifiedBy: WI-458-wi457-doc-promotion
---

# WI-458——WI-457 文档 promotion

本 Work Item 修复 WI-457 close 后 `promote_closed_work_item --check-all` 发现的
文档投影遗漏：增加三语终态页面和 parity 行，在其完成后删除临时 registry bridge，
并保持不可变 Runtime evidence 不变。

[English](WI-458-wi457-doc-promotion.md) · [日本語](WI-458-wi457-doc-promotion.ja.md)

## 范围

- 提升 WI-457 的英文、中文和日文 Work Item 页面。
- 在三语 reference-parity ledger 中加入 WI-457 终态行。
- parity 行存在后删除 `pending-parity-registry.json` 中的 WI-457 条目。
- 保持 Runtime 行为、`.ai` 生命周期记录、历史证据和 WI-445 负责的 inventory 不变。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `cargo test --locked --workspace`
