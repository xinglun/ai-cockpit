---
author: AI Cockpit maintainers
workItemId: WI-876-performance-current-proof
title: 現行版 performance proof
description: 四方向 convergence の受入れに向けた Runtime と開発サイクルの paired measurement。
audience: [adopter, contributor, maintainer]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-876-performance-current-proof
terminalArchive: .ai/work-items/archive/WI-876-performance-current-proof.contract.json
terminalVerification: .ai/evidence/WI-876-performance-current-proof.verification.json
terminalDecision: .ai/decisions/WI-876-performance-current-proof.close.json
---

# WI-876 — 現行版 performance proof

この Work Item は、同じ machine、toolchain、scenario set で比較可能な Runtime latency と開発サイクル cost の evidence を作る。release 公開や Runtime behavior の変更は含まない。

## Acceptance boundary

- release-grade percentile comparison は 100 個以上の valid warm sample を要求し、99 個は diagnostic-only として gate が拒否する。
- inspect、status、Outcome、verification planning/reuse、repository scale/history、single/multi-file change、invalid evidence を paired measurement し、取得不能な理由を明示する。
- 各 external sample に read、hash、parse、Git、subprocess、execution、reuse counter を適用範囲内で bind し、不能なら理由を残す。interval overlap と diagnostics on/off overhead も報告する。
- Contract→reviewable、verification→finish、post-merge cleanup を raw sample、p50/p95、agent operation、preflight reject と分けて報告する。

現在の Runtime または host が公開しない内部 metric は unknown のまま保持する。この Work Item は release 公開を含まない。
