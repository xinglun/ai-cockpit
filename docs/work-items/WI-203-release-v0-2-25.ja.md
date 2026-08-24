---
author: AI Cockpit maintainers
title: "WI-203 — v0.2.25 release and transition compatibility"
description: "v0.2.24 の公開失敗を回復し、新しい immutable v0.2.25 release baseline を確立する。"
audience:
  - maintainer
  - adopter
workItemId: WI-203-release-v0-2-25
status: recovered
authority: canonical
lastVerifiedBy: WI-203-release-v0-2-25
---

# WI-203 — v0.2.25 release and transition compatibility

本 Work Item は WI-202 の明示的な successor です。v0.2.24 tag と公開前に
失敗した workflow は immutable history として保持し、再利用しません。
現在の baseline は一つだけ patch を進めた v0.2.25 です。

対象は version/distribution 文書、parity と governance record、公開 Release
evidence、download 済み adopter acceptance、installed Runtime の
finalization transition に限定します。Runtime source と CI workflow 実装は
対象外です。

公開 acceptance は immutable な v0.2.25 Release asset のみを使用します。
manifest、archive と binary digest、adopter/N-1 receipt、隔離 root manifest、
cleanup proof、transition receipt、terminal Human Decision を記録します。
source checkout や workspace binary を fallback にしてはいけません。

文書入口：[English](WI-203-release-v0-2-25.md) · [简体中文](WI-203-release-v0-2-25.zh-CN.md)

## Acceptance boundary

1. Version、current distribution 文書、三言語 parity が verification 前に
   v0.2.25 と一致する。
2. 公開 Release が stable/immutable で、manifest、checksum、archive、SBOM、
   Formula、provenance evidence を備える。
3. download した v0.2.25 が source fallback なしの隔離環境で adopter と
   v0.2.23→v0.2.25 N-1 acceptance を通過する。
4. installed v0.2.25 が append-only finalization transition を受け入れ、
   記録する。
5. WI-202 recovery、Release identity、evidence reuse、isolation、cleanup、
   三言語の人向け Outcome が監査可能である。
