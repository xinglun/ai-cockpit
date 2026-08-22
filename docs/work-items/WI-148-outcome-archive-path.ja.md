---
author: AI Cockpit maintainers
title: "WI-148 — Archive 済み Outcome の path projection"
description: "Work Item の archive 後も生成された Outcome と human handoff の参照を有効に保つ。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-148-outcome-archive-path
---

# WI-148 — Archive 済み Outcome の path projection

active Work Item directory は一時的な lifecycle state です。Work Item を archive
するとき、Runtime は archive manifest と digest を確定する前に、生成された
Outcome、Task Outcome report、event、`changedPaths` の参照を対応する archive path
へ投影します。これにより raw record と人間向け handoff が、存在しなくなった active
file を指さないようにします。

この projection は新しい archive を作成するときだけ適用します。既存の historical
archive bytes は不変であり、backfill や書き換えは行いません。

Evidence: `.ai/evidence/WI-148-outcome-archive-path.verification.json`。
Close decision: `.ai/decisions/WI-148-outcome-archive-path.close.json`。
