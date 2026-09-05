---
author: AI Cockpit maintainers
title: "WI-592——WI-591 终态文档晋级"
description: "在验证证据记录后晋级 WI-591 发布投影，并将后续 parity 恢复作为不可变历史保留。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: canonical
workItemId: WI-592-wi591-doc-promotion
lastVerifiedBy: WI-592-wi591-doc-promotion
---

[English](WI-592-wi591-doc-promotion.md) · [日本語](WI-592-wi591-doc-promotion.ja.md)

# WI-592——WI-591 终态文档晋级

## 目标

在 WI-591 的不可变归档和验证证据记录后，晋级英文、中文、日文发布与
reference-parity 投影。随后 CI 发现 WI-592 自身缺少 parity 登记；该事实作为
不可变历史保留，并由后继 WI-593 重新交付。

## 边界

本记录只负责文档投影。Runtime 行为、发布制品、对象工程、全局 Agent/MCP 配置
以及生成的归档/证据/决定字节均不在范围内，也不会被修改。

## 验收

1. WI-591 发布文档在三种语言中依据终态 Runtime 证据一致晋级。
2. 在不重写 WI-592 归档字节的前提下记录恢复边界和后继 WI-593。
3. 文档门可重复执行；缺失 parity 登记必须形成有界后继任务，而不能静默改变历史。

## 验证

使用显式 repository context 运行 `python3 tests/docs/promote_closed_work_item.py
--repo <repository> --check-all`、`tests/docs/documentation_acceptance.sh` 和
`tests/docs/parity_status_check.sh`。
