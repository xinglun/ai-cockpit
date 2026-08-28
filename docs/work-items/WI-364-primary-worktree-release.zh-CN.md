---
author: AI Cockpit maintainers
title: "WI-364——主工作树发布恢复"
workItemId: WI-364-primary-worktree-release
description: "防止普通 Work Item 绑定仓库主工作树，并从专用检出重新交付 v0.2.37。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-364-primary-worktree-release
terminalArchive: .ai/work-items/archive/WI-364-primary-worktree-release.contract.json
terminalVerification: .ai/evidence/WI-364-primary-worktree-release.verification.json
terminalFinalization: .ai/decisions/WI-364-primary-worktree-release.finalize.json
terminalDecision: .ai/decisions/WI-364-primary-worktree-release.close.json
capabilityClaims: [lifecycle_entry, release_distribution]
---

# WI-364——主工作树发布恢复

[English](WI-364-primary-worktree-release.md) · [日本語](WI-364-primary-worktree-release.ja.md)

## 目标

根治 WI-363 暴露的发布交付边界：普通 Work Item 不得绑定仓库 primary
worktree 或 default branch。使用专用 Work Item worktree 重新交付 v0.2.37，并保留前置
Work Item 的不可变 recovery evidence。

## 范围与边界

- 在写入 Contract 前，若当前 checkout 是 Git primary worktree 或已知 default branch，拒绝普通 `start` 与 `work-item new`。
- 如果 linked worktree 缺少或无法唯一确定远端 default base，也拒绝；没有 linked worktree 的本地 calibration repository 继续保持 `status: unknown`。
- 为 primary、default、dedicated 和 metadata 含糊场景增加 CLI 回归测试。
- 在三语 canonical workflow、命令和 parity 文档中说明拓扑要求及 WI-363 recovery 边界。
- 从本专用 worktree 完成不可变 v0.2.37 artifact、adopter、N-1、finalization、close 和精确清理验收。

修改 WI-363 archive/evidence/decision bytes、发布 artifact 语义、全局 Agent/MCP 配置或无关 Runtime 行为不在本 Work Item 范围内。

## 验收

1. 普通 `start` 与 `work-item new` 在 primary worktree 和 default branch 上 fail closed，专用 linked worktree 可以通过。
2. 缺少或含糊的远端 default metadata 不能授权 linked worktree，不写入 false-green Contract。
3. 拓扑回归覆盖所有场景，拒绝入口不留下 Work Item 文件。
4. 三语 workflow、命令和 parity 文档说明规则并链接前置 recovery 边界。
5. 公开 v0.2.37 artifact 仅通过校验和下载，不使用源码或 workspace fallback；adopter 与 N-1 receipt 证明隔离和清理。
6. reviewed merge、finalization、close 和精确 branch/worktree 清理完成后，已同步的 `main` 达到 ready on base。

## 验证边界

已安装 Runtime 记录 Contract amendment、preflight、checkpoint、verification、finish、archive、finalization 和 close evidence。Hosted CI 与公开 artifact 验收对发布声明负责。WI-363 的 archive 与 recovery bytes 保持历史不可变，绝不重写。
