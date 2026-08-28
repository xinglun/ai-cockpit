---
author: AI Cockpit 维护者
title: "WI-363——发布 v0.2.37 并验收已安装 binary"
workItemId: WI-363-release-v0-2-37
description: "在发布后清理修复合并后发布下一版不可变 Release，并在隔离 adopter 流程中验证公开 binary。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-363-release-v0-2-37
capabilityClaims: [release_distribution, adopter_acceptance]
---

# WI-363——发布 v0.2.37 并验收已安装 binary

[English](WI-363-release-v0-2-37.md) · [日本語](WI-363-release-v0-2-37.ja.md)

## 目标

在发布后清理修复已经审核合并、默认分支同步后发布 v0.2.37；随后只安装
不可变的公开制品，并在本仓库上验证它。

## 范围与边界

- 将 workspace package、lockfile 和当前三语发布/版本文档统一到 v0.2.37。
- 使用审核过的 hosted release workflow，以及公开制品、checksum、SBOM/provenance、
  adopter 验收和 N-1 升级证据。
- 安装 checksum 验证过的公开 macOS ARM64 binary，并用显式 repository 做健康检查。
- 保留未公开的 v0.2.36 staged 验收失败历史。

Runtime 行为变更、历史 evidence 改写、全局 Agent/MCP 配置、源码构建 fallback、
第二种技术栈 adopter 不属于本 Work Item。

## 验收

1. Cargo metadata 与 lockfile 一致报告 0.2.37。
2. 不可变标签前，CI 与发布策略检查全部通过。
3. 公开制品从 GitHub 下载并绑定 checksum，不得用源码或 workspace binary 替代。
4. 公开 adopter 与 N-1 验收产出可审计 receipt，证明 Runtime/repository identity、隔离、
   lifecycle evidence 和临时目录清理。
5. 已安装公开 binary 在显式 `--repo` 下通过 `inspect`、`status`、`doctor`、`agent doctor`。
6. 合并、finalization、close 和精确分支/工作树清理完成后，仓库回到同步的默认分支。

## 验证边界

Runtime lifecycle 记录 Contract、checkpoint、verification、archive、finalization 和 close
证据。Hosted workflow receipt 与发布后 adopter receipt 是公开制品声明的权威依据。
历史 v0.2.36 失败 bytes 保持不变。
