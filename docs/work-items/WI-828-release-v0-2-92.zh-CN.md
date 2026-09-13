---
author: AI Cockpit maintainers
title: "WI-828——受治理的 v0.2.92 发布"
description: "从最新评审过的 main 准备并发布 ai-cockpit v0.2.92。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-828-release-v0-2-92
lastVerifiedBy: WI-828-release-v0-2-92
---

[English](WI-828-release-v0-2-92.md) · [日本語](WI-828-release-v0-2-92.ja.md)

# WI-828——受治理的 v0.2.92 发布

## 意图与边界

WI-828 将 workspace 和当前发布投影对齐到 v0.2.92，并将经过评审的 main 修订发布为不可变
Release。候选和公开验收必须在隔离根目录中消费绑定身份的制品。

历史 Work Item 迁移、对 v0.2.91 的修改、无关性能工作和用户全局配置均不在本 Work Item 范围内。

## 验收

- Cargo metadata、lockfile、当前发布文档和 reference comparison metadata 一致标识 v0.2.92。
- hosted source verification、构建、候选安装和 N-1 升级通过，并保留绑定身份的 receipt。
- 公开 v0.2.92 制品通过全新安装、N-1 升级、版本一致性和清理检查。
- 失败恢复复用仍有效的阶段，不重新构建或发布；finalization、archive、close 和文档 promotion 均得到确认。

## 验证

- `bash tests/release/version_consistency.sh --repo <repo>`
- `python3 tests/docs/reference_comparison_metadata_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh <repo>`
- `cargo fmt --all --check`
- `cargo test --locked --workspace --all-targets`
- hosted release preflight、候选验收、公开制品验收、N-1 升级和 close-only recovery。

## 证据规则

发布标签、provider 制品、manifest 和公开验收 receipt 都是不可变外部事实。Runtime 的 Contract、
verification、archive、finalization 和 close 记录由安装版 Runtime 生成，不能由本地构建或移动分支替代。
