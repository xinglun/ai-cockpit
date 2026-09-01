---
author: AI Cockpit maintainers
title: "WI-479——WI-478 终态文档 promotion"
description: "晋级已关闭的 WI-478 发布记录，并在本 Work Item close 前登记自身文档。"
audience:
  - maintainer
  - reviewer
  - adopter
status: recovered
authority: human-authorized
lastVerifiedBy: WI-479-wi478-doc-promotion
workItemId: WI-479-wi478-doc-promotion
---

# WI-479——WI-478 终态文档 promotion

这是一个仅限文档的 Work Item：将已关闭的 WI-478 发布记录晋级到面向
读者的 projection，并在自身 close 前登记生命周期。它不改写不可变的
Runtime 记录，也不修改任何 adopter 工程。

[English](WI-479-wi478-doc-promotion.md) · [日本語](WI-479-wi478-doc-promotion.ja.md)

## 范围

- 让三语 WI-478 Work Item 页面和三语 reference-parity 台账继续绑定
  不可变的 WI-478 lifecycle 记录。
- 在本 Work Item close 待完成期间，将本 Work Item 登记到三语 parity
  台账；verified close 后再晋级自身登记。
- 保持 close 后文档 promotion 检查具有确定性。

## 不在范围内

Runtime 或 protocol 行为、发布打包、CI 策略、参考源实现、对象工程、
全局 Agent/MCP 配置，以及不可变 Contract、evidence、archive、
finalization、recovery 或 close bytes。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-478-release-v0-2-57`
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/docs/documentation_acceptance.sh`

本页面的终态 status 和 evidence 链接只在 reviewed merge、finalization
与 close 完成后晋级。
