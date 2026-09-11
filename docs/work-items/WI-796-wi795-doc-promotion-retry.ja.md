---
author: AI Cockpit maintainers
title: "WI-796 — WI-795 ordered documentation redelivery successor"
description: "WI-795 の immutable な順序失敗を保持し、ordered な documentation promotion を完了する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized-successor-recovery
workItemId: WI-796-wi795-doc-promotion-retry
lastVerifiedBy: WI-796-wi795-doc-promotion-retry
terminalArchive: .ai/work-items/archive/WI-796-wi795-doc-promotion-retry.contract.json
terminalVerification: .ai/evidence/WI-796-wi795-doc-promotion-retry.verification.json
terminalFinalization: .ai/decisions/WI-796-wi795-doc-promotion-retry.finalize.e98e9a07ee6e979dade4b7884f937e1ceb9e298d41c937e59a544ef5d3a86c43.json
terminalDecision: .ai/decisions/WI-796-wi795-doc-promotion-retry.close.json
predecessorWorkItemId: WI-795-wi794-doc-promotion
recoveryDecision: .ai/decisions/WI-795-wi794-doc-promotion.recovery.json
---

[English](WI-796-wi795-doc-promotion-retry.md) · [简体中文](WI-796-wi795-doc-promotion-retry.zh-CN.md)

# WI-796 — WI-795 ordered documentation redelivery successor

## Recovery boundary

WI-796 は WI-795 の bounded successor です。WI-795 は immutable evidence として
保持します。parity registration と verification evidence が同じ commit で導入
されたため、history を書き換えずに required な registration-before-evidence の
順序を修復できません。Runtime の recovery decision はこの事実を保持し、この
successor は新しい ordered documentation boundary を提供します。

WI-796 は documentation projection とその Runtime governance record だけを変更
します。Rust behavior、Runtime behavior、release asset、authorization semantics、
および predecessor の archive、evidence、finalization、recovery bytes は変更しません。

## Acceptance

- WI-794 の terminal projection は、immutable な archive、verification、finalization、
  close record に bind されたままです。
- WI-796 自身の三言語 page と parity row は、新しい verification evidence の導入前に
  registration されます。
- English、簡体字中国語、日本語は同じ governance facts、unknowns、recovery boundary、
  operational consequences を保持します。
- Documentation、parity、status-consistency、governance-integrity check は predecessor
  record を書き換えずに pass します。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-794-release-v0-2-90-closure --check`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
