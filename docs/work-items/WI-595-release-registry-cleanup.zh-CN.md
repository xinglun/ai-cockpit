---
author: AI Cockpit maintainers
title: "WI-595——发布 registry 清理"
description: "WI-594 关闭后移除过期的 pending-parity 投影。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-595-release-registry-cleanup
lastVerifiedBy: WI-595-release-registry-cleanup
terminalArchive: .ai/work-items/archive/WI-595-release-registry-cleanup.contract.json
terminalVerification: .ai/evidence/WI-595-release-registry-cleanup.verification.json
terminalFinalization: .ai/decisions/WI-595-release-registry-cleanup.finalize.b6cc3d25fa5d2ccdadcc9e441e9944f87e0ecd381cc845386f0ddd1f88f7adec.json
terminalDecision: .ai/decisions/WI-595-release-registry-cleanup.close.json
---

[English](WI-595-release-registry-cleanup.md) · [日本語](WI-595-release-registry-cleanup.ja.md)

# WI-595——发布 registry 清理

## 目标

从 `docs/reference/pending-parity-registry.json` 移除已关闭 WI-594 的过期条目，
并使三语 parity 投影与当前 Runtime 记录一致。历史 `.ai/` 字节保持不可变。

## 边界

本 Work Item 只修改 pending registry、parity 投影和 WI-594/WI-595 可读文档。
Runtime 行为、发布制品、对象工程以及全局 Agent/MCP 配置不在范围内。

## 验证

使用显式 repository context 运行 JSON 解析、`tests/docs/parity_status_check.sh`、
tag-mode `tests/ci/governance_integrity_gate.py`、文档验收和 status consistency 检查。
