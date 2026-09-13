---
author: AI Cockpit maintainers
title: "WI-823 — terminal documentation projection repair"
description: "最近 close された Work Item の evidence-bound 三言語ドキュメントを復元する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized
workItemId: WI-823-docs-promotion
lastVerifiedBy: WI-823-docs-promotion
terminalArchive: .ai/work-items/archive/WI-823-docs-promotion.contract.json
terminalVerification: .ai/evidence/WI-823-docs-promotion.verification.json
terminalDecision: .ai/decisions/WI-823-docs-promotion.close.json
---

[English](WI-823-docs-promotion.md) · [简体中文](WI-823-docs-promotion.zh-CN.md)

# WI-823 — terminal documentation projection repair

## Intent と boundary

この限定された documentation Work Item は WI-818、WI-820、WI-822 の reader-facing page
を復元し、自身の三言語 projection を登録する。Runtime の動作、release artifact、object
repository、immutable な Contract、verification、archive、recovery、finalization、close
bytes は対象外である。

## Acceptance

- 各 Work Item に正確な英語・簡体字中国語・日本語ページがある。
- 各ページと parity row が正確な Runtime-owned evidence path を参照する。
- evidence を書き換えず documentation promotion と status check が通る。
