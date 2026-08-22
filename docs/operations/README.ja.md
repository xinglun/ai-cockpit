---
author: AI Cockpit maintainers
title: "Operations"
description: "AI Cockpit repository の運用、verify、recovery、upgrade、acceptance route。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - repository_operations
---

# Operations

- [Capabilities and boundaries](../capabilities.ja.md) で governed Work Item sequence と stop condition を確認します。
- 正確な command と recovery の詳細は [Reference](../reference/README.ja.md) を参照します。
- immutable Release の verify、upgrade、rollback、post-release adopter acceptance は [Release と配布](../release/distribution.ja.md) を参照します。
- [バージョニング](../architecture/versioning.ja.md) で shared Runtime upgrade と明示的な repository migration を区別します。
- 測定または negative evidence は [パフォーマンス受入れ](../../tests/performance/README.ja.md) と [敵対的検証](../security/adversarial-validation.ja.md) を参照します。

v0.2.7 の public adopter acceptance baseline が完全に通過した target は `x86_64-unknown-linux-gnu` のみです。
他の Release target は、別の acceptance run が記録されない限り build または smoke evidence です。Legacy evidence は
過去の記録であり、新しい green verification に昇格させません。

[Current route](../current/README.ja.md) | [Getting started](../getting-started/README.ja.md) |
[English](README.md) | [中文](README.zh-CN.md)
