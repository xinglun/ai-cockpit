---
author: AI Cockpit maintainers
title: "WI-267——Finalization parity 回归修复"
workItemId: WI-267-finalization-parity-regression-repair
description: "修复 hosted quality 暴露的受限 finalization/parity append 回归，同时保留 WI-266 不可变。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-267-finalization-parity-regression-repair
authority: canonical
---

# WI-267——Finalization parity 回归修复

## 意图

Hosted quality 暴露了 WI-266 的回归：finalization 后的 pending parity registry
追加被误判为实现漂移。本 successor 保持 WI-266 不可变，并把该例外收紧为显式、受限的治理追加。

## 范围与证据边界

- finalization 后仅允许 pending parity registry 作为 repository 级治理追加；代码、测试、无关
  evidence 和任意文档变化仍必须拒绝。
- 用真正 append-only 的 finalization history 构建 fixture，并保持 pending-parity 回归（含
  default branch 与 adversarial 场景）通过。
- 同步三种支持语言的 governance gate 文档与 parity 行。
- 完成 hosted review、Runtime finalization verify、精确 cleanup 与结构化 close 后才提升状态。

WI-266 的 archive、evidence、finalization 与 close bytes 保持不可变。本 Work Item 不修改
release-version consistency、quality-route 迁移或全局 Agent/MCP 配置。

## 验证

- `bash tests/ci/governance_integrity_gate_test.sh`
- `bash tests/docs/pending_parity_registry_test.sh`
- `bash tests/docs/parity_status_check_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `cargo fmt --all -- --check`
- `cargo test --locked --workspace`
- 使用显式 `--repo` 的已安装 Runtime lifecycle 与可见人工 Outcome

最终交付必须是可见的 `Outcome: 🟢`、`Outcome: 🟡` 或 `Outcome: 🔴`，并包含状态、未知项、证据、
人工决定和下一步。
