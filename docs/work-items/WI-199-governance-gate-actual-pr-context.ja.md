---
author: AI Cockpit maintainers
title: "WI-199 — Governance gate actual PR context"
description: "detached checkout の governance gate 修正を、merge 前に実際の reviewed branch と pull request に bind する。"
audience:
  - maintainer
  - reviewer
workItemId: WI-199-governance-gate-actual-pr-context
status: in_progress
authority: canonical
lastVerifiedBy: WI-199-governance-gate-actual-pr-context
---

# WI-199 — Governance gate actual PR context

WI-199 は immutable な WI-198 の明示的 successor です。archive 後の self-check で、branch
rename 後も WI-198 に古い branch 名が残っていることが分かりました。default-branch
discovery の実装は変更せず、本 Work Item で reviewed PR #153 を実際の branch に bind し、
strict な pre-merge finalization receipt を記録します。

[English](WI-199-governance-gate-actual-pr-context.md) ·
[简体中文](WI-199-governance-gate-actual-pr-context.zh-CN.md)
