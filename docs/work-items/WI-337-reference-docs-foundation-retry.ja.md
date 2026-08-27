---
author: AI Cockpit maintainers
title: "WI-337 — governance documentation foundation retry"
workItemId: WI-337-reference-docs-foundation-retry
description: "WI-336 の history を保持し、最初の 5 つの pinned reference governance-documentation 比較を clean successor lifecycle で再配信する。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-337-reference-docs-foundation-retry
---

# WI-337 — governance documentation foundation retry

## Intent と recovery boundary

WI-337 は WI-336 の明示的な successor です。Runtime 0.2.33 が predecessor の
verification 前 Contract amendment chain を reconcile できなかったため、WI-336 は immutable history として保持します。
同じ 5 path を再検証し、新しい implementation scope は追加しません。

## 再利用する比較

file-level classification、Rust counterpart、non-wire boundary は
[WI-336 comparison](WI-336-reference-docs-foundation.ja.md) と三言語 ledger に記録済みです。
WI-337 は current repository と reviewed PR context に対して bytes を再検証するだけで、source Python、Make、provider、historical tooling を copy しません。

## Acceptance と verification

1. 5 つの pinned path に明確な inventory classification、counterpart、non-overclaiming reason が一つずつ残る。
2. English、簡体中文、日本語の comparison/parity ledger が一致する。
3. verification 前に GitHub resource context を bind し、current Runtime evidence を repository と snapshot に bind する。
4. Inventory、documentation、parity、locked workspace verification が成功する。

predecessor recovery: `.ai/decisions/WI-336-reference-docs-foundation.recovery.e7ccd6381b1492fd0ba72be8c7305217748f03d9c7509a7c58db693e8ba13261.json`。

[English](WI-337-reference-docs-foundation-retry.md) ·
[简体中文](WI-337-reference-docs-foundation-retry.zh-CN.md)
