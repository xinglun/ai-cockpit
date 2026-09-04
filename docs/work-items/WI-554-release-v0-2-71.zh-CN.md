---
author: AI Cockpit maintainers
title: "WI-554 — v0.2.71 发布与公开制品验收"
description: "将能力暴露与文档修复发布为不可变 Runtime 版本。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-554-release-v0-2-71
lastVerifiedBy: WI-554-release-v0-2-71
terminalArchive: .ai/work-items/archive/WI-554-release-v0-2-71.contract.json
terminalVerification: .ai/evidence/WI-554-release-v0-2-71.verification.json
terminalFinalization: .ai/decisions/WI-554-release-v0-2-71.finalize.json
terminalDecision: .ai/decisions/WI-554-release-v0-2-71.close.json
---

[English](WI-554-release-v0-2-71.md) · [日本語](WI-554-release-v0-2-71.ja.md)

# WI-554 — v0.2.71 发布与公开制品验收

## 目标

从已审查的默认分支发布 v0.2.71，作为下一份不可变 Runtime 基线。本版本包含能力注册表、能力发现文档和 WI-552 参考对照；此前公开的 v0.2.70 保留为历史 N-1 证据。

## 范围与边界

- 将 Cargo 元数据/锁文件及英文、中文、日文的当前发布、分发、版本文档统一到 v0.2.71。
- 将发布绑定到已关闭并完成 promotion 的 WI-552 与 WI-553。
- 生成并验证五目标公开制品、manifest、校验和、SBOM、provenance、attestation 及发布后 adopter receipt。
- 对象工程、全局 Agent/MCP 配置、参考源复制和 Runtime 行为变更不在本 Work Item 范围内。

## 验收

1. Cargo 元数据、锁文件和当前发布/版本文档统一识别 v0.2.71；v0.2.70 是紧邻的上一个公开基线，失败 tag 保持不可变历史。
2. Release CI 生成带身份绑定的制品集合及完整供应链证据。
3. 公开 adopter 与 N-1 验收只使用隔离目录中的 v0.2.71 下载制品，证明清理与禁止写入根目录，并用同一二进制治理本仓库。
4. WI-552、WI-553 保持已关闭且文档已晋级；发布从干净且 ready 的默认分支开始。

## 验证边界

Contract 原文保持其原始语言，本地化页面只改变展示。对象工程验收是外部只读交接，只有对象工程团队提供 receipt 后才可宣称通过。
