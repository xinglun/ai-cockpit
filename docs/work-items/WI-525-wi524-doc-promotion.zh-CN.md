---
author: AI Cockpit maintainers
title: "WI-525——WI-524 终态文档晋级"
description: "使用精确终态证据绑定晋级已关闭的 WI-524 文档投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-525-wi524-doc-promotion
lastVerifiedBy: WI-525-wi524-doc-promotion
---

[English](WI-525-wi524-doc-promotion.md) · [日本語](WI-525-wi524-doc-promotion.ja.md)

## 目标

将三语 WI-524 Work Item 页面和 parity 行同步到不可变的 archive、verification、
resource-finalization 与 close 证据。

## 范围

- 晋级 WI-524 页面及三份 reference-parity 投影。
- 保持历史证据字节、Runtime 行为、对象仓库和全局 Agent/MCP 配置不变。
- 确保本投影在终态关闭后仍可审计。

## 验收

- 每个 WI-524 页面和 parity 行都绑定精确终态证据路径。
- 已关闭 Work Item 晋级、文档和治理门禁通过。
- 不修改对象仓库状态或历史证据。

## 验证

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/docs/work_item_status_consistency.py --repo <repo>
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
git diff --check
```
