---
author: AI Cockpit maintainers
title: "WI-522——WI-521 终态文档晋级"
description: "在 WI-521 验证关闭后晋级文档投影，不改写不可变 Runtime 记录。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: human-authorized
workItemId: WI-522-wi521-doc-promotion
lastVerifiedBy: WI-522-wi521-doc-promotion
---

[English](WI-522-wi521-doc-promotion.md) · [日本語](WI-522-wi521-doc-promotion.ja.md)

## 目标

将 WI-521 的读者文档和 parity 行晋级为 Runtime 已记录的确切终态。

WI-522 是不可变的前驱记录。归档推进分支 HEAD 后，其合并前
finalization 变为过期；Runtime recovery 决策记录在
`.ai/decisions/WI-522-wi521-doc-promotion.recovery.json`。WI-523 从最新已评审
基线重新交付同一文档投影；不会改写前驱证据，也不会把前驱伪装成新的成功。

## 范围

- 三个 WI-521 Work Item 页面。
- 三个 `docs/reference/reference-parity` 投影。
- 三语 WI-522 记录自身。

Runtime 源码、参考实现、对象工程、版本发布、全局 Agent/MCP 配置以及
WI-521 生成记录均不在范围内。

## 验收

- `promote_closed_work_item.py --check-all` 不再报告 WI-521 投影过期。
- WI-521 页面与 parity 行均显示已实现，并绑定准确的 archive、verification、
  finalization、close 证据。
- 文档、parity、状态一致性和治理完整性检查通过，且不可变 Runtime 记录不变。

## 验证

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/docs/work_item_status_consistency.py --repo <repo>
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
git diff --check
```
