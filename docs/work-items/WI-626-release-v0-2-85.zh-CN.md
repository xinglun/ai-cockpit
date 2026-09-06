---
title: "WI-626——v0.2.85 发布与对象验收"
description: "在 v0.2.84 失败边界之后发布新版本，并保留可审计的替换 lineage。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-626-release-v0-2-85
lastVerifiedBy: WI-626-release-v0-2-85
terminalArchive: .ai/work-items/archive/WI-626-release-v0-2-85.contract.json
terminalVerification: .ai/evidence/WI-626-release-v0-2-85.verification.json
terminalFinalization: .ai/decisions/WI-626-release-v0-2-85.finalize.json
terminalDecision: .ai/decisions/WI-626-release-v0-2-85.close.json
---

[English](WI-626-release-v0-2-85.md) · [日本語](WI-626-release-v0-2-85.ja.md)

# WI-626——v0.2.85 发布与对象验收

## 目标

在不可变的失败 `v0.2.84` 发布边界之后发布已审查 Runtime `v0.2.85`，再只使用不可变
Release 产物验证公开安装与升级。

## 边界

本 Work Item 覆盖版本元数据、发布/分发文档、失败发布恢复投影及发布验收接口。不修改对象工程，
不复制参考源脚手架/Python/Make 实现，也不修改全局 Agent/MCP 配置。

## 验收

1. Workspace 包与 `Cargo.lock` 均解析为 `0.2.85`。
2. Release CI 发布带注释的 `v0.2.85` 标签、归档、校验和、SBOM/来源证明、Formula 及匹配的 Runtime identity。
3. 发布后 adopter 与 N-1 harness 只使用不可变的 `v0.2.85` 和 `v0.2.83` 产物，并证明隔离与运行根目录清理。
4. 英文、中文、日文发布/版本/parity 记录一致标明 `v0.2.85` 与 `v0.2.83` 边界。
5. 终态 Outcome 面向人显示，并记录状态、未知项、证据、人工决定和下一步。

## 验证

合并前运行 workspace 测试及发布/文档策略检查。发布后运行不可变 adopter 与 N-1 验收 harness，记录下载的
Runtime identity；源码或 workspace binary 不能替代 Release 产物。
