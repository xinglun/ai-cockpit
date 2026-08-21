---
author: AI Cockpit maintainers
title: "Protocol compatibility ルール"
description: "Repository Protocol v1 に対する現在の runtime compatibility behavior。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - protocol_compatibility
---

# Protocol compatibility ルール

## Request envelope の互換性

Core は envelope の `schemaVersion` が `2` の場合だけ
`RequestedOperationV2` と `CapabilityMappingV2` を受け付けます。この request
envelope version は adapter/Core contract であり、repository schema version
ではありません。未知の将来 version は fail closed となり、raw request に
silent downgrade されたり、authorized と扱われたりしません。

現在の runtime が実装する互換性ルールは次のとおりです。

1. repository material を実行せず protocol version を parse する。
2. governed state を read/write する前に malformed または未対応 version を拒否する。
3. record を消費する operation が必要な field を validate した場合だけ protocol major version `1` を受け入れる。
4. optional capability を黙って upgrade したり、未対応 request を pass に変換したりしない。明示的な error、unknown、または stop を返す。
5. compatibility inspection 中に historical artifact を書き換えない。

現在の runtime は protocol major `1` をサポートし、より広い minor/patch range は宣言していません。
v1 の storage contract を保つ限り runtime minor/patch release は migration ではありません。
Protocol major migration は新しい Work Item を作り、旧 evidence を保持し、source/target protocol
version を記録します。
