---
author: AI Cockpit maintainers
title: "发布分发架构"
description: "经过验证的 Rust 构建如何成为可安装的 AI Cockpit runtime，以及安装为何不等于 attach。"
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_distribution
keywords: [ai-cockpit, release, homebrew, distribution, provenance]
---

# 发布分发架构

## 目的

本页回答：**发布过程中验证什么、用户如何安装 runtime，以及 Homebrew 的边界在哪里？**

## 读者

安装 AI Cockpit 或审查 release pipeline 前阅读本页。内容先面向一般采用者，同时标出维护者
需要关注的身份绑定。

## 读完之后

你会知道哪个 artifact 是事实来源、五个 target 如何绑定、tap handoff 可以做什么，以及安装
为什么不会静默 attach repository。

## Release 与安装流程

```text
source commit + immutable tag
            │
            ▼
source quality + policy gate
            │
            ▼
五个 target 构建（archive + SBOM）
            │
            ▼
canonical manifest + SHA256SUMS
            │
            ▼
artifact smoke test + provenance attestation
            │
            ▼
        GitHub Release
       ┌────┼───────────────┬─────────────────┐
       ▼    ▼               ▼                 ▼
 Homebrew  verified       Cargo Git        manual archive
 Formula   archive        fallback          install
       │    │               │                 │
       └────┴───────────────┴─────────────────┘
                         ▼
                   `ai-cockpit`
                         │ 显式 attach
                         ▼
       目标 repository + `.ai/cockpit.toml` + `.ai/project.json`

homebrew-handoff.json ──► 外部 tap review（仅在存在维护中的 tap 时）
                          （不属于本仓库 Runtime authority）
```

Release manifest 绑定 version、tag、commit、target、runner image、archive、SBOM、字节数、digest
和 provenance subject。`SHA256SUMS` 只覆盖 manifest 列出的 archive 与 SBOM。provider Release 或
单独上传 artifact 都不是安装证据。

## 采用者需要做什么

1. 从已发布的 Homebrew Formula 安装，或从 immutable Release 下载匹配的 archive。
2. 验证 version、SHA-256 digest 和 provider attestation。
3. 只有在审查目标 repository 及其 Work Item 后，才运行
   `ai-cockpit attach --repo /path/to/repository`。Attach 是显式步骤，可能创建或更新 `.ai/`。
4. 针对已 attach 的 repository 启动 CLI 或 MCP adapter。

未 attach 的 release-build checkout 可以没有 `.ai`；当前 self-governed checkout 有意拥有
repository-local `.ai/`。`cockpit.toml` 仍是 `.ai/` 下的 TOML；分发工作不会将其迁移为 JSON。

## 信任边界

- `cockpit-release` 与 release workflow 负责本地 release contract、确定性 manifest、Formula 投影、
  hosted checks 和已发布 Release identity。
- 当前不可变公开基线是 `v0.1.1`；WI-40 记录其 public adopter acceptance。外部 Homebrew tap 是
  独立 provider surface，不由本仓库自动保证。
- Tap 接收经过审查的 Formula 投影，不会重新构建 binary。
- Homebrew 是交付路径，不是治理权威。Repository 事实和人类决策仍来自已 attach 的 repository 与 Work Item。

## 停止条件

当 tag、workspace version、binary version、commit、manifest、digest、SBOM、provenance subject 或
provider Release identity 不一致时停止。当 handoff 过期、指向另一个 commit、要求不同 destination，
或试图直接修改 default branch 时停止。当有人把安装结果描述为 repository 已 attach 的证明时也停止。

## 下一步

1. [发布与分发](../release/distribution.zh-CN.md) — 面向采用者的命令。
2. [架构](../architecture.zh-CN.md) — runtime 和证据所有权。
3. [参考源对齐](../reference/reference-parity.zh-CN.md)——与参考 template 的明确差异。

## 技术深度

Rust `cockpit-release` package 执行严格的 manifest、archive、Formula 和 handoff 验证。GitHub
Actions 构建五个保留 target，分离 source、verification、attestation、publication 和 handoff 权限，
并把外部 tap mutation 保持在默认 repository token 之外。
