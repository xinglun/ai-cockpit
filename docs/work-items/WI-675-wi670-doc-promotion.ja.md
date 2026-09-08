---
author: AI Cockpit maintainers
title: "WI-675 — WI-670 terminal documentation promotion"
description: "クローズ済み WI-670 Outcome-rendering successor を統制された三言語ドキュメント投影へ反映します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-675-wi670-doc-promotion
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-675-wi670-doc-promotion
---

[English](WI-675-wi670-doc-promotion.md) · [简体中文](WI-675-wi670-doc-promotion.zh-CN.md)

# WI-675 — WI-670 terminal documentation promotion

## Intent

不変の archive、verification、finalization、close 記録に基づき、三言語の
WI-670 Work Item ページと reference-parity 行を同期します。

## Boundary

これはドキュメントだけの投影です。変更対象は WI-670 の六つの投影ファイルと
WI-675 の三つの自己登録ページだけです。不変の `.ai` lifecycle 記録は読み取り専用
入力であり、runtime の動作、governance rule、履歴 evidence、他 agent の worktree
は変更しません。

## Acceptance

- 三言語の WI-670 ページが検証済み close 後の `Implemented` 状態と正確な終端
  evidence path を示すこと。
- 三つの WI-670 parity 行が対応する終端状態と evidence を示すこと。
- documentation、parity、status-consistency、governance、closed-work-item
  promotion check が対象 head で通過すること。
- `user_visible_benefit_not_declared` を明示したまま、本投影でユーザー向けまたは
  性能上の効果を主張しないこと。

WI-670 の終端 evidence：

- archive：`.ai/work-items/archive/WI-670-wi653-outcome-render-successor.contract.json`
- verification：`.ai/evidence/WI-670-wi653-outcome-render-successor.verification.json`
- finalization：`.ai/decisions/WI-670-wi653-outcome-render-successor.finalize.json`
- close：`.ai/decisions/WI-670-wi653-outcome-render-successor.close.json`
