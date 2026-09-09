---
author: AI Cockpit maintainers
title: "WI-760——WI-759 文档修复 successor"
description: "补齐 WI-759 缺失的自身 projection 页面，并绑定 successor 文档边界。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
workItemId: WI-760-wi759-doc-repair
lastVerifiedBy: WI-760-wi759-doc-repair
---

[English](WI-760-wi759-doc-repair.md) · [日本語](WI-760-wi759-doc-repair.ja.md)

# WI-760——WI-759 文档修复 successor

## 意图

修复 WI-759 的 reviewed PR #742 合并后发现的自身 projection 页面缺失问题。
该 successor 保留 WI-759 的不可变 evidence，并明确 pending 文档边界。

## 边界

本 Work Item 只修改指定的三语文档页面与 reference-parity projection。不改变
Runtime 行为、产品代码、机器 Contract、授权语义、退出码或 WI-759 历史 evidence。

## 验收

- WI-759 与 WI-760 均有准确的三语页面。
- parity rows 绑定本地 Work Item 页面及相关不可变 evidence。
- pre-archive 文档、治理和状态检查通过，不在 Runtime 生成 close 前宣称终态。
