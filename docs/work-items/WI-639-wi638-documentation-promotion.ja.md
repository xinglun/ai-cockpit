---
author: AI Cockpit maintainers
title: WI-639 — WI-638 ドキュメント昇格
description: ガバナンス事実を変更せず、WI-638 の検証済み終端ドキュメント投影を昇格する。
audience: [maintainer, reviewer, adopter]
workItemId: WI-639-wi638-documentation-promotion
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-639-wi638-documentation-promotion
---

# WI-639 — WI-638 ドキュメント昇格

この Work Item は `tests/docs/promote_closed_work_item.py` の決定的な投影を使用し、
WI-638 の三言語 Work Item ページと三言語 reference-parity 行を更新します。変更対象は
読者向け投影だけであり、Contract、verification、archive、finalization、close の不変
レコードは権威として保持し、書き換えません。

投影はこの Rust repository に限定され、reference 実装のコピーや object/adopter repository
の変更は行いません。

## Acceptance

- helper が WI-638 の三つの Work Item ページを昇格する。
- helper が WI-638 の三つの parity 行を昇格する。
- 不変 lifecycle record が byte-for-byte 変更されない。
- documentation、parity、post-close 昇格チェックが通過する。

参照：[English](WI-639-wi638-documentation-promotion.md) · [中文](WI-639-wi638-documentation-promotion.zh-CN.md)。
