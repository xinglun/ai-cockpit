---
author: AI Cockpit maintainers
title: "WI-541——v0.2.67 发布与公开产物验收"
description: "发布已审查的 Runtime，并验证下载的公开产物边界。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
workItemId: WI-541-release-v0-2-67
lastVerifiedBy: WI-541-release-v0-2-67
terminalArchive: .ai/work-items/archive/WI-541-release-v0-2-67.contract.json
terminalVerification: .ai/evidence/WI-541-release-v0-2-67.verification.json
terminalFinalization: .ai/decisions/WI-541-release-v0-2-67.finalize.json
terminalDecision: .ai/decisions/WI-541-release-v0-2-67.close.json
---

[English](WI-541-release-v0-2-67.md) · [日本語](WI-541-release-v0-2-67.ja.md)

## 目标

从已审查且同步的默认分支发布 v0.2.67，然后验证不可变公开产物可以在不回退到源码或工作区二进制的情况下安装并完成新的 adopter 验收。

## 范围与边界

- Workspace 版本和 lockfile、当前发布/版本架构页面、分发说明及三语 parity 登记。
- Hosted 发布检查，以及发布后的产物、校验和、SBOM、provenance、隔离、清理和已安装 Runtime 证据。
- 对象工程 `/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator` 保持只读；不修改对象 `.ai/`、PR 身份或全局 Agent/MCP 设置。

## 验收

- Cargo metadata 和 lockfile 标识 v0.2.67，历史发布事实保持不变。
- 创建标注的 v0.2.67 tag 前，审查后的 PR 和 Hosted checks 全部通过，且 tag 来自同步的 `main`。
- 公开 archive、校验和、SBOM、provenance 和 release manifest 对同一 tag commit 与字节达成一致。
- 下载的公开产物 adopter 与 N-1 验收在隔离根目录中通过，保留 `first-adopter-smoke=not_ready`，并证明临时根目录已清理。
- 安装的公开 v0.2.67 通过仓库绑定健康检查，并记录 human Outcome、archive、finalization、close 和精确分支清理。

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

发布与发布后验收是两个独立事实。发布后验收失败时记录 `releasePublished: true` 和 `adopterAcceptance: failed`，不得改写已经发布的 Release 事实。
