---
author: AI Cockpit maintainers
title: "WI-497 — reference-file comparison batch 28 retry"
description: "WI-496 の immutable な Hosted CI parity order failure 後に同じ 10 path を再配信します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-497-reference-file-comparison-batch-28-retry
predecessorWorkItemId: WI-496-reference-file-comparison-batch-28
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-497-reference-file-comparison-batch-28-retry
canonical: docs/work-items/WI-497-reference-file-comparison-batch-28-retry.md
---

# WI-497 — reference-file comparison batch 28 retry

[English](WI-497-reference-file-comparison-batch-28-retry.md) · [简体中文](WI-497-reference-file-comparison-batch-28-retry.zh-CN.md)

## Boundary

WI-496 は Hosted CI の parity order failure として immutable history に保持します。本 successor は最新の `origin/main` から同じ pinned 10 path を再配信し、parity registration と verification evidence の順序証明だけを修正します。source Python/Make implementation、source receipt、provider decision、object repository はコピーしません。

## Acceptance

- 10 path の既存 classification と source-only boundary を保持する。
- fresh verification evidence より前に三言語の WI-497 parity row を登録する。
- WI-496 recovery receipt を bind し、predecessor bytes を変更しない。
- inventory、documentation、parity、governance、Contract の Runtime check、reviewed merge、exact cleanup を完了する。
