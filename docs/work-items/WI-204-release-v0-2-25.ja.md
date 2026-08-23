---
author: AI Cockpit maintainers
title: "WI-204 — v0.2.25 release と transition compatibility recovery"
description: "verification と archive の前に resource context を bind して v0.2.25 release boundary を再実行する。"
audience:
  - maintainer
  - adopter
workItemId: WI-204-release-v0-2-25
status: in_progress
authority: canonical
lastVerifiedBy: WI-204-release-v0-2-25
---

# WI-204 — v0.2.25 release と transition compatibility recovery

WI-204 は WI-203 の明示的な successor です。WI-203 は `finalize-plan` が
実際の branch、worktree、pull request context を bind する前に archive された
ため、不変の history として保持します。本 Work Item は verification と archive
の前に context を bind し、同じ v0.2.25 release boundary を再実行します。

公開 acceptance は immutable な v0.2.25 Release asset のみを使います。Release
identity、download 済み adopter/N-1 receipt、isolation と cleanup evidence、
append-only transition、terminal human decision を記録します。v0.2.24 tag は
公開前に失敗した history のため再利用しません。

文書入口：[English](WI-204-release-v0-2-25.md) · [简体中文](WI-204-release-v0-2-25.zh-CN.md)

## Acceptance boundary

1. Version、distribution 文書、すべての parity row が v0.2.25 と一致する。
2. Immutable な公開 Release が manifest、checksum、archive、SBOM、Formula、
   provenance evidence を備える。
3. Download した v0.2.25 が source fallback なしの隔離環境で adopter と
   v0.2.23→v0.2.25 N-1 acceptance を通過する。
4. Installed v0.2.25 が append-only finalization transition を受け入れて記録する。
5. WI-203 recovery、Runtime identity、evidence reuse、isolation、cleanup、
   三言語の人向け Outcome が監査可能である。
