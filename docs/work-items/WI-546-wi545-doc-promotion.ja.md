---
author: AI Cockpit maintainers
title: "WI-546 — WI-545 終端ドキュメント promotion"
description: "検証済み close 後に v0.2.68 release ドキュメントを昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-546-wi545-doc-promotion
lastVerifiedBy: WI-546-wi545-doc-promotion
---

[English](WI-546-wi545-doc-promotion.md) · [简体中文](WI-546-wi545-doc-promotion.zh-CN.md)

# WI-546 — WI-545 終端ドキュメント promotion

## Objective

三言語の WI-545 release ページと parity ledger を、immutable な close 済み
Work Item 記録に同期する。ドキュメントは読者向け projection であり、`.ai`
記録は Runtime が管理する authority である。

## Scope と boundary

- 三つの WI-545 Work Item ページ。
- 三つの `docs/reference/reference-parity` ledger。
- Runtime code、generated governance records、release artifact、object
  repository、global Agent/MCP configuration は範囲外。

## Acceptance

六つの projection が terminal `Implemented` status と archive、verification、
finalization、close の evidence link を保持し、documentation、status-consistency、
promotion check に合格する。

