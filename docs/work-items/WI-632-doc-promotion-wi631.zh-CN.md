---
author: AI Cockpit maintainers
title: "WI-632 - WI-631 文档晋级"
description: "将已验证的 WI-631 终态同步到三语读者文档。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-632-doc-promotion-wi631
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-632-doc-promotion-wi631
---

# WI-632 - WI-631 文档晋级

## 意图与边界

将已关闭的 WI-631 参考源重新基线结果同步到英文、中文和日文读者文档。
本 Work Item 只修改声明的九个文档投影文件，不修改 Runtime、参考源字节或对象工程。

## 验收

- 三种语言中的 WI-631 终态与证据链接保持一致。
- 三种 parity 台账在验证前登记本 Work Item。
- 文档与已关闭 Work Item 晋级检查通过。

## 验证

```text
bash tests/ci/recovery_gate_acceptance.sh
python3 tests/docs/promote_closed_work_item.py --repo . --check-all
```

另见：[English](WI-632-doc-promotion-wi631.md) · [日本語](WI-632-doc-promotion-wi631.ja.md)。
