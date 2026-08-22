---
author: AI Cockpit maintainers
title: "WI-156 — 物理実行と Work Item 証拠レシート"
description: "共有物理計算を Work Item の認可から分離し、偽造されたコストテレメトリを拒否します。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-156-physical-execution-receipt
workItemId: WI-156-physical-execution-receipt
---

# WI-156 — 物理実行と Work Item 証拠レシート

WI-156 は物理実行結果と、Work Item を認可するガバナンスレシートを分離します。
複数の Work Item が同じ物理結果を参照できても、各 Work Item は自身のレシートを
bind して検証しなければなりません。他の Work Item のレシートを認可または決定の
証拠として再利用することはできません。

コスト観測は助言テレメトリです。永続化またはキャッシュされた観測は、identity、
カウンター、正規化された小文字 SHA-256 digest が実行レシートと完全一致する場合
だけ受け入れます。偽造キャッシュは `unknown` と `cost_observation_invalid` として
投影され、ガバナンスを green にできません。

Evidence: `.ai/evidence/WI-156-physical-execution-receipt.verification.json`。
Decision: `.ai/decisions/WI-156-physical-execution-receipt.close.json`。
