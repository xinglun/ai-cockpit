---
author: AI Cockpit maintainers
title: "WI-292——CI Contract-aware quality gate 恢复"
workItemId: WI-292-ci-contract-aware-gates-recovery
description: "从最新远端默认分支重新交付有界 CI Contract-aware gate，并在 verification 前登记 parity。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-292-ci-contract-aware-gates-recovery
authority: canonical
---

# WI-292——CI Contract-aware quality gate 恢复

## 目的

WI-291 因 hosted quality 拒绝晚登记的 parity 投影而作为不可变恢复历史保留。
本 successor 从最新远端默认分支交付同一有界 Rust gate，并在创建 verification
证据前登记完整的三语 parity 与 Work Item 文档。

## 边界

- 保留 WI-291 archive、evidence、阻断的 finalization 与 recovery bytes。
- Rust 作为 Contract gate authority，同时保留 Python/Cargo shadow checks；本批不删除既有 CI policy。
- 在最终 verification 前绑定实际 provider PR，然后完成 hosted checks、finalization、close
  与精确 branch/worktree 清理。

## 对象工程一致性

本仓库与全新 adopter 必须使用同一份已安装 Runtime、显式 `--repo` context、
fail-closed evidence 和可见的人类 Outcome。

## 验证

执行 `cargo test --locked --workspace`、CI/conformance 与文档 gate、hosted PR checks、
provider finalization verification、close，以及 close 后 status/doctor 检查。
