---
author: AI Cockpit maintainers
title: "WI-627 — Reference file inventory の rebaseline"
description: "最新の local reference checkout に file-level ledger を再 bind し、監査履歴を保持する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-627-reference-rebaseline
status: implemented
authority: canonical
lastVerifiedBy: WI-627-reference-rebaseline
---

# WI-627 — Reference file inventory の rebaseline

## Intent

file-by-file comparison ledger を最新の local reference commit
`a9224aed77b5c317b53c4551a9eec306d91ee330` に再 bind し、既存の decision を保持したまま、
changed/new path を次の semantic batch に送ります。これは ledger/documentation の修正であり、
source implementation の Rust port ではありません。

## Result

現在の inventory は 5,175 tracked path です。内訳は generated-history 4,262、
implemented-different-by-design 426、implemented-equivalent 1、not-applicable 8、
reference-only 124、deferred-next-batch 354、migrate-gap 0 です。retired path はありません。
Rust baseline は `98f12b18b978db509fc884a8a6225afeb7f10df5`、review に使った Runtime は v0.2.85、
binary digest は `sha256:ece00d0b596c4674eaf37225e95a83aacec66a4c33f0bb89857f5dcaecde3a50` です。

validator は最初に current path set 全体を検証し、その後 historical batch ownership check だけで
source-changed path を除外します。これにより正当な rebaseline を古い batch の欠落として誤検知せず、
current path 全件に classification があることは維持します。

## Boundary と adopter 継承

reference checkout は local の pinned commit のみを使い、source Python、Shell、Make、provider 設定、
source JSON wire format は copy しません。attach 済みの adopter は shared Runtime、明示的な `--repo`、
isolated Contract/evidence/knowledge、人間向け Outcome handoff を継承し、この ledger の状態とは分離されます。

## Verification

```text
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/docs/reference_comparison_metadata_test.py
```

See also: [English](WI-627-reference-rebaseline.md) · [中文](WI-627-reference-rebaseline.zh-CN.md)。
