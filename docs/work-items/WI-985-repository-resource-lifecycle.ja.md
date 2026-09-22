---
author: AI Cockpit maintainers
title: "WI-985 — repository resource lifecycle 境界"
description: "公開 API、receipt、エラー、recovery の動作を維持したまま、resource finalization、ordinary cleanup、close 時の resource 検証を同一 crate の module へ抽出する。"
audience: [maintainer, reviewer, contributor]
status: implemented
authority: human:user
workItemId: WI-985-repository-resource-lifecycle
lastVerifiedBy: WI-985-repository-resource-lifecycle
terminalArchive: .ai/work-items/archive/WI-985-repository-resource-lifecycle.contract.json
terminalVerification: .ai/evidence/WI-985-repository-resource-lifecycle.verification.json
terminalDecision: .ai/decisions/WI-985-repository-resource-lifecycle.close.json
---

[English](WI-985-repository-resource-lifecycle.md) · [简体中文](WI-985-repository-resource-lifecycle.zh-CN.md)

# WI-985 — repository resource lifecycle 境界

この Work Item は `cockpit-repository` に既にある同一 crate 内の境界を
狭める。resource finalization、ordinary cleanup、close 時の resource 検証を
`resource_lifecycle.rs` へ移し、`lib.rs` には安定した公開 export と共通の
repository primitive を残す。

公開 signature、JSON/receipt schema、file path、書き込み順序、error semantics、
historical read、recovery behavior、および duplicate、stale、foreign、dirty、
missing、unknown resource の fail-closed 動作は変更しない。runtime 性能向上は
主張せず、protocol、CLI/MCP、release script、legacy compatibility も変更しない。

受入れは finalization/cleanup の focused regression、archive/close と recovery
regression、Contract が宣言した repository/CLI/MCP test、Runtime に束縛された
verification evidence に基づく。
