---
author: AI Cockpit maintainers
title: "WI-547 — v0.2.69 发布与公开产物验收"
description: "修正 v0.2.68 发布失败投影并发布新的不可变 Runtime 基线。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-547-release-v0-2-69
lastVerifiedBy: WI-547-release-v0-2-69
terminalArchive: .ai/work-items/archive/WI-547-release-v0-2-69.contract.json
terminalVerification: .ai/evidence/WI-547-release-v0-2-69.verification.json
terminalFinalization: .ai/decisions/WI-547-release-v0-2-69.finalize.json
terminalDecision: .ai/decisions/WI-547-release-v0-2-69.close.json
---

[English](WI-547-release-v0-2-69.md) · [日本語](WI-547-release-v0-2-69.ja.md)

# WI-547 — v0.2.69 发布与公开产物验收

## 目标

从已审查并同步的默认分支发布真实的 v0.2.69 Runtime 基线。失败的
v0.2.68 标签保留为不可变历史，绝不作为公开或可安装版本。

## 范围与边界

- 包版本身份与锁文件。
- 三语发布、版本策略和分发文档。
- 本版本的 Work Item 与参考源对照投影。
- 公开制品、校验和、SBOM、adopter 及安装验收，作为绑定本 Work Item 的发布证据。
- Runtime 行为、对象工程、全局 Agent/MCP 配置以及失败的 v0.2.68 标签不在范围内。

## 验收

1. 包和文档统一声明 v0.2.69，并明确 v0.2.68 是发布失败历史。
2. Release CI 和策略门生成与不可变标签、审查源提交绑定的 manifest、SHA256SUMS、SBOM、provenance 与公开制品。
3. 只使用下载的公开二进制，在隔离目录中通过验收并证明清理和禁止写入隔离；随后安装同一已校验二进制，运行显式仓库自检。

## 验证边界

Contract 验收原文保持原语言；本地化标题不翻译治理事实。发布证据必须绑定标签、archive digest、binary digest、Runtime identity 和 adopter receipt。失败发布必须如实记录，不能复用。
