---
author: AI Cockpit maintainers
title: "WI-451——WI-450 文档 promotion"
workItemId: WI-451-wi450-doc-promotion
description: "将已关闭的 WI-450 生命周期提升为终态文档投影。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-451-wi450-doc-promotion
---

# WI-451——WI-450 文档 promotion

将已关闭的 WI-450 生命周期提升到三语 Work Item 文档和 reference-parity
投影，同时保留 Runtime truth 与不可变终态证据。

[English](WI-451-wi450-doc-promotion.md) · [日本語](WI-451-wi450-doc-promotion.ja.md)

## 范围

- 提升 WI-450 的英文、中文和日文文档。
- 将三份 WI-450 parity 行从进行中提升为已实现。
- 保持 Runtime 行为与终态证据不变。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-450-closed-work-item-doc-promotion`
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/docs/documentation_acceptance.sh`
