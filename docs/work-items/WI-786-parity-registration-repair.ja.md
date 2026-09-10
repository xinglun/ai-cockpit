---
author: AI Cockpit maintainers
title: "WI-786 — recovered parity-registration repair"
description: "ガバナンスゲートが要求する証拠バインド済み三言語 parity 投影を修復する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-successor-recovery
workItemId: WI-786-parity-registration-repair
lastVerifiedBy: WI-786-parity-registration-repair
---

[English](WI-786-parity-registration-repair.md) · [简体中文](WI-786-parity-registration-repair.zh-CN.md)

# WI-786 — recovered parity-registration repair

## Intent and boundary

WI-786 は WI-785 の明示的な successor です。Hosted governance gate により、
recovered parity 行には実際の hashed recovery decision パスが必要であり、
WI-784 行には verification evidence のバインドも必要であることが判明しました。

この Work Item は三つの reference-parity 文書と自身の三言語 planning ページ
だけを変更します。WI-785、WI-784、およびそれ以前の archive、evidence、
Outcome、event、decision のバイト列は immutable のままです。Runtime source、
製品動作、release 状態、無関係な文書は対象外です。

## Acceptance mapping

- WI-781、WI-782、WI-784 の英語・簡体字中国語・日本語の行が、実際の
  recovery decision と必要な evidence パスをバインドする。
- WI-783 の terminal facts と WI-785 の recovery facts が三言語で意味的に
  等価である。
- WI-786 の planning ページは close 前の形を維持し、verified close 後に
  Runtime の terminal evidence からのみ投影される。

## Verification

`bash tests/docs/parity_status_check.sh`

`bash tests/docs/documentation_acceptance.sh`

`cargo test --locked --workspace`

前身のバインドは
`.ai/decisions/WI-785-wi784-doc-promotion.recovery.json` です。
