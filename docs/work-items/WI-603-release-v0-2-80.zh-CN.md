---
title: "WI-603——v0.2.80 发布与对象验收"
description: "发布下一版 Rust Runtime，并验证不可变制品与对象工程验收边界。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-603-release-v0-2-80
lastVerifiedBy: WI-603-release-v0-2-80
---

[English](WI-603-release-v0-2-80.md) · [日本語](WI-603-release-v0-2-80.ja.md)

# WI-603——v0.2.80 发布与对象验收

## 目标

将已审查的 Rust Runtime 发布为 `v0.2.80`，再只使用不可变的公开 Release
制品验证安装与升级。这是参考源逐批比对及文档收敛后的发布边界，不是
源工程脚手架迁移。

## 边界

本 Work Item 只覆盖 package 版本、发布/分发、架构与版本文档、参考比对
登记，以及 staged/public 发布验收证据。不会复制参考项目的脚手架、
Python/Make 实现或 JSON wire format。对象工程和 adopter 工程保持只读，
全局 Agent/MCP 配置及历史治理字节不在范围内。

## 验收

1. Workspace package 与 `Cargo.lock` 一致为 `0.2.80`。
2. annotated tag、五个目标制品、SBOM/provenance、Formula、校验和、manifest
   与 Runtime identity 由 Release CI 互相绑定。
3. staged 与发布后 adopter 验收只能使用下载制品，证明 repository 隔离和
   临时运行目录清理；失败时不得改写已发布 Release truth。
4. 中、英、日三语发布、架构、版本和 parity 文档一致说明 `v0.2.80`，并
   明确 N-1 为 `v0.2.79`。
5. Hosted checks 通过后验证公开 Release；不修改参考 checkout 或对象工程。

## 验证

执行 `cargo test --locked --workspace`、文档与 metadata 检查、发布策略/版本
一致性检查，以及只使用不可变制品的 staged/public adopter 验收脚本。在发布
证据中记录下载 Runtime 的版本和 SHA-256。终态 Outcome 必须作为独立、面向
人的可见交接，包含状态、未知项、证据、决定和下一步。

