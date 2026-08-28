---
author: AI Cockpit maintainers
title: "WI-365——发布升级 Git 身份"
workItemId: WI-365-release-upgrade-git-identity
description: "使隔离 CI 环境中的公开版本到候选版本 N-1 验收提交具有确定性。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-365-release-upgrade-git-identity
terminalArchive: .ai/work-items/archive/WI-365-release-upgrade-git-identity.contract.json
terminalVerification: .ai/evidence/WI-365-release-upgrade-git-identity.verification.json
terminalFinalization: .ai/decisions/WI-365-release-upgrade-git-identity.finalize.json
terminalDecision: .ai/decisions/WI-365-release-upgrade-git-identity.close.json
capabilityClaims: [release_distribution, adopter_acceptance]
---

# WI-365——发布升级 Git 身份

[English](WI-365-release-upgrade-git-identity.md) · [日本語](WI-365-release-upgrade-git-identity.ja.md)

## 目标

根治 N-1 adopter 验收在干净 CI runner 上无法提交的问题：控制仓库由 clone
创建后没有 Git 身份，导致提交失败。

## 范围与边界

- 为脚本中所有会提交的验收仓库设置确定性的 repository-local Git 身份，
  只写入各自的 `.git/config`。
- 在隔离的 `HOME`/`XDG_CONFIG_HOME` 与 `GIT_CONFIG_GLOBAL=/dev/null` 环境中，
  增加覆盖初始仓库和 clone 后控制仓库提交路径的回归测试。
- 保留现有不可变 artifact、隔离、清理和 fail-closed 验收边界。

Runtime 语义、Hosted workflow 策略、全局 Git/Agent 配置和无关发布行为不在本
Work Item 范围内。

## 验收

1. 升级脚本的每个提交路径都有显式 repository-local 身份，绝不写入全局 Git 配置。
2. 回归测试证明在禁用全局配置时，初始仓库和 clone 后控制仓库都可以提交。
3. 成功和失败路径保留验收事实、生成清理证据，并只删除经过验证的临时运行根目录。
4. 发布 shell 测试、文档检查和 workspace 质量检查全部通过。

## 验证边界

已安装 Runtime 记录 Contract、preflight、checkpoint、verification、finish、archive、
finalization 和 close evidence。公开 Release 与 N-1 验收是不可变的外部发布证据；
发布后验收失败不得改写已发布事实。
