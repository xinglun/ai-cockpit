---
author: AI Cockpit maintainers
title: "WI-209 — release tag finalization と正しい base"
description: "verification と close の前に release successor を同期済み default branch base に bind する。"
audience:
  - maintainer
  - adopter
workItemId: WI-209-release-tag-pending-close-finalization
status: implemented
authority: canonical
lastVerifiedBy: WI-209-release-tag-pending-close-finalization
---

# WI-209 — release tag finalization と正しい base

WI-209 は不変の WI-208 attempt を修正し、`baseRevision` を同期済み
`origin/main` の merge base（`56b5e8d0584743d4442d50156adf25a6e933eaf3`）に bind
します。verification 前に `finalize-plan` で PR #158 の正確な resource context を
記録し、公開 Runtime が merge 後の finalization と structured human close を実行
できるまで待機します。

文書入口：[English](WI-209-release-tag-pending-close-finalization.md) · [简体中文](WI-209-release-tag-pending-close-finalization.zh-CN.md)
