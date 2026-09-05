---
author: AI Cockpit maintainers
title: "WI-577 — current reference-comparison metadata synchronization"
description: "live comparison baseline と tri-language metadata projection を reviewed release に束縛する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-577-reference-metadata-sync
lastVerifiedBy: WI-577-reference-metadata-sync
terminalArchive: .ai/work-items/archive/WI-577-reference-metadata-sync.contract.json
terminalVerification: .ai/evidence/WI-577-reference-metadata-sync.verification.json
terminalFinalization: .ai/decisions/WI-577-reference-metadata-sync.finalize.json
terminalDecision: .ai/decisions/WI-577-reference-metadata-sync.close.json
---

[English](WI-577-reference-metadata-sync.md) · [简体中文](WI-577-reference-metadata-sync.zh-CN.md)

# WI-577 — current reference-comparison metadata synchronization

## 目的

reader-facing の reference comparison と parity route を pinned local reference source、
review 済み Rust baseline、published Runtime identity に同期します。小さな versioned metadata
sidecar を current fact と ledger count の single source とします。

## 範囲と境界

対象は tri-language reference page 6 枚、metadata sidecar、実行可能な metadata regression
test と documentation acceptance hook、および本 Work Item の三言語 page です。過去 batch の
paragraph と generated governance evidence は append-only のまま保持します。Runtime behavior、
object repository、global Agent/MCP 設定、source implementation copy は対象外です。

## 受入れ

- 六つの reference page が同じ current source commit、metadata sidecar、`lastVerifiedBy` を示す。
- Rust baseline と published Runtime version/digest が sidecar と一致し、ledger count は check から導出される。
- stale header、count、source lock、翻訳 page drift は CI で fail-closed になる。
- semantic classification、history evidence、object repository を書き換えない。

## 検証

active Contract と `tests/docs/reference_comparison_metadata_test.py` を参照してください。
reference inventory、documentation acceptance、Work Item status consistency、`git diff --check`
を含む bounded check を実行します。
