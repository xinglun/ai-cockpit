---
title: "WI-605 — adopter acceptance の Release API 認証"
description: "GitHub Release API への反復アクセス時も staged/public adopter acceptance を決定的に保つ。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: recovered
authority: canonical
workItemId: WI-605-release-api-auth
lastVerifiedBy: WI-605-release-api-auth
terminalArchive: .ai/work-items/archive/WI-605-release-api-auth.contract.json
terminalVerification: .ai/evidence/WI-605-release-api-auth.verification.json
terminalFinalization: .ai/decisions/WI-605-release-api-auth.finalize.json
terminalDecision: .ai/decisions/WI-605-release-api-auth.close.json
---

[English](WI-605-release-api-auth.md) · [简体中文](WI-605-release-api-auth.zh-CN.md)

# WI-605 — adopter acceptance の Release API 認証

## 目的

workflow token で Release metadata を取得し、GitHub API rate limit による
adopter/N-1 acceptance の失敗を防ぎます。artifact download は公開かつ immutable
のまま維持します。

## 境界

対象は二つの release harness、静的回帰テスト、release workflow の環境変数です。
Runtime governance、artifact 内容、installer、object repository は対象外です。

## 検証

focused harness policy test と Contract に宣言した workspace verification を実行します。
merge 前に hosted check を通過し、release 後は公開 artifact だけで isolation/cleanup evidence
を保持した acceptance を実行します。
