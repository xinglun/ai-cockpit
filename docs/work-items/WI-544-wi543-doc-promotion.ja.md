---
author: AI Cockpit maintainers
title: "WI-544 — WI-543 terminal documentation promotion"
description: "完了した WI-543 parity projection を昇格し、証拠リンクを保持します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-544-wi543-doc-promotion
lastVerifiedBy: WI-544-wi543-doc-promotion
---

[English](WI-544-wi543-doc-promotion.md) · [简体中文](WI-544-wi543-doc-promotion.zh-CN.md)

## Goal

三言語の WI-543 parity projection を immutable な close 済み evidence と同期します。
本ページは reader-facing projection であり、Runtime の記録が authoritative です。

## Scope and boundary

- 三つの `docs/reference/reference-parity` projection。
- 本 WI-544 の三言語 reader page と parity 登録。
- Runtime behavior、source inventory semantics、release artifact、生成 `.ai` 記録、
  object repository は対象外です。

## Acceptance

- 三つの parity row が WI-543 の archive、verification、finalization、close 参照を保持する。
- documentation、governance integrity、parity status、Work Item status checks が成功する。
- close 後の documentation promotion check が stale projection を報告しない。

## Evidence boundary

昇格で変更するのは reader-facing projection のみです。immutable Contract、
verification、finalization、close record は Runtime 管理の evidence として保持します。
