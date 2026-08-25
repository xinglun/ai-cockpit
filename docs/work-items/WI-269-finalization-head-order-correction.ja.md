---
author: AI Cockpit maintainers
title: "WI-269 — Finalization head-order correction"
workItemId: WI-269-finalization-head-order-correction
description: "reviewed archive/evidence commit を安定させた後に finalization を記録します。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-269-finalization-head-order-correction
terminalArchive: .ai/work-items/archive/WI-269-finalization-head-order-correction.contract.json
terminalVerification: .ai/evidence/WI-269-finalization-head-order-correction.verification.json
terminalFinalization: .ai/decisions/WI-269-finalization-head-order-correction.finalize.b64cf4237f6474b2dcc9d4be732a67fce482bea85d799eb0c438e95e6d43a24f.json
terminalDecision: .ai/decisions/WI-269-finalization-head-order-correction.close.json
authority: canonical
---

# WI-269 — Finalization head-order correction

## Intent

WI-268 は、evidence/archive commit より前に pre-merge finalization receipt を記録したため reviewed head が stale になる順序不具合を明らかにしました。本 successor は parity 登録を先に行い、archive/evidence を commit してから安定した head に対して finalization を記録します。

## Scope と evidence boundary

- WI-268 と WI-267 の immutable recovery bytes を保持します。
- evidence が Git history に現れる前に successor parity row を登録します。
- archive/evidence を commit してから pre-merge finalization receipt を記録します。
- finalization commit は canonical receipt のみに限定し、hosted review、exact cleanup、structured close を完了します。

## Verification

- `cargo test --locked --workspace`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/work_item_status_consistency_test.sh`
- `--repo` を明示した installed Runtime の lifecycle と visible human Outcome

最終 handoff は可視の `Outcome: 🟢`、`Outcome: 🟡`、`Outcome: 🔴` のいずれかで始まり、status、unknowns、evidence、human decision、next action を含めます。
