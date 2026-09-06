---
title: "WI-607——v0.2.81 发布与对象验收"
description: "发布下一版 Rust Runtime，并验证不可变制品与对象工程验收边界。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-607-release-v0-2-81
lastVerifiedBy: WI-607-release-v0-2-81
terminalArchive: .ai/work-items/archive/WI-607-release-v0-2-81.contract.json
terminalVerification: .ai/evidence/WI-607-release-v0-2-81.verification.json
terminalFinalization: .ai/decisions/WI-607-release-v0-2-81.finalize.json
terminalDecision: .ai/decisions/WI-607-release-v0-2-81.close.json
---

[English](WI-607-release-v0-2-81.md) · [日本語](WI-607-release-v0-2-81.ja.md)

# WI-607——v0.2.81 发布与对象验收

## 目标

将已审查的 Rust Runtime 发布为 `v0.2.81`，再只使用不可变的公开 Release
制品验证安装与升级。本版本包含 GitHub Release API 认证修复，不是源工程迁移。

## 边界

本 Work Item 覆盖 package 版本、发布/分发、三语发布与 parity 记录，以及
staged/public 发布验收证据。不会复制参考项目脚手架、Python/Make 实现或 JSON
wire format。对象工程和 adopter 工程保持只读，全局 Agent/MCP 配置及历史治理字节
不在范围内。

## 验收

1. Workspace package 与 `Cargo.lock` 一致为 `0.2.81`。
2. Release CI 发布 annotated tag、目标制品、SBOM/provenance、Formula、校验和及
   Runtime identity，并保持摘要互相绑定。
3. 发布后 adopter 与 N-1 验收只能使用不可变公开制品，证明隔离和临时运行目录清理。
4. 中、英、日三语发布与 parity 记录一致说明 `v0.2.81` 基线及 `v0.2.80` N-1 边界。
5. 终态 Outcome 必须面向人可见，并记录状态、未知项、证据、决定和下一步。

## 验证

执行 `cargo test --locked --workspace`、文档与 metadata 检查、发布策略/版本一致性
检查，以及只使用不可变制品的 staged/public adopter 验收脚本。在发布证据中记录下载
Runtime 的版本和 SHA-256。终态 Outcome 必须保持为独立的面向人可见交接。
