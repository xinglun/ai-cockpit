---
author: AI Cockpit maintainers
title: "WI-113 v0.2.8 公开发布与自身 adopter 验收"
description: "发布合并后的 Runtime，安装不可变 artifact，并验证它能够治理本仓库。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: release-adopter-acceptance
capabilityClaims:
  - public_release
  - self_adopter_acceptance
---

# WI-113：v0.2.8 公开发布与自身 adopter 验收

## 目标

从已由自身治理的 main 发布 v0.2.8，安装不可变的公开 binary，并证明它能在不回退
源码或 workspace binary 的情况下治理和开发本仓库。

## 范围

更新 workspace 版本与当前发布/版本文档，执行源码和供应链 gate，推送 v0.2.8 tag，
安装下载的 artifact，并记录发布后 adopter 与 N-1 验收证据。不重写历史 Work Item
记录，也不修改外部 Homebrew tap 状态。

## 验收

- 所有 workspace package 和 `Cargo.lock` 标识 0.2.8。
- 当前英文、中文、日文发布、运维、版本与 parity 页面标识 v0.2.8；v0.2.7 只作为
  明确的 N-1 输入或历史记录保留。
- Hosted release、artifact、manifest、checksum、provenance 和 Node24 policy gate 通过。
- 公开 archive 与 binary SHA-256 写入 Runtime identity evidence；adopter 验收不使用源码
  或 workspace binary。
- 安装的 v0.2.8 Runtime 报告 `changedPaths=[]`、`COMPATIBLE`、`doctor=ok`、
  `agent doctor=VERIFIED` 和 `runtimeCodeInRepository=false`。
- 公开 adopter、N-1 upgrade、隔离清理、evidence reuse 与 `first-adopter-smoke=not_ready`
  断言通过。
- 在发布 closure 前记录自身 Work Item lifecycle 与中英日可见 Outcome handoff。

## 验证

```text
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-targets --all-features -- --test-threads=1
bash tests/docs/documentation_acceptance.sh
bash tests/release/version_consistency.sh --repo .
bash tests/release/adopter_acceptance.sh --repository xinglun/ai-cockpit --tag v0.2.8
bash tests/release/adopter_upgrade_acceptance.sh --repository xinglun/ai-cockpit --from-tag v0.2.7 --to-tag v0.2.8
```

发布后 harness 是公开 artifact identity 与隔离的权威来源。失败时记录
`releasePublished: true`，不会回写 Release truth。

## Outcome

状态：**实现与发布准备完成；公开发布和下载 artifact 验收仍是 release-bound 步骤。**
