---
author: AI Cockpit maintainers
title: "WI-400——v0.2.40 公开 Release adopter 验收"
description: "在隔离的 adopter 工程中从零验证不可变 v0.2.40 Release 二进制。"
workItemId: WI-400-release-v0-2-40-adopter-acceptance
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-400-release-v0-2-40-adopter-acceptance
capabilityClaims: [release_acceptance, repository_isolation, evidence_reuse]
---

# WI-400——v0.2.40 公开 Release adopter 验收

[English](WI-400-release-v0-2-40-adopter-acceptance.md) · [日本語](WI-400-release-v0-2-40-adopter-acceptance.ja.md)

## 意图

从零证明公开且不可变的 v0.2.40 Release 可以治理全新的 adopter，且
Runtime identity、evidence reuse、生命周期记录和全局目录隔离都可独立审计。

## 边界

本 Work Item 只处理发布后制品验收、临时 adopter 及其清理收据、已关闭
WI-399 投影的晋升，以及生成的验收证据保留。不修改 Runtime 语义、参考源
功能对齐、对象工程代码或全局 Agent/MCP 配置。验收脚本禁止源码构建回退。

## 验收

1. 公开 v0.2.40 archive 和 binary 从 Release 下载，并与 manifest、SHA-256
   identity 核对。
2. 全新 adopter 获得隔离 scaffold 和独立 repository identity；在人类字段补齐
   前，`first-adopter-smoke` 必须保持 `not_ready`。
3. 真实 Work Item 生命周期记录 schema-2 evidence、精确复用与重新执行、结构化
   close 决定及 Runtime identity。
4. HOME/XDG 目录保持不变，隔离 Runtime 写入目录有清晰清单，且写入收据后删除临时
   run root。

## 验证边界

被测 Runtime 只能是已发布 Release。验收产物属于发布后证据，不改写 Release 真相或历史记录。
