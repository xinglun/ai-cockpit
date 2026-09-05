---
author: AI Cockpit maintainers
title: "WI-593——WI-592 文档 parity 恢复"
description: "在不重写前置历史的前提下重新交付缺失的 WI-592 parity 登记。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-593-wi591-doc-parity-recovery
lastVerifiedBy: WI-593-wi591-doc-parity-recovery
---

[English](WI-593-wi591-doc-parity-recovery.md) · [日本語](WI-593-wi591-doc-parity-recovery.ja.md)

# WI-593——WI-592 文档 parity 恢复

## 目标

修复文档治理门在 WI-592 归档后发现的三语 parity 登记遗漏，同时保留 WI-592
不可变记录，使当前文档投影可审计。

## 边界

范围仅包括三个 reference-parity 文件以及 WI-592/WI-593 的中英日文档页。Runtime
行为、Cargo 元数据、发布制品、对象工程、全局 Agent/MCP 配置和历史 `.ai` 字节均
不在范围内。

## 验收

1. 三个 parity 文件各自包含一致的 WI-592 恢复行和待完成的 WI-593 行，并有有效
   Work Item 链接及证据边界。
2. WI-592 的归档 Contract、证据、恢复决定和事件历史保持逐字节不变。
3. 三语文档页具有有效 frontmatter，说明 append-only 恢复关系，不声称虚假的终态。
4. 在终态关闭前，`python3 tests/docs/promote_closed_work_item.py --repo <repository>
   --check-all`、`tests/docs/documentation_acceptance.sh` 和
   `tests/docs/parity_status_check.sh` 均通过。

## 验证

使用显式 repository context 运行当前 Contract 声明的文档验收、parity、状态一致性
和锁定 workspace 检查；合并后重新执行完整文档门。
