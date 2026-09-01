---
author: AI Cockpit maintainers
title: "WI-474——发布 v0.2.55 与公开 adopter 验收"
description: "从已审查主线发布 Runtime 补丁，并在不修改 adopter 工程的前提下验证不可变公开二进制。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: authorized
lastVerifiedBy: WI-474-release-v0-2-55
workItemId: WI-474-release-v0-2-55
---

# WI-474——发布 v0.2.55 与公开 adopter 验收

## 意图

发布下一次经过审查的 Runtime 补丁，然后证明不可变公开二进制能够安装，
并治理隔离的 adopter。此次发布继续主线参考源比对，不修改参考源或 adopter
工程。

## 范围

- 将 workspace package identity 以及当前三语发布/版本指引推进到
  `v0.2.55`，保留历史发布事实。
- 在 archive 之前把本 Work Item 登记到三语 reference-parity ledger。
- 合并经过审查的 PR，发布 annotated tag，并保留 manifest、checksum、SBOM、
  provenance 和 artifact identity evidence。
- 仅使用隔离运行根目录中的公开 Release 下载物执行 adopter 与 N-1 验收，
  包含 evidence reuse 与临时根目录清理证明。
- 在当前仓库安装或升级公开二进制，验证 repository、Runtime、Agent 与就绪状态。

## 不在范围内

本地参考源、`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`、
其他 adopter 工程、全局 Agent/MCP 配置、Homebrew tap 修改、源码 fallback，
以及无关的参考源比对或 Runtime 架构变更均不在范围内。

## 验收标准

1. Workspace package、lockfile 和必需的三语发布/版本文档准确标识
   `v0.2.55`，不改写历史。
2. PR 在合并前通过托管检查；annotated `v0.2.55` tag 精确指向同步后的审查主线提交。
3. 公开 Release 提供预期 archives、checksums、SBOM、provenance metadata 和
   绑定 identity 的 release manifest。
4. 公开 adopter 与 N-1 验收只使用不可变公开 artifact，保留
   `first-adopter-smoke=not_ready`，绑定 repository/Runtime identity，证明禁止写入根目录隔离，
   并证明成功与失败路径都清理临时运行根目录。
5. 在当前仓库安装公开二进制后，`inspect`/`status`/`doctor`/`agent doctor`
   证明 attach、健康、隔离和 `ready_on_base` 状态。
6. 本 Work Item 产出可见 human Outcome，完成 archive/finalization/close、文档晋级以及精确的分支/工作树清理。

## 验证

源码验证命令：

```text
cargo test --locked --workspace
```

发布与公开 adopter 验收属于发布后的 evidence；失败时不得改写 Release truth。

## 边界

Runtime 升级只替换共享 executable，Repository Protocol 状态仍由各仓库独立持有。
发布不会 attach 或修改 adopter 工程。
