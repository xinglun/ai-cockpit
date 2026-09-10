---
author: AI Cockpit maintainers
title: "WI-771 — WI-770 terminal documentation promotion"
description: "検証済みでクローズされた WI-770 のドキュメント投影を終端状態へ昇格する。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: human:repository-owner
workItemId: WI-771-wi770-doc-promotion
lastVerifiedBy: WI-772-wi771-doc-promotion-recovery
recoveryDecision: .ai/decisions/WI-771-wi770-doc-promotion.recovery.json
---

[English](WI-771-wi770-doc-promotion.md) · [简体中文](WI-771-wi770-doc-promotion.zh-CN.md)

# WI-771 — WI-770 terminal documentation promotion

検証済みでクローズされた `WI-770-performance-terminal-docs-recovery` の
ドキュメントと reference-parity 投影を終端状態へ昇格し、過去の evidence や
Runtime は変更しません。元の WI-771 delivery は Runtime の finish/archive snapshot より
先に merge されたため、immutable な recovery decision がこの successor の再検証を bind します。

対象は WI-770 の六つのドキュメント投影、WI-771 の三言語 recovery projection、および
本 Work Item の governance records だけです。Runtime behavior、性能実装、リリース動作、
バージョンメタデータ、WI-770/WI-771 の過去の archive/evidence/finalization/close bytes は対象外です。

closed Work Item promotion helper の `--check-all` 成功後に終端リンクを投影します。WI-771 の
recovered projection は immutable な archive、verification、recovery decision に bind し、
fresh verification、finalization、close は WI-772 が担当します。

## Recovery state

WI-771 は immutable な recovered predecessor です。PR #755 は merge 済みですが、WI-771 の
finish/archive snapshot より前に merge されたため、元の verification snapshot は現在の完了証拠
として再利用しません。追加された recovery decision
`.ai/decisions/WI-771-wi770-doc-promotion.recovery.json` が WI-772 に新しい検証、finalization、
close の境界を割り当てます。Runtime と性能 behavior は変更しません。
