---
author: AI Cockpit maintainers
title: "WI-534——WI-533 终态文档晋级"
description: "在 WI-533 验证关闭后晋级其读者文档。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-534-wi533-doc-promotion
lastVerifiedBy: WI-534-wi533-doc-promotion
---

[English](WI-534-wi533-doc-promotion.md) · [日本語](WI-534-wi533-doc-promotion.ja.md)

## 目标

使 WI-533 三语页面和 parity 行与不可变的 archive、verification、
finalization 和 close 证据保持一致。

## 范围

- 对 WI-533 运行官方已关闭 Work Item 文档晋级脚本。
- 保持 Runtime 记录、源代码行为、发布 artifact 和对象工程不变。

## 验收

- WI-533 页面和 parity 行绑定准确的终态证据。
- 文档与治理完整性检查通过。

## 验证

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-533-release-v0-2-66
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
tests/docs/documentation_acceptance.sh
tests/docs/parity_status_check.sh
```
