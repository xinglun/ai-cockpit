---
author: AI Cockpit maintainers
title: "WI-464 — 参照ファイル比較バッチ 24 リカバリー再試行"
workItemId: WI-464-reference-file-comparison-batch-24-retry
description: "実際の provider context で限定的な source 比較を再配信します。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-464-reference-file-comparison-batch-24-retry
terminalArchive: .ai/work-items/archive/WI-464-reference-file-comparison-batch-24-retry.contract.json
terminalVerification: .ai/evidence/WI-464-reference-file-comparison-batch-24-retry.verification.json
terminalFinalization: .ai/decisions/WI-464-reference-file-comparison-batch-24-retry.finalize.json
terminalDecision: .ai/decisions/WI-464-reference-file-comparison-batch-24-retry.close.json
---

# WI-464 参照ファイル比較バッチ 24 — リカバリー再試行

## Intent

WI-464 の immutable な配信試行を保持し、実際の provider context で同じ限定比較を完了する。この recovery Work Item は範囲を拡大せず、参照実装の bytes をコピーしない。

## Source と境界

- 参照 repository: `/Users/sei-rinn/dev/workspace_python/ai-cockpit-template`
- 固定 source commit: `fde3380f81fea5fd2e288f7a8849f737dc074060`
- predecessor: `WI-464-reference-file-comparison-batch-24`
- recovery 理由: predecessor が実際の provider PR より前に placeholder PR URL を bind したため。前身の evidence は immutable のまま書き換えない。

## 比較パス

| 参照パス | Rust 側の結果 |
| --- | --- |
| `.github/workflows/compatibility.yml` | 設計上異なる実装；Rust CI は独自の action 固定版と platform policy を使う。 |
| `.github/workflows/release.yml` | 設計上異なる実装；Rust release manifest、SBOM、provenance、checksum、adopter harness が release boundary を担う。 |
| `.github/workflows/smoke.yml` | 設計上異なる実装；Rust lifecycle と release/adopter checks が source Make bridge を置き換える。 |
| `Makefile` | 設計上異なる実装；サポートされる interface は Rust CLI、Cargo checks、repository gate manifest。 |

Rust の omission は確認されなかった。source 側だけの Python/Make/installer 動作は明示的に範囲外である。

## 配信ルール

実際に review 可能な PR を先に作成し、その URL だけを `finalize-plan` で bind する。その後 preflight、checkpoint、verification、finish、archive、finalization、close を順番に実行する。predecessor recovery receipt と今回の evidence は append-only かつ repository-bound のまま保持する。

## Verification

```text
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060
python3 tests/conformance/reference_inventory_docs_test.py
```
