---
author: AI Cockpit maintainers
title: "WI-329 — reference file comparison batch 08 CI 回帰修正"
workItemId: WI-329-reference-file-comparison-batch-08-retry
description: "immutable な WI-328 hosted inventory gate failure 後に、clean な default branch から 9 path batch を再配信する。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-329-reference-file-comparison-batch-08-retry
terminalArchive: .ai/work-items/archive/WI-329-reference-file-comparison-batch-08-retry.contract.json
terminalVerification: .ai/evidence/WI-329-reference-file-comparison-batch-08-retry.verification.json
terminalFinalization: .ai/decisions/WI-329-reference-file-comparison-batch-08-retry.finalize.json
terminalDecision: .ai/decisions/WI-329-reference-file-comparison-batch-08-retry.close.json
---

# WI-329 — reference file comparison batch 08 CI 回帰修正

## Intent と boundary

WI-328 の source comparison と inventory は archive 済みでしたが、hosted quality が
reference-only reason に将来の WI 番号を要求する brittle assertion を発見しました。
WI-328 の closed PR と immutable records は historical evidence として保持します。この
successor は synchronized default branch から同じ batch を再実行し、任意の Work Item
番号ではなく semantic boundary を gate が検査するよう修正します。

Runtime は外部共有であり、全 repository 操作は明示的な `--repo` に bind します。source
Python/Make 実装、generic Session、global Agent/MCP configuration、Runtime behavior は追加
しません。

## 修正と file-level scope

| Path | Decision |
| --- | --- |
| `tests/conformance/reference_file_inventory_test.sh` | 将来 WI 番号に依存する assertion を安定した semantic phrase assertion に置換します。 |
| `tests/conformance/reference_file_inventory.py` と `.json` | capability matrix/claim の 4 path は `reference-only` を維持し、専用の将来 Work Item を示すだけにします。 |
| `docs/reference/reference-file-comparison.*` | WI-328 の 9 分類と immutable predecessor/successor boundary を維持します。 |
| `docs/reference/reference-parity.*` | WI-328 を Recovered として記録し、verification 前に successor を登録します。 |
| `docs/work-items/WI-328-reference-file-comparison-batch-08.*` | capability-claim の将来 follow-up を WI-330 と記録し、WI-329 を CI 修正専用にします。 |
| `docs/work-items/WI-329-reference-file-comparison-batch-08-retry.*` | 3 言語で bounded repair と terminal evidence を記録します。 |

固定された 9 source path は WI-328 と同じで、5 path は
`implemented-different-by-design`、4 path は明示的な `reference-only` です。source の public
capability matrix/checker は copy せず、target gate とも宣言しません。将来の専用
capability-claim/evidence Work Item が必要です。

## Adopter feedback boundary

Cursor report は external validation です。stable lifecycle JSON、durable human Outcome replay、
close-before-next check、fail-closed start は既存 Runtime capability として記録します。
automatic IDE chat posting、diagnostic remediation、controls scaffold、close-gap convenience、
Makefile requirement は範囲外です。

## Acceptance と evidence

1. 固定 source commit と target baseline を使い、`tests/conformance/reference_file_inventory_test.sh`
   が修正後の semantic assertion を含めて pass します。
2. WI-328 inventory は `implemented-different-by-design` 5 件と `reference-only` 4 件を維持し、
   deferred-next-batch と migrate-gap はありません。
3. English、簡体中文、日本語の comparison/parity/Work Item page は predecessor recovery、
   semantic gate assertion、将来 capability-claim boundary で一致します。
4. WI-328 の historical bytes を書き換えず、source Python/Make execution や global Agent/MCP
   configuration を追加しません。
5. installed Runtime の inspect/status/doctor/agent doctor、focused gate、workspace 全体、
   hosted CI、reviewed merge、finalization、close、exact cleanup が pass します。

[English](WI-329-reference-file-comparison-batch-08-retry.md) · [简体中文](WI-329-reference-file-comparison-batch-08-retry.zh-CN.md)
