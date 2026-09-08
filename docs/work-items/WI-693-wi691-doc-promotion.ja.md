---
author: AI Cockpit maintainers
title: "WI-693 — WI-691 ドキュメント昇格"
description: "不変のガバナンス証拠から、完了した WI-691 のドキュメント投影を昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-693-wi691-doc-promotion
lastVerifiedBy: WI-693-wi691-doc-promotion
---

[English](WI-693-wi691-doc-promotion.md) · [简体中文](WI-693-wi691-doc-promotion.zh-CN.md)

# WI-693 — WI-691 ドキュメント昇格

## 意図

三言語の WI-691 reference-parity 投影を、不変の archive、verification、
finalization、close 証拠と同期する。中国語 parity の句読点を決定的な投影規則に
合わせる境界も記録する。

## 範囲

この Work Item は三言語の WI-693 ページと三言語の reference-parity 投影だけを変更する。
WI-691 のガバナンス記録は読み取り専用の証拠として扱い、Runtime の動作、ライフサイクル
意味論、ガバナンス規則、過去の証拠は変更しない。

## 受け入れ条件

- 三言語の WI-693 ページが close 後に終端証拠を参照する。
- WI-691 の三言語 parity 行が決定的な closed Work Item 投影と一致する。
- ドキュメント受け入れ、parity/status、差分検査が成功する。
