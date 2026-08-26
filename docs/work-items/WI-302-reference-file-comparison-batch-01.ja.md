---
author: AI Cockpit maintainers
title: "WI-302 — deferred reference-file comparison batch 01"
workItemId: WI-302-reference-file-comparison-batch-01
description: "最初の10件の deferred reference source file を Rust target と比較し、bounded な semantic conclusion を記録します。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-302-reference-file-comparison-batch-01
terminalArchive: .ai/work-items/archive/WI-302-reference-file-comparison-batch-01.contract.json
terminalVerification: .ai/evidence/WI-302-reference-file-comparison-batch-01.verification.json
terminalFinalization: .ai/decisions/WI-302-reference-file-comparison-batch-01.finalize.json
terminalDecision: .ai/decisions/WI-302-reference-file-comparison-batch-01.close.json
authority: canonical
---

# WI-302 — deferred reference-file comparison batch 01

## Intent

Pinned source commit `e5acb677` に対して lexical order の最初の10件を比較し、portable
な governance semantics と source-language/provider 固有の実装境界を保持します。

## Scope and result

対象は `.ai/cockpit/bandit_low_risk_baseline.json`、`.gitattributes`、選択した3つの
GitHub metadata/workflow file、`.gitignore`、`LICENSE`、`Makefile` です。inventory は
各 source responsibility、Rust counterpart または absence、classification、reason を
記録します。compatibility と smoke workflow matrix は、別の multi-stack/second-adopter
比較が必要なため明示的に deferred のままです。

同期済み ledger と tri-language report は次のとおりです。

- `tests/conformance/reference_file_inventory.json`
- `docs/reference/reference-file-comparison.md`
- `docs/reference/reference-file-comparison.zh-CN.md`
- `docs/reference/reference-file-comparison.ja.md`

## Evidence boundary

target baseline は `a533d49dfa848d95742833f8cd1b5f7e1bb897d5`、installed Runtime `0.2.33`
（binary digest: `sha256:eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4`）で
verification しました。lifecycle の事実は archive/verification record が束縛し、この文書は
source-language Runtime や provider ownership policy を追加しない readable projection です。
