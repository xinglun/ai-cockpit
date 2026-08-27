---
author: AI Cockpit maintainers
title: "WI-339 — reference documentation foundation clean retry"
workItemId: WI-339-reference-docs-foundation-clean-retry
description: "verification 前の parity 登録を証明し、変更のない最初の 5 pinned reference governance-documentation 比較を再配信する。"
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-339-reference-docs-foundation-clean-retry
---

# WI-339 — reference documentation foundation clean retry

WI-339 は WI-338 の明示的 successor です。Hosted CI が同一 commit の
prearchive parity 登録を拒否したため、前項は immutable history として保持します。
既存の 5-file 比較の時系列証明だけを修正し、新しい reference scope は追加しません。

verification 前に parity row を登録します。source Python、Make、provider、historical tooling、
新しい Runtime behavior は追加しません。

Acceptance: 既存 5 classification を変更せず、三言語 parity ledger の prearchive 登録を
verification 前に完了し、documentation、parity、locked workspace check に成功すること。

[English](WI-339-reference-docs-foundation-clean-retry.md) ·
[简体中文](WI-339-reference-docs-foundation-clean-retry.zh-CN.md)
