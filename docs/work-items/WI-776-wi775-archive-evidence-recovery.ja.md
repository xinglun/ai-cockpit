---
author: AI Cockpit maintainers
title: "WI-776 — WI-775 archive-evidence recovery"
description: "verification と archive の間で stale になった WI-775 evidence を bounded に recovery する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorization-for-successor-after-governed-archive-failure
workItemId: WI-776-wi775-archive-evidence-recovery
lastVerifiedBy: WI-776-wi775-archive-evidence-recovery
terminalArchive: .ai/work-items/archive/WI-776-wi775-archive-evidence-recovery.contract.json
terminalVerification: .ai/evidence/WI-776-wi775-archive-evidence-recovery.verification.json
terminalFinalization: .ai/decisions/WI-776-wi775-archive-evidence-recovery.finalize.json
terminalDecision: .ai/decisions/WI-776-wi775-archive-evidence-recovery.close.json
---

[English](WI-776-wi775-archive-evidence-recovery.md) · [简体中文](WI-776-wi775-archive-evidence-recovery.zh-CN.md)

# WI-776 — WI-775 archive-evidence recovery

## Intent と boundary

WI-776 は immutable な yellow WI-775 archive の明示的 successor である。WI-775 の pass 済み
verification は verification と archive の間の commit により repository snapshot が進み、stale
になった。本 Work Item は WI-775、PR #758、すべての predecessor bytes を保持し、fresh evidence
で同じ documentation/governance projection を完了する。

Runtime source、product behavior、authorization semantics、exit code、performance implementation、
historical evidence は変更しない。

## Acceptance

- WI-775 は immutable に保持し、recovery decision で bind する。
- English、Simplified Chinese、Japanese の WI-775/WI-776 page と parity row は Runtime state と一致し、
  terminal evidence を捏造しない。
- parity registration は reviewed PR の fresh verification evidence より先に commit する。
- WI-776 は full Runtime lifecycle と exact cleanup を完了する。
