---
author: AI Cockpit maintainers
title: "WI-818 — evidence Contract 修復"
description: "必須 evidence class を検証前に明示し、過去の Contract の互換性を保つ。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized
workItemId: WI-818-evidence-contract-repair
lastVerifiedBy: WI-818-evidence-contract-repair
terminalArchive: .ai/work-items/archive/WI-818-evidence-contract-repair.contract.json
terminalVerification: .ai/evidence/WI-818-evidence-contract-repair.verification.json
terminalDecision: .ai/decisions/WI-818-evidence-contract-repair.close.json
---

[English](WI-818-evidence-contract-repair.md) · [简体中文](WI-818-evidence-contract-repair.zh-CN.md)

# WI-818 — evidence Contract 修復

## Intent と boundary

この Work Item は必須の evidence class を検証前に明示し、宣言された全 class を診断に
残し、過去の Contract を読み取り可能に保つ。Runtime が生成した archive、verification
evidence、close decision が権威であり、このページはそれらを書き換えない。

## Verification

記録された verification receipt は evidence class と過去互換性の検査に合格した。今後の
Work Item は高価な検証を始める前に、対応済みの evidence vocabulary を使用する。
