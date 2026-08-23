---
author: AI Cockpit maintainers
title: "WI-205 — v0.2.25 release と transition compatibility recovery"
description: "predecessor の base identity drift 後、同期済み default branch から v0.2.25 release boundary を再確立する。"
audience:
  - maintainer
  - adopter
workItemId: WI-205-release-v0-2-25
status: in_progress
authority: canonical
lastVerifiedBy: WI-205-release-v0-2-25
---

# WI-205 — v0.2.25 release と transition compatibility recovery

WI-205 は WI-204 の successor です。predecessor は open な predecessor branch
から開始され、実際の pull request base を正しく bind できませんでした。不変の
archive と失敗した finalization attempt は保持します。本 Work Item は verification
と archive の前に同期済み `origin/main` base を記録し、v0.2.25 public release
boundary を完了します。

Adopter acceptance では immutable な v0.2.25 Release asset だけを使用します。
Release identity、download binary/N-1 evidence、隔離 root manifest、cleanup proof、
append-only transition、terminal human decision を receipt に bind します。v0.2.24
は公開前に失敗した history として再利用しません。

文書入口：[English](WI-205-release-v0-2-25.md) · [简体中文](WI-205-release-v0-2-25.zh-CN.md)

## Acceptance boundary

1. v0.2.25 version、文書、parity が一致する。
2. Immutable な公開 Release が完全な manifest、checksum、archive、SBOM、Formula、
   provenance evidence を備える。
3. Download した v0.2.25 が source fallback なしの隔離環境で adopter と
   v0.2.23→v0.2.25 N-1 acceptance を通過する。
4. Installed v0.2.25 が append-only finalization transition を受け入れて記録する。
5. WI-204 recovery、base/Runtime identity、evidence reuse、isolation、cleanup、
   三言語 Outcome が監査可能である。
