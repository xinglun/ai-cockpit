---
author: AI Cockpit maintainers
title: WI-764——v0.2.88 发布
description: 发布下一版 Runtime，并验证不可变公开产物与 adopter 边界。
audience: [adopter, maintainer, reviewer]
workItemId: WI-764-release-v0-2-88
status: implemented
authority: human-authorized
lastVerifiedBy: WI-764-release-v0-2-88
terminalArchive: .ai/work-items/archive/WI-764-release-v0-2-88.contract.json
terminalVerification: .ai/evidence/WI-764-release-v0-2-88.verification.json
terminalFinalization: .ai/decisions/WI-764-release-v0-2-88.finalize.json
terminalDecision: .ai/decisions/WI-764-release-v0-2-88.close.json
capabilityClaims: [release_distribution, reference_comparison, adopter_acceptance]
---

# WI-764——v0.2.88 发布

[English](WI-764-release-v0-2-88.md) · [日本語](WI-764-release-v0-2-88.ja.md)

## 意图

在性能测量 Work Item 完成后发布 v0.2.88，并验证不可变公开 Release 产物、校验和、
SBOM/来源证明、安装边界、隔离 adopter 验收及 N-1 升级边界。

## 边界

本 Work Item 更新 Runtime 包版本和当前发布文档。不修改 Runtime 行为，不复制参考源，
不改写历史治理或发布 bytes，不修改全局 Agent/MCP 配置，不预创建 provider Release，
也不复用已保留的 tag。发布验收只能使用不可变公开产物；源码 checkout 或 workspace
binary 不能替代 Release。

## 验证

reviewed PR、源码质量、版本一致性、发布策略、五目标构建、manifest/校验和、
SBOM/来源证明、平台 smoke、staged adopter、公开 adopter 与 N-1 升级检查必须通过。
安装的下载 binary 必须报告 v0.2.88 并与公开 manifest 和摘要一致，仓库最终回到
`ready_on_base`。

关闭后，front matter 链接的终态记录是权威 Contract、验证、最终化和人工决定证据。
