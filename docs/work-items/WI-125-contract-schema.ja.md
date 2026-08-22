---
author: AI Cockpit maintainers
workItemId: WI-125-contract-schema
title: Contract V2 schema completeness
description: 旧 bytes を書き換えず、残りの typed Contract V2 lineage と governance field を追加する。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: documentation-acceptance
---

# WI-125 — Contract V2 schema completeness

## 目的

共有 Runtime と repository-local Protocol を分離したまま、reference Work
Item model を読むための Rust Contract boundary を完成させる。本 WI は typed
field と deterministic な cross-field check を追加するが、reference の
Python runtime や Makefile workflow は copy しない。

## 実装内容

- `baseCommit`、`baselineDirtyPaths`、`archiveSequence`、`resumeHistory`、
  `synchronizationCheckpoint`、`synchronizationHistory`、`guidelines`、
  `preReviewWarnings`、optional な `acceptance` を typed に対応。
- identity level、actor、scope、evidence payload を含む repository-local
  authority と destructive approval evidence を typed に対応。
- Contract V2 の mode は `investigate`、`author_todo`、`code`、`review`、
  `cleanup` に限定し、`code` では `unknowns` を空、`notCodable: false` とする。
- unknown nested field、malformed lineage、空 path/guideline、未承認の
  synchronization checkpoint、連続しない history、不十分な approval evidence を拒否。
- protocol-v1 record と legacy `baseRevision`、一行 intent は読み取り可能なまま保持し、過去の bytes は rewrite しない。

## 境界

Summary、WIII、Outcome、evidence strictness、release check、README、MCP、
reference の Python/Makefile runtime は対象外。Approval record は repository
provenance の説明であり、人の認証や provider/enterprise review の代替ではない。

## 検証

- `cargo test --locked -p cockpit-protocol --test contract_v2`
- `cargo test --locked -p cockpit-repository --test contract_schema`
- merge 前に locked workspace の全 test と lint を実行する。

人向け handoff では `Outcome: 🟢`、`Outcome: 🟡`、`Outcome: 🔴` のいずれかを表示し、
status、unknowns、evidence、human decision、next action を折りたたみログに依存せず表示する。
