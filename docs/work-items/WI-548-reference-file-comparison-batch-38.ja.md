---
author: AI Cockpit maintainers
title: "WI-548 — governance と boundary script 比較 batch 38"
description: "Pinned reference 13 script を逐次比較し、source implementation を copy せず Rust-native または external boundary を記録する。"
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: canonical
workItemId: WI-548-reference-file-comparison-batch-38
lastVerifiedBy: WI-548-reference-file-comparison-batch-38
terminalArchive: .ai/work-items/archive/WI-548-reference-file-comparison-batch-38.contract.json
terminalVerification: .ai/evidence/WI-548-reference-file-comparison-batch-38.verification.json
terminalFinalization: .ai/decisions/WI-548-reference-file-comparison-batch-38.finalize.json
terminalDecision: .ai/decisions/WI-548-reference-file-comparison-batch-38.close.json
---

# WI-548 — governance と boundary script 比較 batch 38

## Objective

Pinned local commit `fde3380f81fea5fd2e288f7a8849f737dc074060` の維持対象 13 script を一つずつ読み、shared Rust Runtime と attached object repository の semantic counterpart と non-claim を記録します。Python module、Make orchestration、provider state、source JSON wire format は copy しません。

## File-level result

`tests/conformance/reference_file_inventory.json` が 13 path の classification、counterpart、reason を記録します。Detached uninstaller と global disable/enable は `reference-only`、その他は `implemented-different-by-design` です。詳細な逐次表は tri-language reference comparison page にあります。

## Findings と adopter inheritance

Portable な implementation omission はありません。Detached uninstaller と global disable/enable は意図した source/provider boundary であり、Runtime の欠落機能ではありません。Attached object repository は shared binary、明示的な `--repo` binding、isolated Contract/evidence/knowledge、human Outcome rule を継承しますが、source installer state、Python registry、adopter-specific policy value は継承しません。

## Acceptance

- Inventory が pinned commit の本 batch 13 path を reason と counterpart または明示的 boundary 付きで記録する。
- 本 batch に `deferred-next-batch` または `migrate-gap` を残さず、retired history は append-only とする。
- English、Simplified Chinese、Japanese の比較 page と本 Work Item の判断が一致する。
- Inventory、documentation、format、lint、workspace verification が finish 前に成功する。
