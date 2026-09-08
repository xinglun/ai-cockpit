---
author: AI Cockpit maintainers
title: "WI-676 — WI-675 documentation status repair"
description: "Runtime の挙動と履歴 evidence を変更せず、WI-675 の terminal documentation projection を修復します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-676-wi675-doc-repair
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-676-wi675-doc-repair
---

[English](WI-676-wi675-doc-repair.md) · [简体中文](WI-676-wi675-doc-repair.zh-CN.md)

# WI-676 — WI-675 documentation status repair

## Intent

Governance gate を阻害していた WI-675 の terminal documentation projection を修復
します。不変の WI-675 archive、verification、finalization、close record が lifecycle
の唯一の authority です。

## Boundary

これは documentation-only の corrective Work Item です。WI-675 の三言語 page、
三つの reference-parity row、WI-676 の三つの self-registration page だけを対象に
します。production code、test、governance rule、履歴 `.ai` record は変更しません。

## Authorization record

2026-09-08、人間が `user-request` により provider または lifecycle interruption 後の
governed continuation を明示的に承認しました。正確な interruption、scope、evidence は
Contract と PR に記録され、GitHub review を偽装するものではありません。

## Acceptance and lifecycle

- WI-675 page と parity row が不変の archive、verification、finalization、close record
  に裏付けられた terminal `Implemented` status を示すこと。
- WI-676 page と pre-archive parity registration は明示的で evidence-bound であること。
- `start → preflight → checkpoint → verify → finish → archive → close` を遵守し、
  `user_visible_benefit_not_declared` を明示すること。
- exact reviewed head で documentation、parity、status consistency、closed-work-item
  promotion check が成功すること。

## Evidence

- archive：`.ai/work-items/archive/WI-675-wi670-doc-promotion.archive.json`
- verification：`.ai/evidence/WI-675-wi670-doc-promotion.verification.json`
- finalization：`.ai/decisions/WI-675-wi670-doc-promotion.finalize.json`
- close：`.ai/decisions/WI-675-wi670-doc-promotion.close.json`

