---
author: AI Cockpit maintainers
title: "WI-200 — Governance gate GitHub head binding"
description: "merge 前に governance gate delivery を GitHub が確認した PR head ref と SHA に bind する。"
audience:
  - maintainer
  - reviewer
workItemId: WI-200-governance-gate-github-head-binding
status: in_progress
authority: canonical
lastVerifiedBy: WI-200-governance-gate-github-head-binding
---

# WI-200 — Governance gate GitHub head binding

WI-200 は immutable な WI-199 の明示的 successor です。GitHub の直接確認により、PR #153
の head ref は WI-199 が記録した綴りではなく
`codex/wi-196-governance-recovery-gate-retry` であることが分かりました。本 Work Item は
両 predecessor の記録を保持し、merge 前に正確な head ref と SHA を bind します。

[English](WI-200-governance-gate-github-head-binding.md) ·
[简体中文](WI-200-governance-gate-github-head-binding.zh-CN.md)
