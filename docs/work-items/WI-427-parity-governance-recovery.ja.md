---
author: AI Cockpit maintainers
title: "WI-427 — parity governance recovery"
description: ホスト CI が登録漏れを検出したため、recovery binding と三言語 parity ledger を再配信する。
workItemId: WI-427-parity-governance-recovery
audience: [contributor, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-427-parity-governance-recovery
terminalArchive: .ai/work-items/archive/WI-427-parity-governance-recovery.contract.json
terminalVerification: .ai/evidence/WI-427-parity-governance-recovery.verification.json
terminalFinalization: .ai/decisions/WI-427-parity-governance-recovery.finalize.json
terminalDecision: .ai/decisions/WI-427-parity-governance-recovery.close.json
---

# WI-427 — parity governance recovery

この successor は不変の recovery 履歴を保持しながら binding を再配信し、各
reference-parity ledger に選択された decision と evidence のパスを登録します。
predecessor の archive bytes は書き換えず、documentation gate も弱めません。

parity 行は archive 前の登録です。verification、レビュー済み merge、finalization、
close の receipt が揃った後にだけ `Implemented` になります。

[English](WI-427-parity-governance-recovery.md) · [中文](WI-427-parity-governance-recovery.zh-CN.md)
