---
author: AI Cockpit maintainers
title: "WI-630——WI-629 文档晋级"
description: "将已验证的 WI-629 终态同步到三语读者文档。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-630-doc-promotion-wi629
status: in_progress
authority: canonical
lastVerifiedBy: WI-630-doc-promotion-wi629
---

# WI-630——WI-629 文档晋级

## 意图与边界

将已关闭的 WI-629 参考源重新基线批次同步到中文、英文和日文读者文档。
本 Work Item 只修改文档，不修改 Runtime 代码、参考源字节、生成的治理记录或对象工程。

## 验收

- 三种语言的 WI-629 状态和终态证据链接均为最新。
- 本 Work Item 在验证前已登记到三种语言的对等台账。
- 文档和已关闭 Work Item 晋级检查通过。

## 验证

```text
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
bash tests/docs/work_item_status_consistency_test.sh
python3 tests/docs/promote_closed_work_item.py --check-all
```

参见：[English](WI-630-doc-promotion-wi629.md) ·
[日本語](WI-630-doc-promotion-wi629.ja.md)。
