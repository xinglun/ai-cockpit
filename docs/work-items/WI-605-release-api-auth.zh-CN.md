---
title: "WI-605 —— 发布验收 API 认证"
description: "在重复访问 GitHub Release API 时保持 staged 与 public adopter 验收确定。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: recovered
authority: canonical
workItemId: WI-605-release-api-auth
lastVerifiedBy: WI-605-release-api-auth
terminalArchive: .ai/work-items/archive/WI-605-release-api-auth.contract.json
terminalVerification: .ai/evidence/WI-605-release-api-auth.verification.json
terminalFinalization: .ai/decisions/WI-605-release-api-auth.finalize.json
terminalDecision: .ai/decisions/WI-605-release-api-auth.close.json
---

[English](WI-605-release-api-auth.md) · [日本語](WI-605-release-api-auth.ja.md)

# WI-605 —— 发布验收 API 认证

## 目标

通过使用 workflow token 访问 Release 元数据，避免 GitHub API 限流导致发布
adopter 与 N-1 验收失败，同时保持制品下载公开且不可变。

## 边界

本 Work Item 覆盖两个发布验收脚本、静态回归测试及发布 workflow 环境变量。
Runtime 治理语义、发布制品内容、安装器行为和对象工程均不在范围内。

## 验证

运行脚本策略测试和 Contract 声明的 workspace 验证。合并前必须通过托管检查；
发布后验收只能使用公开制品，并保留隔离与清理证据。
