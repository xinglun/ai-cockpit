---
author: AI Cockpit maintainers
title: Governance profile と cost の分離
description: Profile intensity、verification strength、assurance、operation-specific escalation を分離する境界。
audience:
  - contributor
  - maintainer
  - adopter
status: implemented
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
---

# Governance profile と cost の分離

Rust Runtime には `light`、`standard`、`strict` の quality route があります。Route は change に対する verification intensity を示しますが、cost target や organization の assurance level ではありません。`release` は operation class であり、4 番目の profile ではありません。

次の次元を分けます。

- `VerificationTier`（`T0`–`T3`）は verification strength。
- `EvidenceAssurance`（`SelfDeclared`、`RepositoryVerified`、`ProviderVerified`、`EnterpriseVerified`）は evidence provenance。
- Cost observation は measured work の advisory fact だけです。

Effective route は stage、risk、declared operation、protected gate、repository policy で引き上げられます。Planner は tier/escalation を提案できますが、requirement は policy または protected gate に trace できなければなりません。Requested profile は route を上げられますが floor を下げません。

Release 関連 operation では release preflight と distribution evidence が必要になる場合があります。Non-release strict Work Item は strict という理由だけで release graph を継承しません。Policy が `T3` または `ProviderVerified` を要求する場合、local-only run は完了を主張できず、該当 provider/external evidence が必要です。

Adopter repository も shared Runtime を通じて同じ profile/cost boundary を使います。process-global current project や hidden planner policy を作らず、timing/cache は弱い route を認可しません。

