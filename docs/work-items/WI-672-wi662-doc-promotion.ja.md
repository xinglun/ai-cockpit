---
author: AI Cockpit maintainers
title: "WI-672 — WI-662 terminal documentation promotion"
description: "完了した WI-662 P0 benchmark evidence を、ガバナンス対象の三言語 documentation projection に昇格します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-672-wi662-doc-promotion
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-672-wi662-doc-promotion
---

[English](WI-672-wi662-doc-promotion.md) · [简体中文](WI-672-wi662-doc-promotion.zh-CN.md)

# WI-672 — WI-662 terminal documentation promotion

## Intent

完了した P0 benchmark Work Item の不変な archive、verification、finalization、close
record に、WI-662 の三言語 Work Item page と reference-parity row を同期します。

## Boundary

これは documentation-only projection です。変更対象は六つの WI-662 Markdown projection
と三つの WI-672 self-registration page に限定します。不変の `.ai` lifecycle record は
read-only input とし、Runtime の挙動、benchmark evidence、履歴 record、他 agent の
worktree は変更しません。

## Authorization record

2026-09-08、人間が `user-request` により provider 権限中断後の governed continuation を
明示的に承認しました。承認は PR comment と WI-662 close receipt に記録されており、
GitHub review を偽装するものではありません。この Contract は将来の provider または
lifecycle interruption に対する境界を保持します。

## Evidence and acceptance

- WI-662 page と parity row が verified close 後の terminal `Implemented` status と正確な
  terminal evidence path を示すこと。
- P0 evidence を測定結果と制限の根拠として維持し、`user_visible_benefit_not_declared`
  を明示したまま、この documentation projection から performance benefit を主張しないこと。
- documentation、parity、status consistency、governance、closed-work-item promotion
  check が exact reviewed head で成功すること。

WI-662 terminal evidence:

- archive: `.ai/work-items/archive/WI-662-p0-benchmark-evidence.contract.json`
- verification: `.ai/evidence/WI-662-p0-benchmark-evidence.verification.json`
- finalization: `.ai/decisions/WI-662-p0-benchmark-evidence.finalize.json`
- close: `.ai/decisions/WI-662-p0-benchmark-evidence.close.json`
