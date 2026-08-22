---
author: AI Cockpit maintainers
title: "WI-123 — Parallel Contract Boundary と Slot"
description: "Contract が所有する並列 path boundary と repository-local slot lease。"
audience:
  - adopter
  - maintainer
status: implementation
authority: canonical
lastVerifiedBy: WI-123
capabilityClaims:
  - parallel_contract_boundary
---

# WI-123 — Parallel Contract Boundary と Slot

## 目的

Work Item の並列実行を明示的、repository-local、fail-closed にします。Contract は
4 種類の path、schema、reason、`maxWorkers` を持つ `concurrencyBoundary` を追加できます。
既存の intelligence sidecar は depends、conflicts、`parallelizable` の宣言として残ります。

## 範囲と安全境界

strict な `ConcurrencyBoundary`/`ParallelSlotLease`、exact/prefix/nested glob と Windows separator
の保守的な overlap 判定、repository-local exclusive reservation、CLI/MCP surface、三言語文書と race test
を対象にします。不明・不正な boundary/path/lease は serialize して fail closed にします。lease は
repositoryId と Work Item に bind され、自動 expiry はありません。`maxWorkers` は slot 容量であり
`verify --workers` とは別です。global Agent/MCP config や current repository は作りません。

## 互換性と検証

`concurrencyBoundary` は optional なので従来の Contract/sidecar を読めます。boundary がない場合は既存の
scope 比較を使い、どちらか一方だけが boundary を持つ場合は parallel authorization を出しません。
protocol round-trip、strict unknown field、Windows path、slot capacity、duplicate-ID race、repository
isolation を targeted Rust tests で検証します。
