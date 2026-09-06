---
title: "WI-606 —— WI-605 终态文档投影"
description: "晋级托管文档门要求的三语对等记录。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-606-release-api-auth-doc-projection
workItemId: WI-606-release-api-auth-doc-projection
predecessorWorkItemId: WI-605-release-api-auth
---

[English](WI-606-release-api-auth-doc-projection.md) · [日本語](WI-606-release-api-auth-doc-projection.ja.md)

# WI-606 —— WI-605 终态文档投影

## 目标

保持三语 reference-parity 索引与 Work Item 文档和发布验收修复一致。
这是有边界的文档 successor，不改变 Runtime 行为，也不操作对象工程。

## 边界

范围是三份语言 parity 索引和三份 WI-605 语言文档。前置 Work Item 的 archive、evidence
和 recovery 字节保持不可变。Runtime、发布脚本和安装器行为不在范围内。

## 验证

运行 `bash tests/docs/parity_status_check.sh .` 及 Contract 声明的工作区验证；关闭前必须通过审查 PR 和托管检查。
