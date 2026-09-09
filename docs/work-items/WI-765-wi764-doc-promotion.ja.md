---
author: AI Cockpit maintainers
title: "WI-765 — WI-764 クローズ後のドキュメント promotion"
description: "過去のガバナンス証拠を変更せず、クローズ済み WI-764 のリリース境界ドキュメント投影を昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-765-wi764-doc-promotion
lastVerifiedBy: WI-765-wi764-doc-promotion
---

[English](WI-765-wi764-doc-promotion.md) · [简体中文](WI-765-wi764-doc-promotion.zh-CN.md)

# WI-765 — WI-764 クローズ後のドキュメント promotion

## Intent

検証済みでクローズされた WI-764 のリリース境界投影を終端ドキュメントへ
昇格し、失敗したリリースタグとすべての過去のガバナンス証拠を保持する。

## Boundary

この Work Item は WI-764 のドキュメント投影、reference-parity 台帳、および
自身のガバナンス文書だけを変更する。Runtime の動作、リリースワークフロー、
バージョンメタデータ、WI-764 の過去の archive、evidence、finalization、
cleanup、close のバイト列は変更しない。

## Verification

Runtime の検証証拠と
`python3 tests/docs/promote_closed_work_item.py --check-all` が成功しなければならない。
Hosted 検証前に pre-archive parity 行を登録し、この Work Item の検証とクローズ後に
終端リンクを投影する。
