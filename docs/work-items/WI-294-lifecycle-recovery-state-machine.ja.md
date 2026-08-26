---
author: AI Cockpit maintainers
title: "WI-294 — Lifecycle recovery state machine"
workItemId: WI-294-lifecycle-recovery-state-machine
description: "人が認可した lifecycle recovery を明示的・identity-bound・再現可能にし、predecessor bytes を書き換えません。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-294-lifecycle-recovery-state-machine
terminalArchive: .ai/work-items/archive/WI-294-lifecycle-recovery-state-machine.contract.json
terminalVerification: .ai/evidence/WI-294-lifecycle-recovery-state-machine.verification.json
terminalFinalization: .ai/decisions/WI-294-lifecycle-recovery-state-machine.finalize.json
terminalDecision: .ai/decisions/WI-294-lifecycle-recovery-state-machine.close.json
authority: canonical
---

# WI-294 — Lifecycle recovery state machine

## Intent

失敗した lifecycle transition の後に、人が認可した retry を明示的・安全・再現可能にします。

## Scope

- 合法な `checkpointed` retry state だけを復元します。
- blocked Outcome、predecessor digest、append-only recovery history を保持します。
- stale な report や completion event を再利用せず、fresh verification と finish を実行します。
- Rust Runtime、repository gate、三言語ドキュメントの投影を一致させます。

## Out of scope

Release packaging、adopter acceptance、CI replacement、Runtime module decomposition は別の境界です。

## Acceptance

- failed finish は identity-bound recovery receipt だけで retry できます。
- retry は green preflight を合成せず、immutable predecessor bytes を書き換えません。
- stale recovery candidate は新しい valid projection を隠しません。
- superseded archive は内部 digest binding を保持します。
- Rust、governance、documentation、hosted checks が通ってから close します。

## Verification

`.ai/evidence/WI-294-lifecycle-recovery-state-machine.verification.json` と reviewed PR/closure receipt を参照してください。

## Unknowns

Work Item owner は user-visible benefit を宣言していません。この項目は明示的に unknown のままです。
