---
author: AI Cockpit maintainers
title: ガバナンス複雑度の境界
description: Rust Runtime が source 固有の保守ツールをコピーせず、監査履歴を書き換えずに repository の増加を観測する境界。
audience:
  - contributor
  - maintainer
  - adopter
status: reference
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
---

# ガバナンス複雑度の境界

Reference project には Python/Make の複雑度レポートがあります。Rust Runtime は、その source 固有 scanner、threshold、global complexity budget を同梱しません。これは意図した境界です。保守レポートは governance decision ではなく、adopter repository の事実だと仮定できません。

## Rust Runtime が提供する事実

すべての command は明示的な repository context を持ちます。

```sh
ai-cockpit inspect --repo /path/to/repository
ai-cockpit status --repo /path/to/repository
ai-cockpit doctor --repo /path/to/repository
ai-cockpit diagnose --repo /path/to/repository --work-item WI-123
```

`inspect` は current snapshot と changed paths、`status` は repository compatibility と archive count、`doctor` は attach 済み Runtime boundary を示します。Work Item を選ぶと `diagnose` は観測できた snapshot/verification cost を報告し、欠落した計測は `unknown` のままです。

repository CI integrity gate は archive pair、parity metadata、documentation consistency を検査します。これは current repository facts を守るもので、source の historical complexity scanner の代替でも threshold の推測でもありません。

## Archive と増加の規則

Archived Contract、Summary、Outcome、evidence、decision の bytes は immutable audit history です。履歴が増えただけでは削除、compaction、別 Work Item の変更は許可されません。index 修復や履歴 compaction は retention decision を明示した別の reviewed Work Item とします。

Cost/performance observation は advisory です。required verification tier や protected check を下げず、unknown measurement を green Outcome に変えません。`VerificationTier` と `EvidenceAssurance` は独立した次元です。

## Object project の境界

Adopter repository は shared Runtime を通じて同じ request-scoped rule を受け取ります。各 command は `--repo` を付け、archive/evidence は repository ごとに分離されます。Reference の Python scanner、Make target、threshold file を adopter に暗黙にインストールしません。

