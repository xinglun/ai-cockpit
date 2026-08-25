---
author: AI Cockpit maintainers
title: "WI-271 — WI-270 finalization recovery"
workItemId: WI-271-finalization-recovery
description: "immutable archive を書き換えずに WI-270 を recovery し、verification/archive 前に reviewed PR context を bind します。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-271-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-271-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-271-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-271-finalization-recovery.finalize.3fbc88f554e5c352127cb3872f4e082effd03d1ce8534bd87796be2862252152.json
terminalDecision: .ai/decisions/WI-271-finalization-recovery.close.json
authority: canonical
---

# WI-271 — WI-270 finalization recovery

## Intent

WI-270 は最初の bounded reference Contract comparison を完了しましたが、
`finalize-plan` が provisional な `pullRequest: pending` context のまま archive
されました。Hosted governance は valid な finalization boundary がないことを
正しく拒否しました。この successor は WI-270 の全 bytes を保持し、実際の PR
context を verification/archive 前に bind して reviewed delivery を完了します。

## Scope

- WI-270 の archive、evidence、preflight receipt、docs、inventory を完全に保存し、
  predecessor bytes を書き換えません。
- Runtime-valid な WI-270 successor recovery decision を記録します。
- WI-271 の archive evidence を作成する前に、三言語 parity ledger で WI-270 を
  Recovered とし、WI-271 を登録します。
- reviewed PR を作成し、正確な URL で `finalize-plan` を実行してから、installed
  Runtime lifecycle と hosted checks を実行します。
- merge observation、正確な branch/worktree cleanup、finalization verification、
  structured close、visible human Outcome を完了します。

## Boundary

これは narrow な lifecycle recovery です。次の reference slice の比較、historical
evidence の書き換え、Runtime の大きな source file の refactor、global Agent/MCP
configuration の変更は行いません。Architecture cleanup は reference-comparison
batch 完了後に別 WI として境界を定め、検証します。

## Verification

- `--repo` を明示した installed Runtime
- governance integrity、parity、inventory、documentation checks
- hosted quality、Windows、reference-oracle checks
- finalization と exact cleanup receipts
- status、unknowns、evidence、human decision、next action を含む visible
  `Outcome: 🟢`、`Outcome: 🟡`、または `Outcome: 🔴`

## Terminal evidence

- Archive: `.ai/work-items/archive/WI-271-finalization-recovery.contract.json`
- Verification: `.ai/evidence/WI-271-finalization-recovery.verification.json`
- Recovery: `.ai/decisions/WI-270-reference-contract-batch.recovery.json`
- Finalization: `.ai/decisions/WI-271-finalization-recovery.finalize.json`、
  `.ai/decisions/WI-271-finalization-recovery.finalize.e1afe79cf257e0675123913123a2eca1aba0b7bf7ffa85893d0054409b76a258.json`、
  `.ai/decisions/WI-271-finalization-recovery.finalize.3fbc88f554e5c352127cb3872f4e082effd03d1ce8534bd87796be2862252152.json`
- Close: `.ai/decisions/WI-271-finalization-recovery.close.json`
- Reviewed resource: PR [#224](https://github.com/xinglun/ai-cockpit/pull/224) と merge-observation PR
  [#225](https://github.com/xinglun/ai-cockpit/pull/225) は hosted checks 通過後に merge 済みです。
