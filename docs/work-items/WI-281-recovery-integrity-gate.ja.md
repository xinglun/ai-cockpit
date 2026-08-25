---
author: AI Cockpit maintainers
title: "WI-281 — recovery integrity gate"
workItemId: WI-281-recovery-integrity-gate
description: "CI が append-only recovery head を解決し、current-cycle Work Item projection の完全性を要求します。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-281-recovery-integrity-gate
terminalArchive: .ai/work-items/archive/WI-281-recovery-integrity-gate.contract.json
terminalVerification: .ai/evidence/WI-281-recovery-integrity-gate.verification.json
terminalFinalization: .ai/decisions/WI-281-recovery-integrity-gate.finalize.75797b8a6607897f2f36b13ee0fa30e60a3cd6902b4adf0562390da662cb1ed1.json
terminalDecision: .ai/decisions/WI-281-recovery-integrity-gate.close.json
authority: canonical
---

# WI-281 — recovery integrity gate

この Work Item は、immutable predecessor に canonical retry と digest-suffixed
successor/supersession receipt が併存する場合の hosted governance gap を閉じます。
gate は有効な recovery head を deterministic に選択し、invalid candidate は
fail-closed のままにし、current release cycle が宣言する三言語の Work Item と
parity projection を要求します。
