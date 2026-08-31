---
author: AI Cockpit maintainers
title: "WI-459 — v0.2.53 发布与公开二进制验收"
workItemId: WI-459-release-v0-2-53
description: "发布下一版经过审查的 Rust Runtime，并用 adopter 基线验收公开二进制。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-459-release-v0-2-53
---

# WI-459 — v0.2.53 发布与公开二进制验收

本 Work Item 将默认分支上已经审查的变更打包为 v0.2.53，发布后只使用
公开二进制执行 adopter 验收，然后回到参考源逐文件比对主线。

[English](WI-459-release-v0-2-53.md) · [日本語](WI-459-release-v0-2-53.ja.md)

## 范围

- 将 workspace package 与 lockfile identity 对齐到 v0.2.53。
- 同步三语安装、发布与版本文档，同时保留历史发布与失败记录。
- 继续以 annotated tag、manifest、checksum、SBOM、provenance 与 staged/public
  adopter gate 作为发布依据。
- 合并后下载公开 v0.2.53 二进制，保留 Runtime、repository、隔离、清理和 lifecycle receipt。

## 不在范围内

WI-445 持有的参考 inventory/parity ledger、本地参考源 checkout、对象工程、全局
Agent/MCP 配置、Homebrew tap 写入、源码构建 fallback 及无关 Runtime 行为。

## 验收

- workspace metadata、lockfile 和三语发布文档识别 v0.2.53，且不改写历史发布事实。
- 审查后的 PR 与 hosted release workflow 绑定 annotated tag、source commit、manifest、
  Cargo.lock digest、archive/SBOM checksum、provenance 和公开资产。
- 版本、workflow、文档和 workspace 质量门通过，不使用 workspace binary 或源码 fallback。
- 发布后的 adopter harness 只下载并校验 v0.2.53 公开 archive；receipt 证明 repository/runtime
  identity、隔离、evidence reuse、cleanup，并保留 `first-adopter-smoke` 的 `not_ready` contract。
- 发布与验收后默认分支同步，Work Item 关闭，repository 达到 `ready_on_base`。

## 验证

- `bash tests/release/version_consistency.sh --repo <repo>`
- `bash tests/release/version_consistency_test.sh`
- `bash tests/release/workflow_policy.sh .github/workflows/release.yml`
- strict repository gate manifest 与 documentation acceptance
- `cargo test --locked --workspace`
- 发布后只使用公开 v0.2.53 的 `tests/release/adopter_acceptance.sh`

## 发布边界

只有在 PR 审查合并且默认分支同步后才推送 tag。provider Release 由 workflow 在所有
source、artifact 与 staged-adopter gate 通过后创建。公开验收只在发布后执行；即使失败，
也记录失败而不改写已经发布的 Release。
