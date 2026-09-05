---
author: AI Cockpit maintainers
title: "WI-590——WI-589 终态文档晋级"
description: "在 WI-589 验证关闭后晋级其文档投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-590-wi589-doc-promotion
lastVerifiedBy: WI-590-wi589-doc-promotion
---

[English](WI-590-wi589-doc-promotion.md) · [日本語](WI-590-wi589-doc-promotion.ja.md)

# WI-590——WI-589 终态文档晋级

## 目标

在 WI-589 的不可变 archive、verification、finalization 和 close 证据
通过验证后，将其三语 Work Item 与 reference-parity 文档投影晋级为终态。
本 Work Item 只负责文档投影。

## 边界

Runtime 行为、对象工程、全局 Agent/MCP 配置以及生成的 evidence 或 decision
字节不在范围内。治理事实必须从 Runtime 不可变记录推导。

## 验收

1. WI-589 的中英日页面包含终态证据路径并报告已实现，不改变语义内容。
2. 三个 WI-589 parity 行包含匹配的终态证据路径。
3. WI-590 自身页面和 parity 行已登记为确定性的自终态文档晋级，不因关闭
   本 Work Item 引入新的文档债务。

## 验证

使用显式 repository context 运行
`python3 tests/docs/promote_closed_work_item.py --repo <repository> --check-all`、
`tests/docs/documentation_acceptance.sh` 和 `tests/docs/parity_status_check.sh`。
