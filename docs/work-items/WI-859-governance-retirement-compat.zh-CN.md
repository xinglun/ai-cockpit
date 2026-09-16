---
author: AI Cockpit maintainers
title: "WI-859——治理退休兼容"
description: "让静态治理完整性门禁理解 Runtime 的退休协议。"
audience: [maintainer, reviewer]
status: implemented
authority: authorized
workItemId: WI-859-governance-retirement-compat
lastVerifiedBy: WI-859-governance-retirement-compat
terminalArchive: .ai/work-items/archive/WI-859-governance-retirement-compat.contract.json
terminalVerification: .ai/evidence/WI-859-governance-retirement-compat.verification.json
terminalDecision: .ai/decisions/WI-859-governance-retirement-compat.close.json
---

[English](WI-859-governance-retirement-compat.md) · [日本語](WI-859-governance-retirement-compat.ja.md)

# WI-859——治理退休兼容

本 Work Item 让治理完整性门禁理解 Runtime 的追加式退休路径。有效的
retirement receipt 与匹配的 retired archive 是历史收尾证据，不声称新的验证结果，
也不伪造 close decision。无效、外部来源或摘要不匹配的退休记录仍然 fail-closed。

变更范围仅限门禁、回归 fixture 和所需的三语文档投影。
