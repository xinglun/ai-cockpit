---
author: AI Cockpit maintainers
title: WI-639——WI-638 文档晋级
description: 在不改变治理事实的前提下晋级 WI-638 的已验证终态文档投影。
audience: [maintainer, reviewer, adopter]
workItemId: WI-639-wi638-documentation-promotion
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-639-wi638-documentation-promotion
---

# WI-639——WI-638 文档晋级

本 Work Item 使用 `tests/docs/promote_closed_work_item.py` 的确定性投影，更新
WI-638 的三语 Work Item 页面和三语参考对等行。只修改面向读者的投影；Contract、
verification、archive、finalization、close 等不可变记录仍是权威，不会被重写。

投影只属于当前 Rust 仓库，不复制参考源实现，也不修改对象/adopter 工程。

## 验收

- helper 晋级三份 WI-638 Work Item 页面。
- helper 晋级三份 WI-638 对等行。
- 不可变生命周期记录保持字节不变。
- 文档、对等和 post-close 晋级检查通过。

参见：[English](WI-639-wi638-documentation-promotion.md) · [日本語](WI-639-wi638-documentation-promotion.ja.md)。
