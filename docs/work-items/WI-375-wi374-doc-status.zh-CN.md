---
author: AI Cockpit maintainers
title: "WI-375——WI-374 终态文档提升"
description: "为确定性的关闭后提升准备三语 Work Item 与 parity 投影。"
workItemId: WI-375-wi374-doc-status
audience: [maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-375-wi374-doc-status
capabilityClaims: [documentation_governance]
---

# WI-375——WI-374 终态文档提升

[English](WI-375-wi374-doc-status.md) · [日本語](WI-375-wi374-doc-status.ja.md)

## 意图

通过仓库明确的关闭后提升 helper，保持已关闭 WI-374 的文档与 parity ledger 真实一致。本 Work Item 只准备和验证仓库文档边界；关闭后由 helper 写入机器拥有的终态投影。

## 范围与边界

- 维护三语 WI-374 投影及三份 parity ledger，使其符合提升 helper 要求的关闭前形式。
- 验证文档、parity 和治理完整性 gate。
- 保留 WI-374 不可变 Runtime evidence，并遵循文档规定的 `close → promote closed docs → terminal CI` 顺序。

Runtime 语义、发布 artifact、历史 evidence 字节以及全局 Agent/MCP 配置不属于本 Work Item。

## 验收

1. 三语 WI-374 文档和 parity 行为合法的提升前投影，并链接不可变终态 receipt。
2. 关闭前文档、parity 和治理完整性检查通过。
3. 审查合并和关闭后，提升 helper 可以只确定性写入终态 frontmatter 与 parity 行。
4. Work Item 完成审查合并、finalization、close 和精确清理。

## 验证边界

Runtime 记录本 Work Item 的 Contract、checkpoint、verification、archive、finalization 与 close evidence。`promote_closed_work_item.py` 是显式的关闭后文档投影，不会重写 Runtime truth 或历史 evidence。
