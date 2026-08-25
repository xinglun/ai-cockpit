---
author: AI Cockpit maintainers
title: "WI-243 — pre-merge finalization reason policy"
workItemId: WI-243-finalization-reason-policy
description: "pre-merge finalization に Runtime が検証できる空でない reason を要求します。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-243-finalization-reason-policy
terminalArchive: .ai/work-items/archive/WI-243-finalization-reason-policy.contract.json
terminalVerification: .ai/evidence/WI-243-finalization-reason-policy.verification.json
terminalFinalization: .ai/decisions/WI-243-finalization-reason-policy.finalize.json
terminalDecision: .ai/decisions/WI-243-finalization-reason-policy.close.json
authority: canonical
---

# WI-243 — pre-merge finalization reason policy

WI-243 は pre-merge finalization の reason を明示的な監査項目にします。
gate は Runtime が検証した空でないテキストを利用し、文書化されていない
magic token は要求しません。repository、Contract、evidence、Runtime、PR、
base、head、resource context、blocked/unmerged の binding は fail-closed
です。

権威ある記録は、archive Contract、verification evidence、finalization chain、
close decision、三言語 parity 行です。
