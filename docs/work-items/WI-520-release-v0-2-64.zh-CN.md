---
author: AI Cockpit maintainers
title: "WI-520——v0.2.64 发布与对象采用者兼容性验收"
description: "发布已合并的历史 finalization 兼容修复，并在不修改对象工程的前提下验证公开制品。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-520-release-v0-2-64
lastVerifiedBy: WI-520-release-v0-2-64
---

[English](WI-520-release-v0-2-64.md) · [日本語](WI-520-release-v0-2-64.ja.md)

## 目标

从已审核并同步的默认分支发布 v0.2.64。该版本包含 WI-518 的历史
direct-merge 应用路径及准确诊断。发布后，采用者验收只能使用下载的公开
制品；对象工程保持只读，由对象工程团队自行执行验收。

## 范围

- 三语当前发布文档以及 workspace package/lockfile 版本。
- Release workflow、公开 adopter 验收、N-1 验收及其清理/隔离 wrapper。
- 本 Work Item 的三语文档和 parity 登记。
- 不可变 tag、公开 Release 制品、校验和、SBOM、provenance，以及下载制品
  的安装/健康检查。

对象工程 `/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`
明确为只读。不得手改其 `.ai/` 记录或伪造 PR identity。全局 Agent/MCP 配置、
源码构建 fallback 和无关 Runtime 行为不在范围内。

## 验收

- 所有 workspace package 和 Cargo.lock 标识 v0.2.64，且不改写历史发布事实。
- 同步的 `main` 在创建 annotated v0.2.64 tag 前通过全部托管检查；失败发布的
  tag 永不复用。
- 公开 Release manifest、五个 archive、五个 SBOM、SHA256SUMS 和 provenance
  绑定同一 tag、制品、target 与 digest。
- 公开 adopter 与 N-1 验收只使用不可变下载制品，证明 HOME/XDG 隔离，分类
  TMPDIR/CARGO_HOME 写入，证明清理完成，并保留 `first-adopter-smoke=not_ready`。
- 在本仓库安装公开 binary，并通过 `inspect`、`status`、`doctor`、Agent doctor
  与文档晋级检查。
- 在宣布发布完成前，Outcome、archive、finalization、close 以及精确分支/工作树
  清理都必须可见且有记录。

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
python3 tests/ci/governance_integrity_gate.py --repo <repo>
```

发布与发布后验收是两个独立事实。若公开验收失败，记录
`releasePublished: true` 和 `adopterAcceptance: failed`，不能回写 Release truth。
不可变 Release 建立后，再提供最终 adopter receipt 和对象工程团队的验收步骤。
