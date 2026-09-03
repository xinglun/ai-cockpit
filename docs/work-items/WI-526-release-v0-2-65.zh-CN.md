---
author: AI Cockpit maintainers
title: "WI-526——v0.2.65 发布与对象工程恢复验收"
description: "发布 direct-merge recovery context 修复，并在不修改对象仓库的前提下验证不可变公开 artifact。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-526-release-v0-2-65
lastVerifiedBy: WI-526-release-v0-2-65
terminalArchive: .ai/work-items/archive/WI-526-release-v0-2-65.contract.json
terminalVerification: .ai/evidence/WI-526-release-v0-2-65.verification.json
terminalFinalization: .ai/decisions/WI-526-release-v0-2-65.finalize.json
terminalDecision: .ai/decisions/WI-526-release-v0-2-65.close.json
---

[English](WI-526-release-v0-2-65.md) · [日本語](WI-526-release-v0-2-65.ja.md)

## 目标

从经过评审且已同步的默认分支发布 v0.2.65。本版本包含 direct-merge recovery context
兼容性修复和终态文档投影修正。对象工程保持只读，由对象工程团队在发布后自行验收。

## 范围

- 三语 workspace package/lockfile 版本和当前发布文档。
- 发布流程与不可变公开 artifact 证据。
- 本 Work Item 的三语文档和 parity 登记。
- 根据不可变 archive、verification、finalization 和 close 证据晋级已关闭的 WI-527 终态文档。
- 下载 binary 的安装、健康检查、隔离和 adopter 验收。

对象工程 `/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`
明确为只读；不得手改其 `.ai/` 记录或伪造 PR identity。全局 Agent/MCP 配置和源码构建
fallback 不在范围内。

## 验收

- package 与 lockfile 标识 v0.2.65，且保留历史发布事实。
- 在同步 `main` 创建带注释的 v0.2.65 tag 前，托管检查全部通过。
- 公开 archive、SHA256SUMS、SBOM、provenance 和 manifest 绑定同一 tag 与 bytes。
- 仅使用下载 artifact 的 adopter/N-1 验收证明隔离和临时目录清理。
- 安装发布 binary 后，仓库健康和文档检查全部通过。
- 可见 Outcome、archive、finalization、close 以及精确分支/worktree 清理均有记录。

## 验证

```text
cargo metadata --locked --format-version 1
cargo test --locked --workspace
tests/release/version_consistency.sh --repo <repo>
tests/release/action_runtime_policy.sh .github/workflows/ci.yml .github/workflows/release.yml
tests/release/adopter_acceptance_test.sh
tests/release/adopter_upgrade_acceptance_test.sh
tests/docs/promote_closed_work_item.py --repo <repo> --check-all
tests/docs/documentation_acceptance.sh
tests/docs/parity_status_check.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo> --report <report>
```

发布与发布后验收是两个独立事实。发布后失败记录
`releasePublished: true` 和 `adopterAcceptance: failed`，不会改写发布事实。
