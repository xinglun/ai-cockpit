---
author: AI Cockpit maintainers
title: WI-646——v0.2.87 发布
description: 发布 Runtime，并验证公开不可变产物与 adopter 边界。
audience: [adopter, maintainer, reviewer]
workItemId: WI-646-release-v0-2-87
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-646-release-v0-2-87
capabilityClaims: [release_distribution, reference_comparison, adopter_acceptance]
---

# WI-646——v0.2.87 发布

[English](WI-646-release-v0-2-87.md) · [日本語](WI-646-release-v0-2-87.ja.md)

## 意图

在 Runtime 版本绑定修正后发布 v0.2.87，并验证公开 Release 产物、校验和、
SBOM/来源证明、安装路径及隔离 adopter 验收。

## 边界

本 Work Item 更新 Runtime 版本及发布/文档投影，不复制参考源、不改写历史治理
bytes、不操作对象工程，也不修改全局 Agent/MCP 配置。发布验收只能使用公开不可变
产物；源码工作区和本地 target binary 不能替代 Release。

## 验证

工作区、文档、发布策略、源码质量、公开 adopter 与 N-1 升级检查必须通过。安装的
公开 binary 必须报告 v0.2.87，下载摘要必须与公开 manifest 和 receipt 一致；仓库
必须回到 `ready_on_base`。

关闭后，front matter 链接的终态记录是权威 Contract、验证、最终化和人工决定证据。
