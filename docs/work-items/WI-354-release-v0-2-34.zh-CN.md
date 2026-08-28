---
author: AI Cockpit maintainers
title: "WI-354——v0.2.34 发布准备"
workItemId: WI-354-release-v0-2-34
description: "在生命周期清理门修复后准备 v0.2.34，并把公开制品验收交给发布后 successor。"
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-354-release-v0-2-34
terminalArchive: .ai/work-items/archive/WI-354-release-v0-2-34.contract.json
terminalVerification: .ai/evidence/WI-354-release-v0-2-34.verification.json
terminalFinalization: .ai/decisions/WI-354-release-v0-2-34.finalize.d571d77922d342af1d3f2e43819cf77b73ba1affb0c0b15d1ac6d96d61d46577.json
terminalDecision: .ai/decisions/WI-354-release-v0-2-34.close.json
capabilityClaims: [release_distribution]
---

# WI-354——v0.2.34 发布准备

[English](WI-354-release-v0-2-34.md) · [日本語](WI-354-release-v0-2-34.ja.md)

## 意图与边界

在 WI-352 完成生命周期清理门之后，从已审阅的默认分支准备 v0.2.34，
统一 workspace 版本和当前安装文档，并且只通过已审阅的 hosted release
流程发布。

本 Work Item 不改写历史 Release truth，也不在标签发布前宣称公开制品已
安装。发布后的 successor 必须下载不可变公开 archive、安装它，并验收当前
仓库与 adopter 边界。

## 范围

- 将 `Cargo.toml`、`Cargo.lock` 和三种语言的当前发布、分发架构、版本文档
  统一到 v0.2.34。
- 保留 v0.2.30、v0.2.32 的失败发布历史。
- 标签前运行文档、版本一致性、治理完整性、发布策略和完整 workspace 门。
- 通过 `.github/workflows/release.yml` 发布精确审阅过的标签，并绑定
  manifest、校验和、SBOM、provenance、archive smoke 与 staged adopter 证据。
- 把公开 binary 安装和当前仓库验收交给发布后的 successor。

## 不在范围内

WI-351/WI-353 recovery、Runtime 新治理行为、外部 Homebrew tap、全局
Agent/MCP 配置、第二技术栈 adopter，以及发布后 receipt 内容都不在本边界。

## 验收与验证

- 所有 workspace 包和 `Cargo.lock` 为 0.2.34，标签为 `v0.2.34`。
- 三种语言的当前发布、分发架构和版本文档指向 v0.2.34，同时保留历史失败事实。
- 发布前源码路线和 hosted release 门全部通过。
- 标签 workflow 将 manifest、`SHA256SUMS`、五个 target archive、SBOM、
  provenance 与 staged adopter 门绑定到同一提交。
- 本 Work Item 不记录发布后安装成功；该结论只属于不可变公开制品 successor。

声明的检查包括 `cargo test --locked --workspace`、文档和发布一致性脚本、
发布策略测试，以及 hosted quality、Windows、behavioral-oracle、archive、
SBOM 和 staged-adopter jobs。
