---
author: AI Cockpit maintainers
title: Intent、Scenario、Stage の束縛
description: Contract の人間定義の事実を Policy 駆動の Verification routing に束縛する境界を説明します。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-143-intent-scenario-binding
---

# Intent、Scenario、Stage の束縛

Contract の intent と scenario coverage は人間が定義する事実です。実行前に
route validator は intent が空でないこと、必須 scenario が全て存在すること、
operation と stage が一致することを確認し、既に Policy から生成された
`VerificationRequirement` に束縛します。

Validator は実装文を読んで authority、risk、assurance、T3 requirement を推測
しません。そのため high-risk route でも Planner の明示的な Policy rule と
stage/gate reference が必要です。事実の不足または route の不一致は
Verification 開始前に fail-closed になります。

`FinalDimensionsReceipt` は正確な Governance dimension 集合のままです。
`fourPillarProjection` は表示専用で、route を認可または弱体化できません。
