---
author: AI Cockpit maintainers
title: "WI-570——v0.2.74 发布与公开制品验收"
description: "发布并验证下一版不可变 AI Cockpit Runtime。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-570-release-v0-2-74
lastVerifiedBy: WI-570-release-v0-2-74
---

[English](WI-570-release-v0-2-74.md) · [日本語](WI-570-release-v0-2-74.ja.md)

# WI-570——v0.2.74 发布与公开制品验收

## 目标

从已审查并同步的默认分支发布 v0.2.74，形成不可变 Runtime 基线；随后证明下载的公开
binary 可以在不使用源码 checkout 或 workspace fallback 的情况下治理本仓库。

## 范围与边界

- 在三种语言中同步 Cargo 元数据、lockfile、发布与版本说明至 v0.2.74。
- 将发布绑定到默认分支上已经关闭的参考源比对与文档晋级记录。
- 生成并验证绑定 identity 的五目标 archive、manifest、checksums、SBOM、provenance、
  attestation 和 Runtime identity。
- 使用不可变下载制品，在隔离目录中执行 public adopter 与 N-1 验收，包含禁止写入目录和
  临时运行目录清理证明。

Runtime 实现、对象工程、全局 Agent/MCP 配置、复制参考源实现、重写失败 tag，以及无关历史记录
不在本 Work Item 范围内。

## 验收

1. Cargo 元数据、lockfile 和当前发布/版本页面标识 v0.2.74，同时保留 v0.2.73 作为前一个公开基线。
2. Release CI 为 v0.2.74 生成绑定 identity 的五目标制品和供应链证据集。
3. public adopter 与 N-1 验收只使用下载的 v0.2.74 制品，证明隔离和清理，并使用同一 binary
   验证本仓库。
4. 发布从同步且 ready 的默认分支开始，不改 Runtime 行为、对象工程、全局配置或无关历史证据。

## 验证边界

Contract 验收在其编写语言中保持权威；本地化页面只改变呈现。公开 Release 只有在不可变资产和
adopter receipt 验证后才算验收完成。对象工程验收是外部只读交接，本页面不代为宣称。

## 验证

- `tests/release/version_consistency.sh`
- `tests/release/workflow_policy.sh`
- `tests/release/action_runtime_policy.sh`
- `tests/release/source_archive_policy_test.sh`
- `tests/release/adopter_acceptance_test.sh`
- `tests/release/adopter_upgrade_acceptance_test.sh`
- `tests/docs/documentation_acceptance.sh`
- `tests/docs/parity_status_check.sh`
- `git diff --check`
