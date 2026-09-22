---
author: AI Cockpit maintainers
title: "WI-986 — WI-985 documentation promotion"
description: "不変の lifecycle evidence を書き換えず、終了済み WI-985 の documentation projection を昇格する。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: authorized
workItemId: WI-986-wi985-doc-promotion
lastVerifiedBy: WI-986-wi985-doc-promotion
---

[English](WI-986-wi985-doc-promotion.md) · [简体中文](WI-986-wi985-doc-promotion.zh-CN.md)

# WI-986 — WI-985 documentation promotion

これは終了済み WI-985 の repository resource lifecycle documentation
projection を昇格する、範囲を限定した documentation Work Item である。
変更するのは WI-985 の三言語ページと三つの reference-parity 行だけであり、
不変の `.ai` lifecycle evidence は書き換えない。

## Acceptance

- promotion helper が WI-985 に stale projection がないことを報告する。
- English、簡体字中国語、日本語のページが同じ archive、verification、close の
  terminal facts を保持する。
- repository-wide `--check-all` documentation projection check が通る。
- source code、release logic、または不変の governance receipt を変更しない。
