---
title: "WI-601 — reference test parity batch 49"
description: "次の十件の maintained reference test path を一件ずつ比較し、source 実装や wire format をコピーしない。"
author: AI Cockpit maintainers
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
workItemId: WI-601-reference-test-parity-batch-49
lastVerifiedBy: WI-601-reference-test-parity-batch-49
terminalArchive: .ai/work-items/archive/WI-601-reference-test-parity-batch-49.contract.json
terminalVerification: .ai/evidence/WI-601-reference-test-parity-batch-49.verification.json
terminalFinalization: .ai/decisions/WI-601-reference-test-parity-batch-49.finalize.json
terminalDecision: .ai/decisions/WI-601-reference-test-parity-batch-49.close.json
---

# WI-601 — reference test parity batch 49

[English](WI-601-reference-test-parity-batch-49.md) · [简体中文](WI-601-reference-test-parity-batch-49.zh-CN.md)

## Intent と boundary

固定した local reference checkout の次の maintained test path 10 件を一件ずつ再読します。portable governance semantics は Rust Runtime または repository-native gate に写し、source/provider 固有の fixture、Dependabot intake、deprecated-assets registry behavior は bounded な `reference-only` responsibility として保持します。

これは semantic parity であり、source command、Python module、JSON wire compatibility ではありません。reference checkout、object repository、global Agent/MCP 設定、immutable historical evidence は変更しません。

## Bounded result

10 path は `tests/conformance/reference_file_inventory.json` の `WI-601-reference-test-parity-batch-49` に記録します。

- 7 件は `implemented-different-by-design`。typed Contract、profile、lifecycle、trust、CI、documentation boundary が対応します。
- 3 件は `reference-only`。source 七 stack long-cycle fixture、Dependabot intake、deprecated-assets registry は source/provider boundary であり、Runtime control の欠落ではありません。

`migrate-gap` はありません。tri-language ledger、parity page、metadata sidecar、regression wrapper と本 record を同時に更新し、append-only ledger と source history は書き換えません。

## Acceptance と verification

- 各 selected path に一つの classification、counterpart set、bounded reason がある。
- 確認された portable omission はこの WI 内で修正し、黙って延期したり successor に隠したりしない。
- inventory、regression script、metadata、tri-language comparison/parity page と本 record が一致する。
- finish 前に conformance、documentation、governance-integrity、locked workspace check を通す。

次の比較 batch は reviewed release、exact cleanup、visible human Outcome の後でのみ開始します。attached object/adopter repository は shared Runtime と repository-bound isolation を継承しますが、source Python/Make module、provider policy value、stack matrix、source wire format は継承しません。
