---
author: AI Cockpit maintainers
title: "WI-196 — Governance integrity recovery gate retry"
description: "fresh checkpoint から current-batch の recovery gate と release acceptance isolation を再検証します。"
audience:
  - maintainer
  - reviewer
workItemId: WI-196-governance-recovery-gate-retry
status: recovered
authority: canonical
lastVerifiedBy: WI-196-governance-recovery-gate-retry
---

# WI-196 — Governance integrity recovery gate retry

WI-196 は finish 後に同じ scope の correction が見つかった WI-195 の明示的 successor
です。同じ bounded scope を保持し、fresh checkpoint を作成して recovery-aware governance
gate、documentation acceptance、public-adopter isolation regression を再実行します。
predecessor は recovered history として保持し、その evidence を current verification として
再利用しません。

この Work Item の review、merge、close と、修正済み immutable public artifact による Release
acceptance が完了した後、次の batch で reference source の file-by-file 比較を開始します。

[English](WI-196-governance-recovery-gate-retry.md) ·
[简体中文](WI-196-governance-recovery-gate-retry.zh-CN.md)
