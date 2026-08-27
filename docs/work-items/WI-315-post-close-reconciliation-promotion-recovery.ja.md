---
author: AI Cockpit maintainers
title: "WI-315 — post-close reconciliation promotion recovery"
workItemId: WI-315-post-close-reconciliation-promotion-recovery
description: "immutable な W314 履歴を書き換えず、recovered predecessor の promotion semantics を修正する。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-315-post-close-reconciliation-promotion-recovery
---

# WI-315 — post-close reconciliation promotion recovery

## Intent と boundary

W314 は immutable な hosted failure delivery です。その documentation gate により、
predecessor に confirmed close がある場合でも有効な successor recovery を無視する projection
defect が見つかりました。本 successor は最新 default branch からこの狭い gate 条件だけを修正し、
W314 の履歴を書き換えません。

## Scope と acceptance

- repository に bind された有効な `successor` または `supersede` recovery は、predecessor の
  close projection に関係なく predecessor を履歴として扱います。
- retry、malformed、foreign、non-canonical recovery は通常の promotion 検証を継続し、無効な
  evidence は fail closed します。
- confirmed approved close と有効な successor recovery の組み合わせ、および無効な recovery
  variant を regression でカバーします。
- 三言語の文書と parity に verification 前に W315 を登録し、W314 の failure/recovery boundary
  を保持します。

## Verification

documentation regression、documentation acceptance、`cargo fmt`、warning deny の clippy、
locked single-process workspace test を実行します。merge 前に正確な review 済み branch が
hosted CI を通過する必要があります。Governance interface は installed Runtime を使用します。

## Related history

- W314: 本 defect を発見した immutable predecessor の hosted failure delivery。
- W315: promotion projection のみを修正する bounded successor。

[English](WI-315-post-close-reconciliation-promotion-recovery.md) ·
[简体中文](WI-315-post-close-reconciliation-promotion-recovery.zh-CN.md)
