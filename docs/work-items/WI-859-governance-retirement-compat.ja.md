---
author: AI Cockpit maintainers
title: "WI-859 — governance retirement compatibility"
description: "Runtime retirement protocol と静的 governance integrity gate の整合。"
audience: [maintainer, reviewer]
status: in_progress
authority: authorized
workItemId: WI-859-governance-retirement-compat
lastVerifiedBy: WI-859-governance-retirement-compat
---

[English](WI-859-governance-retirement-compat.md) · [简体中文](WI-859-governance-retirement-compat.zh-CN.md)

# WI-859 — governance retirement compatibility

この Work Item は、静的 governance integrity gate が Runtime の append-only
retirement route を認識できるようにします。有効な retirement receipt と対応する
retired archive は履歴の cleanup evidence であり、新しい verification result や
close decision を捏造しません。無効、foreign、または digest 不一致の記録は
fail-closed のままです。

変更範囲は gate、回帰 fixture、必要な三言語ドキュメント投影に限定します。
