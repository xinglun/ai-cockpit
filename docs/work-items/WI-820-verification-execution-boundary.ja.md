---
author: AI Cockpit maintainers
title: "WI-820 — verification execution boundary"
description: "限定された successor の cleanup 後も、過去の検証実行境界を保持する。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: explicit-user-authorized
workItemId: WI-820-verification-execution-boundary
lastVerifiedBy: WI-820-verification-execution-boundary
terminalArchive: .ai/work-items/archive/WI-820-verification-execution-boundary.contract.json
terminalVerification: .ai/evidence/WI-820-verification-execution-boundary.verification.json
terminalDecision: .ai/decisions/WI-820-verification-execution-boundary.close.json
recoveryDecision: .ai/decisions/WI-820-verification-execution-boundary.recovery.e32d46b7f9dc633f05616acf63aea043c28edbd461257af1ebbe082eacd79aad.json
---

[English](WI-820-verification-execution-boundary.md) · [简体中文](WI-820-verification-execution-boundary.zh-CN.md)

# WI-820 — verification execution boundary

## Historical status

WI-820 の元の verification bytes は immutable のまま保持される。その後の限定された
successor が evidence class の source file 変更を扱い、WI-822 が必要な resource
finalization の修復を完了した。Runtime の `supersede` decision が lineage を記録して
おり、WI-820 は現在の新しい verification result として扱わず、文書化のために workspace
を再実行しない。

## Boundary

実行境界は full workspace 要件、identity-bound node result、exit status、timeout state、
bounded log を引き続き定義する。新しい検証は現在の Runtime と将来の Contract が担う。
