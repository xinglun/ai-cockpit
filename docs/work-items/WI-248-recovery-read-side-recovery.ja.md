---
author: AI Cockpit maintainers
title: "WI-248 — Recovery decision read-side recovery"
workItemId: WI-248-recovery-read-side-recovery
description: "Failed predecessor lifecycle を import せず、current default branch から strict な recovery-decision read validation を再 delivery します。"
audience:
  - adopter
  - maintainer
status: current
lastVerifiedBy: WI-248-recovery-read-side-recovery
authority: canonical
---

# WI-248 — Recovery decision read-side recovery

WI-242 は古い base で verified immutable archive と canonical pre-merge finalization を
生成しましたが、tri-language parity registration の欠落により hosted quality が delivery
を拒否しました。draft PR は failed delivery truth として保持します。WI-248 は successor
decision を記録し、`origin/main@7d1bd78` から開始して、`a3846e5` の reviewed Rust
implementation と regression test だけを replay します。WI-242 archive、evidence、
finalization、old lifecycle commit は import しません。

## Current read boundary

- record 時には append-only receipt の書き込み前に repository、Runtime、predecessor
  artifact、decision、timestamp、successor identity を検証します。
- Outcome/archive consumer は current recovery candidate ごとに同じ検査を繰り返し、
  regular-file と digest-bound filename 境界も確認します。
- foreign、stale、tampered、malformed、ambiguous な candidate は安定した
  `recovery_decision_invalid:<code>` diagnostic で fail closed になり、active artifact を
  移動したり Outcome を green にしたりできません。
- valid な successor/supersede decision は existing successor Contract から同じ repository、
  predecessor identity、predecessor Contract digest へ逆向きに bind される必要があります。

## Historical boundary

archived historical record は historical projection のまま読み取り可能です。current-read
validator は immutable bytes を書き換えず、legacy Runtime identity を新しい current failure
に変えません。WI-242 は preserved failed predecessor として PR #192 に残り、
`.ai/decisions/WI-242-recovery-read-side.recovery.json` が WI-248 への handoff を bind します。

## Verification

TDD regression は valid current recovery、forged repository/Runtime/predecessor binding、
record 後の predecessor/successor tamper、invalid candidate filename、rejection 時の active
artifact 不変、historical archive compatibility を検証します。documentation、parity、
governance、format、Clippy、focused repository suite、locked workspace suite は必須です。
