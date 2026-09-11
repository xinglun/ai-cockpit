---
author: AI Cockpit maintainers
title: "WI-794——v0.2.90 受治理的发布收尾"
description: "已发布 v0.2.90 制品的发布后收尾 successor。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-release-recovery
workItemId: WI-794-release-v0-2-90-closure
lastVerifiedBy: WI-794-release-v0-2-90-closure
---

[English](WI-794-release-v0-2-90-closure.md) · [日本語](WI-794-release-v0-2-90-closure.ja.md)

# WI-794——v0.2.90 受治理的发布收尾

WI-794 是 WI-793 的追加式 successor。WI-793 作为历史阻断证据保留，原因是它的 required closure scenario 只有在同一场景要求授权的 `finish` 转换之后才能取得证据。Runtime 已记录真实的 `finish.governance` 停止；本 Work Item 不改写它。

## 收尾边界

v0.2.90 Release 不可变，并绑定已合并的 main 提交
`2d45d6f2e0c99131c6476bd1aa3b3dedc5d81921`。公开安装和 N-1 升级 receipt 保存在 `.ai/evidence/external/v0.2.90/` 下。

本 Work Item 先验证这些可在 finish 前执行的事实，再使用 Runtime 生命周期和明确的合并后 finalization 门完成 archive、close、准确的 branch/worktree 清理及同步 main。不修改 Rust、Runtime、发布制品、标签或前序证据。

## 验证

- `cargo test --locked --workspace`
- 公开 v0.2.90 adopter acceptance 及 isolation/cleanup receipt
- 公开 v0.2.89 → v0.2.90 升级 receipt 及字节一致的历史 digest
- Runtime status/validate 以及最终 `promote_closed_work_item.py --check-all`

## 证据规则

Release 与 adopter receipt 是外部不可变事实。Runtime 所有的 Contract、Summary、Outcome、archive、finalization 和 close 记录都由安装版 Runtime 生成。在转换执行前，不把未来的收尾状态投影为已验证。
