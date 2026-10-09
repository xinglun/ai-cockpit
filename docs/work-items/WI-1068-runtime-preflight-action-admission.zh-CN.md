---
author: AI Cockpit maintainers
title: "WI-1068——Runtime preflight action admission"
description: "只有当前 Work Item Contract 与仓库快照绑定的 preflight 为 green 时，才允许 finish。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1068-runtime-preflight-action-admission
status: archived
authority: historical-projection
---

# WI-1068——Runtime preflight action admission

本页是已归档 Work Item 的文档投影。生命周期、验证和 close 的不可变记录
仍以 `.ai/work-items/archive/WI-1068-runtime-preflight-action-admission.*` 为准。

## 目标与行为

此 Work Item 让 Status projection 与 `finish` 所要求的 green preflight 保持一致。
当验证证据有效，但 preflight 尚未运行、结果不是 green，或绑定的是旧 Contract
或仓库快照时，Runtime 会提供 `run_preflight` 并阻止 `finish`。只有针对当前
Contract 和仓库快照重新得到 green 的 preflight，才允许进入 `finish`。现有
verification、governance-control 和 human-review 检查继续生效。

实现涉及 `crates/cockpit-repository/src/status_projection.rs` 及其测试、
`crates/cockpit-cli/tests/collaboration_consistency.rs` 和 `.ai/policy.json`。
测试 fixture 显式将 `directory.path()` 绑定为 `&Path` 后再拼接路径，避免
assessment 错报 `repository_material_inspection_unavailable`。

## 归档结果

归档 Contract 声明了 `cargo fmt --all -- --check`、
`cargo test --locked --workspace` 和 workspace Clippy。归档 Outcome 将验证状态
记为 `verified`；task report 保留用户可见收益为 unknown，因为 Work Item owner
没有声明该收益。

[English](WI-1068-runtime-preflight-action-admission.md) ·
[日本語](WI-1068-runtime-preflight-action-admission.ja.md)
