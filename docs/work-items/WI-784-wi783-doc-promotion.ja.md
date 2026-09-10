---
author: AI Cockpit maintainers
title: "WI-784 — WI-783 terminal documentation promotion"
description: "WI-783 の close 済み証拠を bounded な三言語ドキュメント投影へ昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-documentation-promotion
workItemId: WI-784-wi783-doc-promotion
lastVerifiedBy: WI-784-wi783-doc-promotion
---

[English](WI-784-wi783-doc-promotion.md) · [简体中文](WI-784-wi783-doc-promotion.zh-CN.md)

# WI-784 — WI-783 terminal documentation promotion

## Intent と boundary

WI-784 は WI-783 の close 後に行う bounded documentation Work Item である。
不変の WI-783 Contract、verification、finalization、close レコードを、英語・
簡体字中国語・日本語の Work Item ページと reference-parity 表へ投影する。

Runtime のソース、製品動作、release 状態、governance rule、WI-781/WI-782/
WI-783 の不変レコードは変更しない。

## Scope

- WI-783 の terminal documentation projection を三言語で昇格する。
- close 前に本 Work Item 自身の三つの planning page と parity row を登録し、
  close 後の projection を bounded に保つ。
- promotion helper と documentation acceptance check を再現可能に保つ。

## Verification

宣言する documentation check は次のとおり。

`python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-783-parity-finalization-recovery --check`

`bash tests/docs/documentation_acceptance.sh --repo <repo>`

terminal projection は WI-784 の不変 archive、verification、finalization、close
レコードから導出する。Runtime がこの Work Item を close するまでは、これらの
terminal field は意図的に存在しない。
