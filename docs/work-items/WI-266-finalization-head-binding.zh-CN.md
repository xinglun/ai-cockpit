---
author: AI Cockpit maintainers
title: "WI-266——Finalization head binding successor"
workItemId: WI-266-finalization-head-binding
description: "将 repository finalization receipt 绑定到精确的 reviewed provider head。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-266-finalization-head-binding
authority: canonical
---

# WI-266——Finalization head binding successor

## 意图

失败的 WI-261 证明，仅有自洽的 finalization receipt 还不够；reviewed
checkout 必须是 receipt 的精确 head，只有 Runtime 治理记录本身的受限 append
可以例外。本 successor 从最新 default branch 重新交付该控制，同时保持失败
predecessor 的不可变历史。

## 范围与证据边界

- 将 feature 与 pull-request finalization receipt 绑定到 provider reviewed
  checkout head。
- 只允许 canonical Runtime finalization append 及同一 Work Item 的明确受限
  治理记录；代码或无关 repository 漂移必须拒绝。
- 在 archive 前保持 governance-integrity fixture、回归测试、参考文档及中英日
  parity 同步。
- 只有完成 hosted review、Runtime finalization verify、精确 cleanup 与结构化
  close 后，才提升为已实现。

失败的 WI-261 archive、evidence、branch 与 PR 仅作为历史保留。本 Work Item
不迁移 quality-route 到 Rust，也不修改全局 Agent/MCP 配置。

## 失败与恢复

缺少或 foreign reviewed head、finalization 后代码漂移、无关文件、格式错误的
transition，或缺少 parity 时，治理门必须 fail closed。只有绑定到同一 Work
Item 与 reviewed head 的 append-only 治理证据才可接受。

## 验证

- `bash tests/ci/governance_integrity_gate_test.sh`
- `bash tests/docs/parity_status_check_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `cargo fmt --all -- --check`
- `cargo test --locked --workspace`
- 使用显式 `--repo` 的已安装 Runtime 生命周期、finalization verify 与人工
  Outcome

最终交付必须是可见的 `Outcome: 🟢`、`Outcome: 🟡` 或 `Outcome: 🔴`，并包含状态、
未知项、证据、人工决定和下一步。
