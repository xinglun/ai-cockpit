---
author: AI Cockpit maintainers
title: "WI-479 — WI-478 terminal documentation promotion"
description: "終了した WI-478 の release 記録を公開 projection に反映し、この Work Item 自身を close 前に登録する。"
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-479-wi478-doc-promotion
workItemId: WI-479-wi478-doc-promotion
---

# WI-479 — WI-478 terminal documentation promotion

これは documentation-only Work Item です。終了済み WI-478 の release
記録を reader-facing projection に反映し、自身の close 前に lifecycle を
三言語の台帳へ登録します。immutable な Runtime record や adopter
repository は変更しません。

[English](WI-479-wi478-doc-promotion.md) · [简体中文](WI-479-wi478-doc-promotion.zh-CN.md)

## Scope

- 三言語の WI-478 Work Item ページと reference-parity ledger を、WI-478
  の immutable lifecycle record に束縛する。
- この Work Item の close が pending の間は三言語 ledger に自身を登録し、
  verified close 後にその登録を promotion する。
- close 後の documentation promotion check を deterministic に保つ。

## Out of scope

Runtime/protocol、release packaging、CI policy、reference source の実装、
adopter repository、global Agent/MCP configuration、immutable な Contract、
evidence、archive、finalization、recovery、close bytes。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-478-release-v0-2-57`
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/docs/documentation_acceptance.sh`

このページの terminal status と evidence link は reviewed merge、
finalization、close 完了後にのみ promotion します。
