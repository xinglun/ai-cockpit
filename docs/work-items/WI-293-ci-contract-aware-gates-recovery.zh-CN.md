---
author: AI Cockpit maintainers
title: "WI-293——CI Contract-aware quality gate 恢复"
workItemId: WI-293-ci-contract-aware-gates-recovery
description: "从最新远端默认分支重新交付有界 CI Contract-aware gate，并在 verification 前登记 parity。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-294-lifecycle-recovery-state-machine
authority: canonical
---

# WI-293——CI Contract-aware quality gate 恢复

## 目的

WI-293 作为不可变的 recovered history 保留。合并的 CI gate 由 PR #253 记录；
合并后发现的生命周期恢复缺口由有界 successor WI-294 负责，两个 Work Item
都不会重写 predecessor bytes。

## 边界

- 保留 WI-293 archive、evidence、阻断的 finalization 与 recovery bytes。
- Rust 作为 Contract gate authority，同时保留 Python/Cargo shadow checks；本批不删除既有 CI policy。
- 在最终 verification 前绑定实际 provider PR，然后完成 hosted checks、finalization、close
  与精确 branch/worktree 清理。

## 对象工程一致性

同一份已安装 Runtime、显式 `--repo` context、fail-closed evidence 和可见的人类
Outcome 已治理合并交付；WI-294 记录关闭阶段发现的 recovery boundary。

## 验证

执行 `cargo test --locked --workspace`、CI/conformance 与文档 gate、hosted PR checks、
provider finalization verification、close，以及 close 后 status/doctor 检查。
