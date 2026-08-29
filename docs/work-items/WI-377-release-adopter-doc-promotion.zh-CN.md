---
author: AI Cockpit maintainers
title: "WI-377 — 关闭后文档晋级修复"
description: "在 WI-376 已验证关闭后晋级三语文档，并明确必须执行的关闭后检查。"
workItemId: WI-377-release-adopter-doc-promotion
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-377-release-adopter-doc-promotion
terminalArchive: .ai/work-items/archive/WI-377-release-adopter-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-377-release-adopter-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-377-release-adopter-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-377-release-adopter-doc-promotion.close.json
capabilityClaims: [documentation_governance, release_quality]
---

# WI-377 — 关闭后文档晋级修复

[English](WI-377-release-adopter-doc-promotion.md) · [日本語](WI-377-release-adopter-doc-promotion.ja.md)

## 目的

恢复质量门要求的关闭后文档投影。Runtime 以及不可变的 v0.2.39 发布/采用者证据保持不变。

## 范围与边界

- 使用确定性的 `promote_closed_work_item.py` 晋级 WI-376 Work Item 和 reference-parity 投影。
- 在继承的 Agent 路由中记录关闭后的必要检查，防止未来发布让已关闭 Work Item 的文档停留在 `completed`。
- 不修改 Runtime、发布物或历史证据字节。

## 结果

三种语言的 WI-376 文档均为 `implemented`，并绑定 archive、verification、finalization 和 close 收据。关闭后晋级检查已成为明确的交付步骤。
