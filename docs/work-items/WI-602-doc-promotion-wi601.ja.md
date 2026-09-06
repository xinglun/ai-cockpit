---
author: AI Cockpit maintainers
title: "WI-602 — WI-601 終端ドキュメント昇格"
description: "ガバナンス事実を変更せず、WI-601 の終端 parity projection を補完します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-602-doc-promotion-wi601
lastVerifiedBy: WI-602-doc-promotion-wi601
---

[English](WI-602-doc-promotion-wi601.md) · [简体中文](WI-602-doc-promotion-wi601.zh-CN.md)

# WI-602 — WI-601 終端ドキュメント昇格

## Objective

閉じられた WI-601 の三言語 parity row を補完し、closed Work Item の文書検査に
監査可能な唯一の終端 projection を登録します。

## Boundary

この Work Item は三つの reference-parity page と本 Work Item の三言語記録だけを
変更します。Runtime の動作、reference source の scaffold や wire format、object
repository、global Agent/MCP configuration、生成済み evidence/decision bytes は
対象外です。

## Acceptance

1. English、Chinese、Japanese の parity table に WI-601 の終端 row がそれぞれ
   ちょうど一つあり、immutable archive、verification、finalization、close の
   path を含みます。
2. 三つの WI-602 page は evidence 生成前に登録され、Contract の原文と human-owned
   boundary を保持します。
3. 文書、metadata、closed Work Item promotion の検査が生成済み governance fact を
   変更せずに成功します。
4. reference source の scaffold をコピーせず、object/adopter repository を変更しません。

## Verification

明示的な repository context で documentation acceptance、reference metadata、closed
Work Item promotion の検査と、Contract が宣言する locked workspace 検証を実行します。
