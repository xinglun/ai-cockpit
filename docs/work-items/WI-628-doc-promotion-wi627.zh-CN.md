---
author: AI Cockpit maintainers
title: "WI-628——WI-627 文档晋级"
description: "将已验证的 WI-627 终态同步到三语读者文档。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-628-doc-promotion-wi627
status: in_progress
authority: canonical
lastVerifiedBy: WI-628-doc-promotion-wi627
---

# WI-628——WI-627 文档晋级

## 意图

将已关闭的 WI-627 参考源重新基线结果同步到中、英、日三语读者文档，不修改
Contract、evidence、archive、finalization 或 decision bytes。

## 边界

本 Work Item 仅处理文档：WI-627 三语页面、本 Work Item 三语页面，以及三份
reference parity 台账。Runtime 代码、测试、CI、参考源台账、生成的治理记录以及
全局 Agent/MCP 配置均不在范围内。

## 验收

- 三语 WI-627 读者文档的状态和终态证据链接保持最新。
- 本 Work Item 在验证前拥有对应的中、英、日页面，并登记在三份 parity 台账中。
- 晋级后的文档检查全部通过。

## 验证

```text
python3 tests/docs/promote_closed_work_item.py --check-all
python3 tests/docs/reference_comparison_metadata_test.py
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
```

另见：[English](WI-628-doc-promotion-wi627.md) ·
[日本語](WI-628-doc-promotion-wi627.ja.md)。
