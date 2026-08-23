---
author: AI Cockpit maintainers
title: "WI-161 — Historical Runtime evidence close compatibility"
description: "Archived evidence を不変に保ち、Runtime upgrade 後の close を可能にする。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-161-historical-runtime-close
workItemId: WI-161-historical-runtime-close
---

# WI-161 — Historical Runtime evidence close compatibility

## Intent

Runtime upgrade によって、既に archive された Work Item を close できなく
してはいけません。Active Work Item は verification を実行した Runtime に
strict に bind し、archived evidence は不変の historical truth として扱います。

## Boundary

Archived Work Item を close するときは、まず current Runtime identity を
適用せずに archived verification evidence を検証します。bytes がその他の点で
有効なら、別 Runtime による記録は current failure ではなく明示的な historical
compatibility です。Resource finalization は引き続き request-scoped で、close を
実行する Runtime に bind されなければなりません。

Evidence の書き換え、historical evidence の green 化、active の `finish`/`archive`
gate の弱体化は行いません。

WI-159 が導入した Runtime command と receipt の境界は維持し、本 Work Item は
historical compatibility lane だけを定義します。

## Acceptance

1. Active lifecycle は foreign Runtime verification evidence を拒否する。
2. Archived foreign-Runtime evidence は historical として投影され、digest、identity、
   archive manifest に bind されたままである。
3. Runtime upgrade 後の close は、current resource finalization 要件を満たした場合だけ成功する。
4. English、Simplified Chinese、日本語の workflow/parity 文書が同じ境界を説明する。

## Verification

Evidence: `.ai/evidence/WI-161-historical-runtime-close.verification.json`。
Archive: `.ai/work-items/archive/WI-161-historical-runtime-close.archive.json`。
