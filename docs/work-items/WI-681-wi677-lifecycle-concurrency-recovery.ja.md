---
author: AI Cockpit maintainers
title: WI-681 — WI-677 lifecycle concurrency recovery redelivery
description: 最新の default base から P2-C の lifecycle concurrency と recovery 境界を再配信します。
workItemId: WI-681-wi677-lifecycle-concurrency-recovery
audience:
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-681-wi677-lifecycle-concurrency-recovery
---

# WI-681 — WI-677 lifecycle concurrency recovery redelivery

WI-681 は WI-677 の明示的な successor です。remote default branch の進展により
archived PR が conflicting になったため、predecessor の archive、evidence、Outcome、
finalization、recovery decision を保持し、最新の `origin/main` から同じ P2-C boundary
を再配信します。

## Boundary

各 Work Item の `finish`、`archive`、`close`、recovery decision の記録、active artifact
reconciliation を安定した OS advisory lock で直列化します。temporary file 名は process
内で一意にし、failure projection が committed terminal state を巻き戻さないように
します。public JSON、lifecycle semantics、archive layout、Runtime compatibility は変更
しません。

## Verification

同一 process と cross-process の finish race、archive/close race、missing または corrupt
projection の fail-closed 動作を concurrency tests で確認します。merge 前に locked
workspace、format、Clippy、documentation acceptance、governance integrity、hosted checks
を完了します。
