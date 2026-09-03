---
author: AI Cockpit maintainers
title: "WI-533——v0.2.66 发布与 direct-merge 恢复验收"
description: "发布包含 bundled historical direct-merge 兼容修复的 Runtime，并验证公开 artifact 边界。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
workItemId: WI-533-release-v0-2-66
lastVerifiedBy: WI-533-release-v0-2-66
---

[English](WI-533-release-v0-2-66.md) · [日本語](WI-533-release-v0-2-66.ja.md)

## 目标

从经过评审且已同步的默认分支发布 v0.2.66。本版本包含历史 direct-merge
恢复修复：将真实 merge parent 与归档 Contract base 分开绑定，使 bundled
merge 可以在不伪造 Pull Request、不改写历史的前提下记录。

## 范围与边界

- Workspace 版本、lockfile、发布 workflow、发布文档和三语 parity 登记。
- 不可变发布 archive、manifest、校验和、SBOM、provenance，以及下载 artifact
  的 adopter/N-1 验收。
- 发布后的 Runtime 安装和自身仓库健康检查。

对象工程 `/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`
是只读的，不得修改其 `.ai/` 记录，也不得伪造 PR identity。全局 Agent/MCP
配置和源码构建 fallback 不在范围内。

## 验收

- package 与 lockfile 标识 v0.2.66，同时保留历史发布事实。
- 从同步的 `main` 创建带注释的 v0.2.66 tag 前，托管检查全部通过。
- 公开 artifact 绑定同一 tag、commit、bytes、SHA256SUMS、SBOM 和 provenance subject。
- public/N-1 验收只使用不可变下载 artifact，证明仓库隔离和临时目录清理，并保留
  `first-adopter-smoke=not_ready`。
- 安装发布 binary 后 inspect/status/doctor/Agent doctor 和文档检查全部通过。
- 完成前记录可见 human Outcome、archive、finalization、close 以及精确分支和 worktree 清理。

## 验证

```text
cargo metadata --locked --format-version 1
cargo test --locked --workspace
tests/release/version_consistency.sh --repo <repo>
tests/release/action_runtime_policy.sh .github/workflows/ci.yml .github/workflows/release.yml
tests/release/adopter_acceptance_test.sh
tests/release/adopter_upgrade_acceptance_test.sh
tests/docs/documentation_acceptance.sh
tests/docs/parity_status_check.sh
tests/docs/promote_closed_work_item.py --repo <repo> --check-all
```

发布与发布后验收是独立事实。发布后失败记录
`releasePublished: true` 和 `adopterAcceptance: failed`，不会改写已发布的 Release。
