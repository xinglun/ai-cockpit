---
author: AI Cockpit maintainers
workItemId: WI-146-verification-cost-observation
title: Verification コスト観測
description: ガバナンスの意味を変えずに identity-bound のコスト推定と実行観測を追加する。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-146-verification-cost-observation
---

# WI-146 — Verification コスト観測

この Work Item は単一および並列 Verification のコスト推定と実行観測を追加する。
projection は advisory のみであり、Policy が決める `VerificationTier`、
`EvidenceAssurance`、protected gate、governance result を変更できない。confidence は
`complete`、`partial`、`unknown` を明示し、不明な事実を隠さない。

実装証拠：`.ai/evidence/WI-146-verification-cost-observation.verification.json`。
Close decision：`.ai/decisions/WI-146-verification-cost-observation.close.json`。
