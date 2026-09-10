---
author: AI Cockpit maintainers
title: "WI-793——v0.2.90 发布恢复"
description: "在 WI-792 不可消费的 retry 证据被保留后，从最新默认分支重新交付 v0.2.90。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-release-recovery
workItemId: WI-793-release-v0-2-90-recovery
lastVerifiedBy: WI-793-release-v0-2-90-recovery
---

[English](WI-793-release-v0-2-90-recovery.md) · [日本語](WI-793-release-v0-2-90-recovery.ja.md)

# WI-793——v0.2.90 发布恢复

## 恢复边界

WI-793 是 WI-792 的严格绑定 successor。WI-792 的 retry 回执是不可变证据，
继续保留在 predecessor 分支；Runtime 因相同的未来时间戳使旧 stale candidate
排序在当前 retry 之后而拒绝消费它。本 successor 从最新同步的 `origin/main`
启动，不重写 predecessor bytes。

## 意图与发布边界

在 A–E 信任评审完成后发布 v0.2.90。发布必须来自审查后的 PR 和同步的默认分支，
使用新的 annotated tag，并且只有下载的公开产物可作为验收依据。本 Work Item 不改变
Rust 或 Runtime 行为，不复用 tag，不重写治理历史，也不削弱发布门槛。

## 发布说明分类

- 交流修正：Outcome 原因区分实际治理缺口；finalization 恢复动作保留具体条件；
  CLI/MCP、摘要/完整报告和三语检查验证人类语义。
- 验证覆盖：A–E 反例、无历史交接、历史兼容、受控清理和确定性并发变化保持为可执行证据。
- 架构一致性：Outcome 输入在一个有界且重新校验的 ObservationContext 边界内组装，纯渲染保持无 I/O。
- 性能诊断：主路径阶段耗时及受控字节/哈希/Git/进程计数覆盖 macOS/Linux 文件系统路径。
  这些是诊断数据，不宣称未经验证的提速；被拒绝的优化仍保持原结论。

## 验收

- Cargo 元数据、lockfile、当前三语投影和 reference metadata 对同一 v0.2.90 identity 一致。
- 审查后的 PR 在合并前通过 hosted route、quality、behavioral、Windows 和任务完成检查。
- annotated tag 与公开 Release manifest 绑定已合并的 main commit；产物、checksum、SBOM 和 attestation 通过门槛。
- 公开安装和 N-1 升级只使用隔离根目录中的下载产物并证明清理；workspace 构建不能替代发布证据。
- successor 完成 archive、finalization、close，精确清理及 main 同步得到验证。

## 验证

- `bash tests/release/version_consistency.sh --repo <repo>`
- `python3 tests/docs/reference_comparison_metadata_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `cargo test --locked --workspace`
- `bash tests/release/version_consistency.sh --repo <repo> --post-release --repository xinglun/ai-cockpit --tag v0.2.90`
- 使用公开 Release 执行 `tests/release/adopter_acceptance.sh` 与 `tests/release/adopter_upgrade_acceptance.sh`
