---
author: AI Cockpit maintainers
title: "WI-549 — v0.2.70 发布与公开制品验收"
description: "发布下一份不可变 Runtime 基线，并仅使用公开下载制品完成验收。"
audience: [maintainer, reviewer, adopter]
status: in-progress
authority: canonical
workItemId: WI-549-release-v0-2-70
lastVerifiedBy: WI-549-release-v0-2-70
---

[English](WI-549-release-v0-2-70.md) · [日本語](WI-549-release-v0-2-70.ja.md)

# WI-549 — v0.2.70 发布与公开制品验收

## 目标

从已审查的默认分支发布 v0.2.70，作为下一份不可变 Runtime 基线。失败的
v0.2.68 标签保留为历史，绝不复用。

## 范围与边界

- 将 workspace 包身份和锁文件更新到 v0.2.70。
- 保持英文、简体中文、日文发布/版本文档一致，明确 tag、校验和、SBOM、
  provenance、attestation 和 adopter 验收步骤。
- 将发布绑定到已关闭并完成 promotion 的 WI-548 参考对照批次。
- 对象工程、全局 Agent/MCP 配置、参考源复制以及 Runtime 行为变更不在本
  Work Item 范围内。

## 发布验收

1. 包元数据、锁文件和发布文档统一识别 v0.2.70；v0.2.68 保留为不可变的
   发布失败历史。
2. Release CI 生成带有 release manifest、SHA256SUMS、SBOM、provenance、
   attestation 以及 tag/源提交/Runtime 身份绑定的公开制品。
3. 发布后的 adopter 与 N-1 验收只使用隔离目录中的公开下载制品，证明禁止
   写入根目录和清理，并保留可审计 receipt。
4. 同一份公开二进制能够在显式仓库上下文下 inspect、status、doctor，并治理
   本仓库。

## 验证边界

Contract 验收原文保持原语言。本地化页面只改变展示标签，不翻译或改写治理事实。
失败发布必须如实记录，标签不能复用。对象工程验收是外部只读步骤，只有对象工程团队
提供 receipt 后才能宣称通过。
