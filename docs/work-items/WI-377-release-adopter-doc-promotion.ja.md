---
author: AI Cockpit maintainers
title: "WI-377 — close 後 documentation promotion 復旧"
description: "検証済み close 後の WI-376 三言語ドキュメントを昇格し、必須の close 後チェックを明示します。"
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

# WI-377 — close 後 documentation promotion 復旧

[English](WI-377-release-adopter-doc-promotion.md) · [简体中文](WI-377-release-adopter-doc-promotion.zh-CN.md)

## Intent

品質ゲートが要求する close 後のドキュメント投影を復旧します。Runtime と不変の v0.2.39 release/adopter evidence は変更しません。

## 範囲と境界

- 決定的な `promote_closed_work_item.py` helper で WI-376 Work Item と reference-parity 投影を昇格します。
- 将来の release で closed Work Item のドキュメントが `completed` のまま残らないよう、継承 Agent route に close 後チェックを明記します。
- Runtime、release artifact、過去の evidence bytes は変更しません。

## 結果

三言語の WI-376 ドキュメントは `implemented` となり、archive、verification、finalization、close receipt にバインドされています。close 後 promotion check を明示的な delivery step にしました。
