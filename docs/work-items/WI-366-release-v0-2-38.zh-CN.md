---
author: AI Cockpit maintainers
title: "WI-366——N-1 身份根治后的 v0.2.38 发布准备"
workItemId: WI-366-release-v0-2-38
description: "在 v0.2.37 N-1 Git 身份根治后准备发布，并将不可变公开产物验收交给后续 Work Item。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-366-release-v0-2-38
terminalArchive: .ai/work-items/archive/WI-366-release-v0-2-38.contract.json
terminalVerification: .ai/evidence/WI-366-release-v0-2-38.verification.json
terminalFinalization: .ai/decisions/WI-366-release-v0-2-38.finalize.json
terminalDecision: .ai/decisions/WI-366-release-v0-2-38.close.json
capabilityClaims: [release_distribution, adopter_acceptance]
---

# WI-366——N-1 身份根治后的 v0.2.38 发布准备

[English](WI-366-release-v0-2-38.md) · [日本語](WI-366-release-v0-2-38.ja.md)

## 意图

在 v0.2.37 N-1 升级验收根因修复并合并后，从已审核且同步的 `main`
准备 v0.2.38。公开产物安装和 adopter 验收属于发布后的后续 Work Item，
本 WI 不提前宣称结果。

## 范围与边界

- 将 workspace package metadata、lockfile 和当前三语发布/版本文档统一到
  v0.2.38。
- 在打 tag 前执行已审核的 release policy、文档、parity 和 staged adopter
  回归检查。
- 保留明确的后续 Work Item 交接，由它在发布后下载不可变公开 artifact 并执行
  adopter/N-1 验收。
- 保留未公开的 v0.2.37 候选失败为不可变历史，不移动或重新标记其 tag。

Runtime 行为修改、历史 evidence 重写、全局 Agent/MCP 配置、源码构建回退和
第二技术栈 adopter 不属于本 WI。

## 验收

1. Cargo metadata 与 lockfile 一致报告 0.2.38。
2. 不可变 tag 前 hosted CI 与发布策略 gate 全部通过。
3. v0.2.37 N-1 Git 身份失败由 repository-local identity 回归覆盖，不需要全局 Git 配置。
4. 发布后的公开 artifact、已安装 binary、adopter 隔离和 N-1 验收明确交接给后续
   Work Item，发布前不宣称这些结果。
5. 合并、finalization、close 及精确分支/工作树清理后，仓库处于同步默认分支并可继续工作。
6. 已关闭的 WI-365 三语 Work Item 投影与终态 evidence 和 parity 行一致报告为已实现。

## 验证边界

Runtime lifecycle 记录 Contract、checkpoint、verification、archive、finalization
和 close evidence。后续 Work Item 必须绑定不可变发布 tag，并记录 hosted workflow、
安装和 adopter receipt 后，才能声明公开 artifact。失败的 v0.2.37 候选保持不变，
不作为安装来源。
