---
title: "WI-598 — reference test parity batch 48"
description: "次の二十件の maintained reference test path を逐次比較し、source 実装や wire format をコピーしない。"
author: AI Cockpit maintainers
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
workItemId: WI-598-reference-test-parity-batch-48
lastVerifiedBy: WI-598-reference-test-parity-batch-48
---

# WI-598 — reference test parity batch 48

[English](WI-598-reference-test-parity-batch-48.md) · [简体中文](WI-598-reference-test-parity-batch-48.zh-CN.md)

## Intent と boundary

固定した local reference checkout の maintained file 次の二十件を一件ずつ比較します。
portable な governance semantics は Rust Runtime または repository-native gate に写し、
stack 固有の toolchain と source fixture は `reference-only` として保持します。

これは semantic parity であり、source command、Python module、JSON wire compatibility
ではありません。object repository、global Agent/MCP 設定、immutable historical evidence
は変更しません。

## Bounded result

二十 path は `tests/conformance/reference_file_inventory.json` の
`WI-598-reference-test-parity-batch-48` に記録します。

- 18 件は `implemented-different-by-design`。typed Git、repository、profile、evidence、
  CI、release boundary が対応します。
- 2 件は `reference-only`。Java runtime 選択と Bandit baseline は provider/toolchain 固有で、
  Runtime 要件ではありません。

`migrate-gap` はありません。三言語 ledger と metadata sidecar を同時に更新し、ledger は
append-only のまま source history を書き換えません。

## Acceptance と verification

- 各 path に一つの classification、counterpart set、bounded reason がある。
- 確認された portable omission はこの WI 内で修正し、黙って延期しない。
- inventory、regression script、metadata、三言語比較/parity page と本記録が一致する。
- finish 前に conformance、documentation、governance-integrity、locked workspace check を通す。

この batch 後の release は immutable public artifact と adopter/N-1 acceptance harness を使用します。
review 済み release、exact cleanup、visible human Outcome の後でのみ次の比較 batch を開始します。
