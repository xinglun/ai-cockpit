---
author: AI Cockpit maintainers
title: "WI-987 — WI-986 documentation promotion"
description: "不変の lifecycle evidence を書き換えず、終了済み WI-986 の documentation projection を昇格する。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: authorized
workItemId: WI-987-wi986-doc-promotion
lastVerifiedBy: WI-987-wi986-doc-promotion
---

[English](WI-987-wi986-doc-promotion.md) · [简体中文](WI-987-wi986-doc-promotion.zh-CN.md)

# WI-987 — WI-986 documentation promotion

これは終了済み WI-986 の documentation projection を昇格する、範囲を
限定した documentation Work Item である。変更するのは WI-986 の三言語
ページと三つの reference-parity 行だけであり、不変の `.ai` lifecycle
evidence は書き換えない。

## Acceptance

- promotion helper が WI-986 に stale projection がないことを報告する。
- English、簡体字中国語、日本語のページが同じ archive、verification、close の
  terminal facts を保持する。
- repository-wide `--check-all` documentation projection check が通る。
- source code、release logic、または不変の governance receipt を変更しない。
