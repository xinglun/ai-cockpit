---
author: AI Cockpit maintainers
workItemId: WI-141-policy-planner
title: Policy 駆動 Verification Planner
description: Policy と Stage を追跡可能な Verification requirement のソースにし、歴史的 Artifact の孤立を修正します。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-141-policy-planner
---

# WI-141 — Policy 駆動 Verification Planner

この Work Item は Planner requirement を明示的な Policy layer に束縛し、監査で
見つかった二つの歴史的な生成 Artifact の孤立を修正します。dependency confidence、
Work Item 間の実行再利用、CI convergence、性能目標は実装しません。

protocol/Planner test、archive integrity test、lint、documentation acceptance が
通過した後、インストール済み Runtime が Evidence を生成します。
