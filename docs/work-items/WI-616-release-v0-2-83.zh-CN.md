---
title: "WI-616——v0.2.83 发布与对象验收"
description: "发布 direct-merge 恢复修复，并用对象与 N-1 验收验证不可变发布产物。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-616-release-v0-2-83
lastVerifiedBy: WI-616-release-v0-2-83
---

[English](WI-616-release-v0-2-83.md) · [日本語](WI-616-release-v0-2-83.ja.md)

# WI-616——v0.2.83 发布与对象验收

## 目标

将已审查的 Rust Runtime 发布为 `v0.2.83`，再只使用不可变的公开 Release 产物验证安装与升级。本版本向对象工程提供 WI-614 的首次 `direct_merge_no_pr` 恢复修复。

## 边界

本 Work Item 覆盖包版本元数据、发布/分发、三语发布与 parity 记录，以及 staged/public 发布验收证据。不修改对象工程，不复制参考源脚手架/Python/Make 实现，也不修改全局 Agent/MCP 配置。

## 验收

1. Workspace 包版本与 `Cargo.lock` 均解析为 `0.2.83`。
2. Release CI 发布带注释的 `v0.2.83` 标签、目标归档、SBOM/来源证明、Formula、校验和及匹配的 Runtime identity。
3. Public adopter 与 N-1 验收只使用不可变的 `v0.2.83`/`v0.2.82` 产物，并证明仓库隔离与临时运行根目录清理。
4. 英文、中文、日文发布/版本/parity 记录一致标明 `v0.2.83` 及其 `v0.2.82` N-1 边界。
5. 终态 Outcome 面向人显示，并记录状态、未知项、证据、人工决定和下一步。

## 验证

运行 workspace 测试、文档与元数据检查、发布策略/版本一致性检查以及不可变 staged/public adopter 和 N-1 harness。将下载的 Runtime 版本和 SHA-256 记录到发布证据中。
