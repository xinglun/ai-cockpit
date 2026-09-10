---
author: AI Cockpit maintainers
title: "WI-787 —— WI-786 finalization 恢复"
description: "为已合并的 WI-786 successor delivery 完成 Runtime 治理的 finalization 边界。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-successor-recovery
workItemId: WI-787-parity-finalization-recovery
lastVerifiedBy: WI-787-parity-finalization-recovery
---

[English](WI-787-parity-finalization-recovery.md) · [日本語](WI-787-parity-finalization-recovery.ja.md)

# WI-787 —— WI-786 finalization 恢复

## 意图与边界

WI-787 是已归档 WI-786 parity 注册修复的明确 successor。WI-786 的 archive、Outcome、验证证据和 finalization 记录保持不可变。之所以需要 successor，是因为第一条当前 Runtime receipt 在合并后以 `retained` 记录；必须保持 pre-merge blocked、合并观察、清理和 close 的有序边界 fail-closed，不能靠颜色推断或改写历史。

恢复绑定为
`.ai/decisions/WI-786-parity-registration-repair.recovery.96b7c5cd733a2b74247c4c2520191ba28e86d0a8c3b7ea4793821dec2df42c24.json`。
本页面和三份 parity ledger 是 WI-787 的全部文档范围。产品行为、版本发布和前驱字节不在范围内。

## 受治理的交付

该 successor 会在验证前绑定自己的 PR 上下文，使用独立分支和 worktree，并通过 Runtime 记录 provider merge、准确清理、finalization 验证和结构化 close。面向人的状态必须区分不可变的前驱恢复和 successor 的当前证据。任何人类建议都不能绕过 Runtime 的执行门槛；没有证据时不得推断用户可见收益或性能提升。

## 验证

声明的仓库检查包括 parity-status gate、documentation acceptance 和 `cargo test --locked --workspace`。只有 successor 具备已验证 archive、经过评审的 PR、准确的 provider finalization、`finalize-verify` 和 close 决定后，才晋级本页面。WI-786 只能通过已完成的 successor 恢复路径关闭。
