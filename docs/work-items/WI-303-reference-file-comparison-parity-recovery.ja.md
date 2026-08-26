---
author: AI Cockpit maintainers
title: "WI-303 — reference-file comparison parity recovery"
workItemId: WI-303-reference-file-comparison-parity-recovery
description: "predecessor の記録を書き換えず、immutable な WI-302 比較 delivery に欠けた三言語 parity 登録を復元する。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-303-reference-file-comparison-parity-recovery
terminalArchive: .ai/work-items/archive/WI-303-reference-file-comparison-parity-recovery.contract.json
terminalVerification: .ai/evidence/WI-303-reference-file-comparison-parity-recovery.verification.json
terminalFinalization: .ai/decisions/WI-303-reference-file-comparison-parity-recovery.finalize.json
terminalDecision: .ai/decisions/WI-303-reference-file-comparison-parity-recovery.close.json
authority: canonical
---

# WI-303 — reference-file comparison parity recovery

## Intent

WI-302 は merge 済みの不変な比較 delivery ですが、merge 後の pending parity
bridge を昇格すると finalization append 境界に違反します。この successor は
recovery decision を記録し、三つの parity 投影で WI-302 を正しく Recovered とし、
古い pending registry を削除します。

## Scope and boundary

変更対象は `docs/reference/reference-parity*`、typed pending parity registry、
および本 Work Item の三言語 readable projection だけです。WI-302 の archive、
verification、finalization、recovery、merge-observation bytes は変更しません。
Runtime、CLI、CI、release、adopter、global Agent/MCP behavior も変更しません。

## Acceptance and verification

- 三つの parity 文書に、不変 predecessor と recovery evidence を示す WI-302 の
  Recovered 行と、verification 前に登録した WI-303 行が一つずつ存在する。
- recovery projection と同じ変更で pending registry が空になる。
- installed Runtime の repository-bound lifecycle と documentation/governance gate を
  通過し、現在の repository に bind した verification receipt を生成する。
- hosted checks 通過後に merge し、finalization、正確な cleanup、close を Runtime で bind する。
