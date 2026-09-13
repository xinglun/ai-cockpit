---
author: AI Cockpit maintainers
title: "WI-821 — no-resource document promotion"
description: "local Work Item に provider finalization を捏造せず、post-close documentation promotion の厳格さを保つ。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized
workItemId: WI-821-no-resource-document-promotion
lastVerifiedBy: WI-821-no-resource-document-promotion
terminalArchive: .ai/work-items/archive/WI-821-no-resource-document-promotion.contract.json
terminalVerification: .ai/evidence/WI-821-no-resource-document-promotion.verification.json
terminalDecision: .ai/decisions/WI-821-no-resource-document-promotion.close.json
---

[English](WI-821-no-resource-document-promotion.md) · [简体中文](WI-821-no-resource-document-promotion.zh-CN.md)

# WI-821 — no-resource document promotion

## Intent と boundary

この Work Item は documentation promotion helper が明示的な no-resource Contract と
provider-bound Contract を区別するようにする。local Work Item は provider finalization
receipt を捏造せずに close でき、provider-bound Work Item は完全な identity-bound finalization
chain を引き続き要求する。

WI-819 の close record と projection 修復は release 後の governance boundary を復元するためだけに
含め、WI-819 の archived evidence は書き換えない。

## Verification

focused tests は absent/null `resourceContext`、resource-bound finalization の欠落、no-resource
terminal reference の決定性、no-write check path を検証する。
