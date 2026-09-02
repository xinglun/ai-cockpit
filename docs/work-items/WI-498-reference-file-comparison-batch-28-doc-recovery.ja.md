---
author: AI Cockpit maintainers
title: "WI-498 — batch 28 documentation recovery"
description: "Hosted CI が検出した predecessor の stale status projection を修正します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-498-reference-file-comparison-batch-28-doc-recovery
predecessorWorkItemId: WI-497-reference-file-comparison-batch-28-retry
status: recovered
authority: human-authorized
lastVerifiedBy: WI-498-reference-file-comparison-batch-28-doc-recovery
successorWorkItemId: WI-499-reference-file-comparison-batch-28-parity-order-recovery
recoveryDecision: .ai/decisions/WI-498-reference-file-comparison-batch-28-doc-recovery.recovery.json
canonical: docs/work-items/WI-498-reference-file-comparison-batch-28-doc-recovery.md
---

# WI-498 — batch 28 documentation recovery

[English](WI-498-reference-file-comparison-batch-28-doc-recovery.md) · [简体中文](WI-498-reference-file-comparison-batch-28-doc-recovery.zh-CN.md)

## Boundary

WI-497 は immutable な Hosted CI 失敗履歴として保持します。本 successor は
authoritative recovery と parity record が要求する三言語 documentation projection
だけを修正し、predecessor の archive/evidence bytes、Runtime policy、source
implementation を変更せず、object repository も操作しません。

## Acceptance

- WI-496 と WI-497 のページが `recovered` status と Runtime recovery receipt を示す。
- batch 28 の 10 classification と source-only boundary を変更しない。
- review 済み merge と exact cleanup 前に documentation、parity、inventory、governance、
  宣言した Runtime check が通過する。

## Recovery boundary

WI-498 は immutable recovery history として保持します。archive 前の hosted check
は通過しましたが、parity row と verification evidence が同じ commit で導入されたため、
厳密な prearchive order を証明できません。WI-499 が明示的な successor として bytes を保持し、
parity registration の commit を先に作成してから verification evidence を追加します。
