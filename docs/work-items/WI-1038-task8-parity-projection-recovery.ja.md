---
author: AI Cockpit maintainers
workItemId: WI-1038-task8-parity-projection-recovery
title: Task 8 parity 投影の復旧
description: Runtime により正確に bind された successor を通じて WI-1037 の不変 archive 後の遅延登録を修正し、履歴上の順序警告を保持する。所有されない recovery は fail closed とする。
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1038-task8-parity-projection-recovery
---

[English](WI-1038-task8-parity-projection-recovery.md) · [简体中文](WI-1038-task8-parity-projection-recovery.zh-CN.md)

# WI-1038 — Task 8 parity 投影の復旧

この限定 successor は、archive 済み WI-1037 の三つの parity ledger への遅延登録を修正する。WI-1037 の archive と verification evidence は不変であり、検証前に登録されたかのように書き換えず、履歴として明示する。

## 範囲

- 有効な Runtime successor decision が WI-1038 を WI-1037 に bind し、WI-1038 の Contract scope と Summary の changed paths の両方が三つの parity 文書を所有する場合に限り、履歴投影を受け入れる。
- predecessor の元の順序を historical warning として残す。recovery の欠落、malformed、別 repository、identity 不一致、または一部 ledger のみの所有は引き続き block する。
- 既存の厳密な PR lifecycle gate と、WI-1037 の不変 archive、Summary、Outcome、events、verification evidence を保持する。
- 汎用 waiver、広範な governance redesign、Task 9 migration、release、version 変更、tag、publication は対象外。

## Acceptance

1. 全 parity ledger を所有する identity-valid successor recovery の real-Git 正例が通り、`recovered_postarchive_parity_registration` の historical warning を三件報告する。
2. successor Contract scope から parity 文書を一つでも外す負例は `stale_prearchive_parity_registration` で引き続き block する。
3. 英語・中国語・日本語の ledger は WI-1037 の不変 archive と、archive 済みで merge と close 待ちの WI-1038 を正確に示し、いずれも close 済みとは主張しない。
4. focused lifecycle fixture により WI-1038 の parity row が archive 遷移後も有効であることを示す。
5. PR #997 merge 前に、宣言された focused checks と exact-head hosted CI が通ること。merge、Task 8 cleanup、Task 9 開始は Runtime admission に従い、release 前に human review のため停止する。

宣言済みの local focused verification は pass し、Runtime は WI-1038 を archive した。archive 後の parity check で、現行 parity row が Git 上で verification evidence より後に導入される問題を検出した。evidence は保持し、登録が evidence path より先になる commit 順序で対応する。exact-head hosted CI、PR merge、formal close、cleanup は未完了。
