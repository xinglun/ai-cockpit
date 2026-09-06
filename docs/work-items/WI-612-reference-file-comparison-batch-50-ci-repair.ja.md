---
title: "WI-612 — reference test parity batch 50"
description: "次の二十件の maintained reference test path を一件ずつ比較し、source 実装や wire format をコピーしない。"
author: AI Cockpit maintainers
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
workItemId: WI-612-reference-file-comparison-batch-50-ci-repair
lastVerifiedBy: WI-612-reference-file-comparison-batch-50-ci-repair
terminalArchive: .ai/work-items/archive/WI-612-reference-file-comparison-batch-50-ci-repair.contract.json
terminalVerification: .ai/evidence/WI-612-reference-file-comparison-batch-50-ci-repair.verification.json
terminalFinalization: .ai/decisions/WI-612-reference-file-comparison-batch-50-ci-repair.finalize.json
terminalDecision: .ai/decisions/WI-612-reference-file-comparison-batch-50-ci-repair.close.json
---

# WI-612 — reference test parity batch 50

[English](WI-612-reference-file-comparison-batch-50-ci-repair.md) · [简体中文](WI-612-reference-file-comparison-batch-50-ci-repair.zh-CN.md)

## Intent と boundary

固定した local reference checkout の次の maintained test path 二十件を一件ずつ再読
します。portable governance responsibility は Rust Runtime、native test、documentation
に写し、source/provider 固有 fixture と participant-study material は `reference-only`
として保持します。これは semantic parity であり、source command、Python module、
JSON wire compatibility ではありません。

## Bounded result と acceptance

完全な mapping は `tests/conformance/reference_file_inventory.json` と tri-language
comparison ledger に記録します。15 件は `implemented-different-by-design`、5 件は
`reference-only` で、portable omission と `migrate-gap` はありません。metadata、
documentation、parity、inventory check と `cargo test --locked --workspace` を通します。
attached object/adopter repository は shared Runtime と repository-bound isolation を
継承しますが、source Python/Make、provider policy、stack preset、source wire format は
継承しません。
