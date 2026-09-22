---
author: AI Cockpit maintainers
title: "WI-984 — WI-982 documentation promotion"
description: "不変の lifecycle evidence を書き換えず、終了済み WI-982 の documentation projection を昇格する。"
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
workItemId: WI-984-doc-promotion
lastVerifiedBy: WI-984-doc-promotion
---

[English](WI-984-doc-promotion.md) · [简体中文](WI-984-doc-promotion.zh-CN.md)

# WI-984 — WI-982 documentation promotion

これは終了済み WI-982 に対して repository promotion helper を実行する、範囲を
限定した documentation Work Item である。変更するのは WI-982 の三言語ページ、
三つの reference-parity 行、および本 WI-984 の三言語ページだけであり、不変の
`.ai` lifecycle evidence は書き換えない。

## Acceptance

- promotion helper が WI-982 に stale projection がないことを報告する。
- English、簡体字中国語、日本語のページが同じ archive、verification、close の
  terminal facts を保持する。
- source code、release、または不変の governance receipt を変更しない。
