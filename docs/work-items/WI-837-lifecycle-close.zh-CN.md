---
author: AI Cockpit 维护者
title: "WI-837——生命周期关闭投影"
description: "为已关闭的 WI-837 生命周期关闭集成晋级有证据绑定的阅读文档。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-837-lifecycle-close
lastVerifiedBy: WI-837-lifecycle-close
terminalArchive: .ai/work-items/archive/WI-837-lifecycle-close.contract.json
terminalVerification: .ai/evidence/WI-837-lifecycle-close.verification.json
terminalDecision: .ai/decisions/WI-837-lifecycle-close.close.json
---

[English](WI-837-lifecycle-close.md) · [日本語](WI-837-lifecycle-close.ja.md)

# WI-837——生命周期关闭投影

## 意图与边界

WI-837 集成 post-merge WI-836 关闭流程的 Runtime 终态记录，并完成面向阅读者的文档投影。
投影必须保留精确的 archive、verification 和 close 证据。

源代码行为、产品制品、发布、workspace 验证、远端分支删除和其他 Work Item 不在范围内。

## 验收

- 三个 WI-837 语言页面都是普通非符号链接文件，并描述相同的有界意图、范围、证据和终态。
- 三个 parity ledger 各包含且仅包含一条 WI-837 行，链接到对应页面和不可变终态证据。
- 定向及全仓库 promotion 检查通过，且不改写 Runtime 证据。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-837-lifecycle-close --check`。
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`。
- `git diff --check` 及三个页面和三个 parity ledger 的精确路径/类型检查。
