---
author: AI Cockpit maintainers
title: "WI-792——v0.2.90 发布"
description: "在信任诊断与治理文档集成验收后发布下一补丁版本。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-release
workItemId: WI-792-release-v0-2-90
lastVerifiedBy: WI-792-release-v0-2-90
---

[English](WI-792-release-v0-2-90.md) · [日本語](WI-792-release-v0-2-90.ja.md)

# WI-792——v0.2.90 发布

## 意图与边界

在 A–E 信任评审完成集成后发布策略批准的下一补丁版本。发布必须来自已审查
的 PR 和已同步的默认分支，使用新的 annotated tag，并且只有下载的公开产物
可以作为采用验收依据。本 Work Item 不改变 Rust 或 Runtime 行为，不重写历史
证据，不复用 tag，也不削弱发布门槛。

## 发布说明分类

- 交流修正：Outcome 原因投影区分实际治理缺口；finalization 恢复动作保留
  具体条件；CLI/MCP、摘要/完整报告和三语协作检查验证人类语义。
- 验证覆盖：A–E 反例、无历史交接、历史兼容、受控资源清理和确定性的并发变化
  路径绑定到可执行检查。
- 架构一致性：Outcome 输入在一个有界且重新校验的 ObservationContext 边界内
  组装，纯渲染继续保持无 I/O。
- 性能诊断：主路径提供阶段耗时和受控的字节/哈希/Git/进程计数，并覆盖
  macOS/Linux 文件系统路径。这些是诊断测量，不宣称未经验证的提速；此前拒绝
  的优化仍保持拒绝结论。

## 验收

- Cargo 元数据、lockfile、当前三语发布/版本投影及 reference metadata 对同一
  精确 package identity 一致。
- 审查后的 PR 在合并前通过 hosted quality、route、behavioral、Windows 和任务
  完成检查。
- annotated tag 与公开 Release manifest 绑定已合并的 main commit；发布产物、
  checksum、SBOM 和 attestation 通过各自门槛。
- 公开安装与 N-1 升级使用隔离根目录中的下载产物并证明清理；workspace 构建
  不能替代发布产物。
- Work Item 完成 archive、finalization、close，精确分支/工作树清理及 main 同步
  均得到验证。

## 验证

- `bash tests/release/version_consistency.sh --repo <repo>`
- `python3 tests/docs/reference_comparison_metadata_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `cargo test --locked --workspace`
- `bash tests/release/version_consistency.sh --repo <repo> --post-release --repository xinglun/ai-cockpit --tag v0.2.90`
- 使用公开 Release 执行 `tests/release/adopter_acceptance.sh` 与 `tests/release/adopter_upgrade_acceptance.sh`
