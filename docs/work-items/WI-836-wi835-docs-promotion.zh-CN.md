---
author: AI Cockpit 维护者
title: "WI-836——WI-835 文档 promotion"
description: "为已关闭的 WI-835 晋级有证据绑定的阅读文档。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-836-wi835-docs-promotion
lastVerifiedBy: WI-836-wi835-docs-promotion
terminalArchive: .ai/work-items/archive/WI-836-wi835-docs-promotion.contract.json
terminalVerification: .ai/evidence/WI-836-wi835-docs-promotion.verification.json
terminalDecision: .ai/decisions/WI-836-wi835-docs-promotion.close.json
---

[English](WI-836-wi835-docs-promotion.md) · [日本語](WI-836-wi835-docs-promotion.ja.md)

# WI-836——WI-835 文档 promotion

## 意图与边界

WI-836 将已关闭的 WI-835 生命周期清理处置投影到英文、简体中文、日文阅读页面和
reference-parity 行。投影必须链接到 Runtime 所拥有的精确 archive、verification 和
close 记录，不改写这些记录。

Runtime 行为、发布制品、分支删除、历史证据和其他 Work Item 不在范围内。

## 验收

- 三个 WI-835 语言页面都是普通非符号链接文件，并描述相同的意图、范围、证据和终态。
- 三个 parity ledger 各包含且仅包含一条 WI-835 行，链接到对应页面以及不可变 archive、
  verification 和 close 证据。
- 验证前已存在三个 WI-836 语言页面及其 parity 行的 pre-archive 形式。
- WI-835 定向 promotion 检查通过，且不改写 Runtime 证据。
- 全仓库 promotion 检查通过，或明确报告有独立证据支持的既有问题。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-835-lifecycle-cleanup --check`。
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`。
- `git diff --check` 及六个页面和三个 parity ledger 的精确路径/类型检查。
