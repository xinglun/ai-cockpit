---
author: AI Cockpit maintainers
title: "WI-477——发布 v0.2.56 与公开 adopter 验收"
description: "发布经审核的 Runtime 补丁，并在不修改对象工程的前提下验收不可变公开二进制。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implementation_active
authority: canonical
lastVerifiedBy: WI-477-release-v0-2-56
workItemId: WI-477-release-v0-2-56
---

# WI-477——发布 v0.2.56 与公开 adopter 验收

## 意图

发布下一版经审核的 Runtime，证明其不可变公开二进制可以治理隔离的
adopter，在当前仓库安装后再回到本地参考源逐文件比对。本 Work Item 不修改
参考源或任何对象工程。

## 范围

- 将 workspace package identity 与当前三语发布/版本文档对齐到 `v0.2.56`，保留历史事实。
- 在归档前将本 Work Item 注册到三语 parity ledger。
- 通过审核 PR、发布 annotated tag，并保留 manifest、checksum、SBOM、provenance 和 artifact identity evidence。
- 只使用下载的公开 Release artifact，在隔离目录执行 public adopter 与 N-1 验收，包含 evidence reuse 与临时目录清理。
- 在本仓库安装或升级公开二进制，并验证 repository、Runtime、Agent 和 readiness 状态。

## 不在范围内

本地参考源、`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`、其他 adopter、全局 Agent/MCP 配置、Homebrew tap 修改、源码 fallback，以及无关的 Runtime 架构变更。

## 验收标准

1. workspace 版本、lockfile 与所需三语发布文档一致标识 `v0.2.56`，不重写历史。
2. PR 合并前通过托管检查，annotated `v0.2.56` tag 精确指向同步后的审核主线提交。
3. 公开 Release 提供预期 archives、校验和、SBOM/provenance 与 identity-bound manifest。
4. adopter 与 N-1 验收只使用不可变公开 artifact，保留 `first-adopter-smoke=not_ready`，绑定 identity/digest，证明隔离并证明成功/失败路径清理。
5. 公开二进制安装到本仓库后，`inspect`/`status`/`doctor`/`agent doctor` 确认健康且 `ready_on_base`。
6. 本 Work Item 完成可见 Human Outcome、archive/finalization/close、文档 promotion 与精确分支/worktree 清理。

## 验证

```text
cargo test --locked --workspace
```

发布与公开验收属于发布后 evidence。失败时记录失败，不重写既有 Release truth。

## 边界

Runtime 升级只替换共享 executable，Repository Protocol 状态仍由各仓库持有；发布不会隐式 attach 或修改仓库。
