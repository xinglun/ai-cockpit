---
author: AI Cockpit maintainers
title: "WI-538 — WI-537 terminal documentation promotion"
description: "完了した WI-537 capability documentation を昇格し、この限定投影を登録する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-538-wi537-doc-promotion
lastVerifiedBy: WI-538-wi537-doc-promotion
---

[English](WI-538-wi537-doc-promotion.md) · [简体中文](WI-538-wi537-doc-promotion.zh-CN.md)

## Goal

WI-537 の三言語 reader page と parity row を不変の close 済み evidence に同期し、
本ドキュメント Work Item 自体も登録する。

## Scope and boundary

- WI-537 の三言語 reader page と三つの parity ledger。
- WI-538 自身の三言語 reader page と parity 登録。
- Runtime の挙動、生成 `.ai` record、release artifact、対象 repository は対象外。

## Acceptance

- WI-537 の投影が terminal evidence と `implemented` status を持つ。
- WI-538 の page と parity row が言語間で相互リンクされ、意味的に同等である。
- documentation acceptance、status consistency、parity integrity、close 後 promotion
  check がすべて成功する。

## Evidence boundary

Promotion は reader-facing projection のみを変更する。不変の Contract、verification、
finalization、close record は Runtime の管理下に残す。
