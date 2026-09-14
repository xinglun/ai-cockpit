---
author: AI Cockpit 维护者
title: "WI-835——生命周期清理处置"
description: "为剩余发布分支和工作树记录有证据绑定的处置结果。"
audience: [maintainer, reviewer]
status: implemented
authority: authorized
workItemId: WI-835-lifecycle-cleanup
lastVerifiedBy: WI-835-lifecycle-cleanup
terminalArchive: .ai/work-items/archive/WI-835-lifecycle-cleanup.contract.json
terminalVerification: .ai/evidence/WI-835-lifecycle-cleanup.verification.json
terminalDecision: .ai/decisions/WI-835-lifecycle-cleanup.close.json
---

[English](WI-835-lifecycle-cleanup.md) · [日本語](WI-835-lifecycle-cleanup.ja.md)

# WI-835——生命周期清理处置

## 意图与边界

WI-835 为当前每条非 main 远端发布分支及其本地工作树记录有证据绑定的
处置结果。脏、分叉、未合并、绑定不明确或证据不足的资源必须保留；只有
精确的 reviewed merge、Runtime 生命周期和干净资源事实全部得到证明时才可删除。

Runtime 生成的生命周期记录和历史证据保持不可变；release tag、Release、产品行为
及无关源码变更不在本 Work Item 范围内。

## 验收

- 每条剩余远端分支都有包含 PR 状态、合并事实、工作树状态及阻塞或清理证据的处置记录。
- 使用 Runtime 生成的 archive、finalization 和 close 记录；不改写或删除历史证据字节。
- 只有在 finalization 验证后，才删除精确匹配且干净的已合并分支和工作树资源。
- 清理 Work Item 通过 reviewed PR 集成、archive、close 和文档 promotion 检查。

## 验证

- 使用仓库绑定的处置验证器运行 Runtime `verify`。
- `git ls-remote --heads origin 'codex/*'` 和本地 `git worktree list`。
- 对每条分支核对 GitHub PR 状态及合并事实。
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`。
