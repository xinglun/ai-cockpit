---
title: "WI-610——v0.2.82 发布与对象验收"
description: "发布下一版 Rust Runtime，并用不可变发布产物验证 adopter 边界。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-610-release-v0-2-82
lastVerifiedBy: WI-610-release-v0-2-82
---

[English](WI-610-release-v0-2-82.md) · [日本語](WI-610-release-v0-2-82.ja.md)

# WI-610——v0.2.82 发布与对象验收

## 目标

将已审查的 Rust Runtime 发布为 `v0.2.82`，再只使用不可变的公开 Release
产物验证安装与升级。共享 Runtime/仓库隔离模型以及上一 Work Item 已验证的
抗竞态 adopter 清理能力保持不变。

## 边界

本 Work Item 覆盖包版本元数据、发布分发、三语发布与 parity 记录，以及 staged/
public 发布验收证据。不修改对象仓库，不复制参考源脚手架、Python/Make 实现，
也不修改全局 Agent/MCP 配置。

## 验收

1. Workspace 包版本与 `Cargo.lock` 均解析为 `0.2.82`。
2. Release CI 发布带注释的 `v0.2.82` 标签、目标归档、SBOM/来源证明、Formula、
   校验和，并保持 Runtime identity 一致。
3. 发布后 adopter 与 N-1 验收只使用不可变公开产物，证明仓库隔离与临时运行目录清理。
4. 英文、中文、日文发布/parity 记录一致标明 `v0.2.82` 与 `v0.2.81` N-1 边界。
5. 终态 Outcome 必须面向人显示，并记录状态、未知项、证据、人工决定与下一步。

## 验证

运行 `cargo test --locked --workspace`、文档与元数据检查、发布策略/版本一致性检查，以及
不可变 staged/public adopter 与 N-1 验收脚本。在发布证据中记录下载的 Runtime 版本与
SHA-256。终态 Outcome 始终是独立、面向人的交接。
