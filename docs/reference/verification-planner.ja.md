---
author: AI Cockpit maintainers
title: Policy 駆動 Verification Planner
description: Policy と Stage から追跡可能な Verification plan を作る境界を説明します。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-141-policy-planner
---

# Policy 駆動 Verification Planner

Planner は Organization、Project、Work Item の Policy layer をこの順序で
消費します。Policy rule は独立した `requiredTier` と `requiredAssurance`
を持つ `VerificationRequirement` を指定できます。`T3` は強い
Verification が必要であることだけを示し、ProviderVerified や
EnterpriseVerified を意味しません。

選択した operation と stage について、Planner は次を要求します。

- 入力された各 Policy layer に一致する rule があること
- requirement が有効で、source policy id を参照すること
- stage reference が要求された stage と一致すること
- protected gate を要求する場合、対応する gate reference があること

rule または reference が不足する場合は fail-closed です。下位 Policy は
evidence や tier/assurance を強化できますが、上位要求を弱められません。
Planner 出力には source policy id と escalation reason が記録されるため、
required tier が operation 名の隠れたルールになることはありません。

Planner は Verification requirement だけを定義し、人間の権限、provider
assurance、依存関係の完全性、実行再利用、性能免除は作りません。

checkpoint 済みで現在の preflight が red ではなく、Contract と repository
snapshot の identity が一致する active Work Item では、Summary に typed
required check がまだ無い場合でも、Runtime は Contract に宣言された check
を一度だけ実行できます。これは初回検証の実行境界であり、passed evidence
ではありません。`finish`、`archive`、`close` は引き続き正式な
identity-bound receipt を要求します。failed、重複、stale、foreign、malformed
な entry は、project process の spawn 前に引き続き fail-closed です。

WI-139C と WI-139F の歴史的 approach artifact は元の bytes を保持したまま
archive manifest に束縛され、active に現在のプロジェクト状態と誤認される
孤立 artifact は残りません。
