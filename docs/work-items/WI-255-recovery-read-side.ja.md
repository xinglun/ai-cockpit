---
author: AI Cockpit maintainers
title: "WI-255 — Recovery decision read-side validation"
workItemId: WI-255-recovery-read-side
description: "Outcome/archive consumption 前に current recovery decision を再検証し、immutable historical records を保持します。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-255-recovery-read-side
terminalArchive: .ai/work-items/archive/WI-255-recovery-read-side.contract.json
terminalVerification: .ai/evidence/WI-255-recovery-read-side.verification.json
terminalFinalization: .ai/decisions/WI-255-recovery-read-side.finalize.70b8faaab38e83dcd7d4fe55892abfe4c553ec1efb369bf81c2e259a9fe8566b.json
terminalDecision: .ai/decisions/WI-255-recovery-read-side.close.json
authority: canonical
---

# WI-255 — Recovery decision read-side validation

WI-255 は synchronized default branch から current recovery read-side を再構築します。
unmerged PR #192/#202 から reviewed code、tests、user-facing boundary だけを選択的に
移行し、WI-242/WI-248 lifecycle bytes を copy/rewrite したり current evidence として
提示したりしません。

## Acceptance boundary

- current recovery candidate は bounded regular non-symlink JSON file でなければなりません。
  duplicate key、malformed/oversized input、canonical JSON digest と一致しない digest suffix
  filename は fail closed です。
- Outcome または superseded archive が candidate を consume する前に、repository、Work Item、
  current Runtime、predecessor Contract/Summary/Outcome/Events、timestamp、decision shape、
  successor Contract binding を再検証します。
- invalid current candidate を older valid candidate への fallback で飛ばせません。同時刻の
  valid candidates は deterministic path order で選択します。
- failure は stable な `recovery_decision_invalid` となり、current Outcome は red、active
  artifacts は移動されません。
- historical immutable archive は recorded Runtime identity/projection を保持し、current-read
  rule が遡及的に再分類することはありません。

## Verification scenarios

Contract は valid current recovery、forged current recovery、invalid current candidate
files、deterministic candidate selection、historical archive compatibility の 5 scenarios
を必須にします。focused repository tests は real filesystem artifacts で全 scenario を
cover し、その後 repository、documentation、governance、clippy、full-workspace checks を
実行します。

## Lifecycle projection

この row は verified close まで conditional です。future evidence paths は
`.ai/work-items/archive/WI-255-recovery-read-side.contract.json`、
`.ai/evidence/WI-255-recovery-read-side.verification.json`、
`.ai/decisions/WI-255-recovery-read-side.finalize.json`、
`.ai/decisions/WI-255-recovery-read-side.close.json` です。

## References

- [Agent workflow](../reference/agent-workflow.ja.md)
- [Commands](../reference/commands.ja.md)
- [Reference parity](../reference/reference-parity.ja.md)
