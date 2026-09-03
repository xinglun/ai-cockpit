---
author: AI Cockpit maintainers
title: "WI-532——WI-531 终态文档晋级"
description: "在 WI-531 验证关闭后晋级其读者文档。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-532-wi531-doc-promotion
lastVerifiedBy: WI-532-wi531-doc-promotion
---

[English](WI-532-wi531-doc-promotion.md) · [日本語](WI-532-wi531-doc-promotion.ja.md)

## 目标

使 WI-531 三语页面和 parity 行与不可变的 archive、verification、
finalization 和 close 证据保持一致。

## 范围

- 对 WI-531 运行官方已关闭 Work Item 文档晋级脚本。
- 保持 Runtime 记录、源代码行为和对象工程不变。

## 验收

- WI-531 页面和 parity 行绑定准确的终态证据。
- 文档与治理完整性检查通过。

## 验证

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-531-historical-direct-merge-apply
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/ci/governance_integrity_gate.py --repo <repo>
```
