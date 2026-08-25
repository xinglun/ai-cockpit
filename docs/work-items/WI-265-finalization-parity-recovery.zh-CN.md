---
author: AI Cockpit maintainers
title: "WI-265——Finalization 与 parity 恢复"
workItemId: WI-265-finalization-parity-recovery
description: "在不改写不可变历史的前提下恢复 WI-263 的 closure boundary。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-265-finalization-parity-recovery
terminalArchive: .ai/work-items/archive/WI-265-finalization-parity-recovery.contract.json
terminalVerification: .ai/evidence/WI-265-finalization-parity-recovery.verification.json
terminalFinalization: .ai/decisions/WI-265-finalization-parity-recovery.finalize.d2ffd7299322f97652f941f2ba7a640ba750d0aa9d625cbd4edd4f169a5ec20d.json
terminalDecision: .ai/decisions/WI-265-finalization-parity-recovery.close.json
authority: canonical
---

# WI-265——Finalization 与 parity 恢复

## 意图

WI-263 的 archive 必须保持不可变，但合并后留下了过时的 pre-merge
finalization head，parity 投影也仍写着等待合并。本 successor 只负责新的
closure boundary；不改写 WI-263，也不把缺少 close receipt 当成完成决定。

## 范围与证据边界

- 通过 Runtime 记录 WI-263 的 successor recovery decision。
- 在 archive 前登记英文、简体中文、日文三语 parity；只有在 merge 与精确
  cleanup 证据出现后才提升为已实现。
- 在 verification/archive 前使用 `work-item finalize-plan` 绑定本 Work Item
  自己的 branch、worktree、provider 与 reviewed PR。
- 只从 reviewed merge head 完成 hosted PR 生命周期和精确 cleanup；缺失、过时
  或 foreign receipt 都必须阻断。Runtime 已记录 merge observation、删除 transition
  与结构化 close decision。

WI-263 的 archive、Outcome、Summary、Events、verification、旧 recovery 与旧
finalization bytes 都是历史记录，绝不修改。

## 失败与恢复

缺少任一语言 parity、缺少 finalization receipt，或记录的 head 漂移到 reviewed
checkout 之外时，治理门必须 fail closed。这个已关闭的 successor 推进 closure
boundary，但不改变 predecessor bytes。

## 验证

- `bash tests/ci/governance_integrity_gate_test.sh`
- `bash tests/ci/docs_parity_regression_test.sh`
- `cargo fmt --all -- --check`
- `cargo test --locked --workspace`
- 使用显式 `--repo` 的已安装 Runtime 执行 `inspect`、`status`、`doctor`、
  `agent doctor`、生命周期、finalization verify、close 与 `work-item outcome`

最终人工交付必须显示 `Outcome: 🟢`、`Outcome: 🟡` 或 `Outcome: 🔴`，并包含状态、
未知项、证据、人工决定和下一步。最终 parity、删除 transition 与 close receipt
共同构成这个 Work Item 的终态记录。
