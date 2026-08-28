---
author: AI Cockpit 维护者
title: "WI-359——从同步 main 发布 v0.2.36"
workItemId: WI-359-release-v0-2-36
description: "仅从完全同步的默认分支发布清理修复，并验证真实公开制品。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-359-release-v0-2-36
capabilityClaims: [release_distribution, adopter_acceptance]
---

# WI-359——从同步 main 发布 v0.2.36

[English](WI-359-release-v0-2-36.md) · [日本語](WI-359-release-v0-2-36.ja.md)

## 目标

从已审核、合并并同步的默认分支发布 v0.2.36 清理修复。v0.2.35 的失败发布保持为不可变历史。

## 范围

- 将 workspace package、lockfile 和三语发布/版本文档统一到 v0.2.36。
- 只有在 WI-358 已完成 finalization 与 close 且默认分支同步后才创建标签。
- 只使用 hosted release workflow 生成的真实公开制品、校验和、SBOM、provenance、adopter 与清理证据。
- 安装准确的公开 macOS ARM64 binary，并用显式 repository 执行健康检查。

## 边界

不得移动、删除或重新标记 v0.2.35，也不得改写其失败 workflow 事实。不得新增 runtime 行为、修改全局
Agent/MCP 配置，或在发布验收中使用源码构建 fallback。

## 验收

1. 所有 workspace package 与 `Cargo.lock` 为 0.2.36，并通过版本一致性检查。
2. v0.2.36 只能在审核合并、默认分支同步检查通过后打标签。
3. 公开 workflow 通过 strict source quality、全部 target 构建、制品绑定，以及包含临时目录清理证明的 adopter 验收。
4. 下载的公开 binary checksum/digest 与 release manifest 一致；安装 binary 报告 0.2.36，并在显式 `--repo` 下通过 inspect/status/doctor/agent doctor。
5. 创建 v0.2.36 标签前，所有已合并交付分支和工作树都已同步并精确清理；不得遗留已合并分支或工作树。
6. v0.2.35 仍记录为失败发布历史，不得被描述为 Release。

## 验证

Runtime lifecycle evidence、审核过的 PR、hosted release workflow、公开 Release manifest/checksum、安装 binary digest
和 adopter acceptance receipt 是权威记录。
