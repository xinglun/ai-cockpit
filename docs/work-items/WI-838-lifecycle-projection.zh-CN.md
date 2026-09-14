---
author: AI Cockpit maintainers
title: "WI-838——生命周期投影守卫集成"
description: "集成有证据绑定的生命周期投影守卫，并完成阅读文档契约。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-838-lifecycle-projection
lastVerifiedBy: WI-838-lifecycle-projection
terminalArchive: .ai/work-items/archive/WI-838-lifecycle-projection.contract.json
terminalVerification: .ai/evidence/WI-838-lifecycle-projection.verification.json
terminalDecision: .ai/decisions/WI-838-lifecycle-projection.close.json
---

[English](WI-838-lifecycle-projection.md) · [日本語](WI-838-lifecycle-projection.ja.md)

# WI-838——生命周期投影守卫集成

## 意图与边界

WI-838 集成使 WI-837 投影保持当前所需的 Runtime 终态记录，并在本工作项进入终态生命周期前登记自身文档投影。

Runtime 源码行为、发布产物、工作区验证、远端分支删除及无关工作项不属于本工作项。

## 验收

- WI-836 和 WI-837 的转移记录保持原始字节，并绑定原始证据。
- WI-837 的三语页面和 parity 行由官方 helper 置为当前状态。
- WI-838 的三语页面和 parity 行在验证前存在，并在关闭后从 Runtime 终态记录完成投影。
- 仓库范围文档 promotion 通过，且不重新运行产品构建或工作区验证。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-837-lifecycle-close --check`。
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`。
- `git diff --check` 以及投影页面为普通非符号链接文件的检查。
