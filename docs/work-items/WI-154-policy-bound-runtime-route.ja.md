---
author: AI Cockpit maintainers
workItemId: WI-154-policy-bound-runtime-route
title: Policy-bound Runtime verification route
description: Policy requirement と stage/base の事実を実際の Verification receipt に接続し、no-policy 互換性を維持します。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-154-policy-bound-runtime-route
---

# WI-154 — Policy-bound Runtime verification route

Runtime は実行前に、宣言された repository/Work Item の verification
requirement を解決します。`VerificationTier` と `EvidenceAssurance` は直交
し、local route はコマンドが成功しても `T3` や `ProviderVerified` を満たした
とは主張できません。`pr`、`merge`、`release` stage では Contract の有効な
`baseRevision` が必要で、`task` では不要です。

新しい Work Item receipt は repository/Work Item identity、snapshot digest、
base revision、Policy 参照、required/actual route dimensions、affected paths、
dependency confidence を bind します。Lifecycle は binding を再検証するため、
改ざんされた receipt は finish/archive の事実になりません。typed verification
requirement のない repository は no-policy/legacy route を維持します。

[Verification route](../reference/verification-route.ja.md)、
[English](WI-154-policy-bound-runtime-route.md)、
[中文](WI-154-policy-bound-runtime-route.zh-CN.md) を参照してください。
