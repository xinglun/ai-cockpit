---
author: AI Cockpit maintainers
title: "WI-207 — release tag finalization order recovery"
description: "Runtime が要求する finalize-plan 境界より前に verification を記録した successor を不変の history として保持する。"
audience:
  - maintainer
  - adopter
workItemId: WI-207-release-tag-pending-close-finalization
status: recovered
authority: canonical
lastVerifiedBy: WI-207-release-tag-pending-close-finalization
---

# WI-207 — release tag finalization order recovery

WI-207 は immutable な recovery history として保持します。`finalize-plan` より前に
verification と archive を記録したため、installed Runtime は後続の finalization を
正しく拒否しました。必要な順序は `finalize-plan` の後に verification evidence を
記録することです。正確な PR context の bind は WI-208 で継続します。

文書入口：[English](WI-207-release-tag-pending-close-finalization.md) · [简体中文](WI-207-release-tag-pending-close-finalization.zh-CN.md)
