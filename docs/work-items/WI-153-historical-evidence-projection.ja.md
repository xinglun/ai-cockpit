---
author: AI Cockpit maintainers
title: "WI-153 — Historical evidence projection"
description: "旧 Runtime の有効な archived evidence を履歴として投影し、active の fail-closed 検証を維持します。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-153-historical-evidence-projection
workItemId: WI-153-historical-evidence-projection
---

# WI-153 — Historical evidence projection

WI-153 は、immutable な archived evidence を保持しながら、旧 Runtime identity で作成された有効な証拠と現在の検証失敗を区別します。旧 Runtime が生成した有効な v2 archived evidence は `historical_evidence_not_revalidated` を伴う履歴 yellow として表示されます。破損、改ざん、または identity 不一致の evidence は引き続き fail-closed です。Active Work Item の foreign Runtime identity は red のままです。

三言語 parity index も修正し、WI-147 から WI-152 までを追加しました。既存の Work Item archive と evidence の bytes は書き換えていません。

Evidence: `.ai/evidence/WI-153-historical-evidence-projection.verification.json`。
Decision: `.ai/decisions/WI-153-historical-evidence-projection.close.json`。
