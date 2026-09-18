---
author: AI Cockpit maintainers
title: "WI-904 — Work-item outcome インターフェース記述の単一事実源"
description: "プロトコル所有の記述から CLI、MCP、参照資料のインターフェース事実を生成する。"
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
workItemId: WI-904-interface-description-ssot
lastVerifiedBy: WI-904-interface-description-ssot
terminalArchive: .ai/work-items/archive/WI-904-interface-description-ssot.contract.json
terminalVerification: .ai/evidence/WI-904-interface-description-ssot.verification.json
terminalDecision: .ai/decisions/WI-904-interface-description-ssot.close.json
---

[English](WI-904-interface-description-ssot.md) · [简体中文](WI-904-interface-description-ssot.zh-CN.md)

# WI-904 — Work-item outcome インターフェース記述の単一事実源

この Work Item は work-item outcome の発見経路にある重複したインター
フェース事実を除去する。プロトコル所有の記述を CLI、MCP、参照資料の
マーク済み領域のソースとし、参照ページの説明文は人手で維持する。

## 受入れ

- CLI の解析、MCP の検証、生成された参照事実がプロトコル記述と一致する。
- 発見経路は決定的で、リポジトリ状態を観測せず検証プロセスも起動しない。
- Outcome 配信、認可、ライフサイクルの動作を変更しない。
