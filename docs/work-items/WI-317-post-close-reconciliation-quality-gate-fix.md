---
author: AI Cockpit maintainers
title: "WI-317 — post-close reconciliation quality gate fix"
workItemId: WI-317-post-close-reconciliation-quality-gate-fix
description: "Redeliver the bounded W316 quality-gate corrections without rewriting immutable history."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-317-post-close-reconciliation-quality-gate-fix
terminalArchive: .ai/work-items/archive/WI-317-post-close-reconciliation-quality-gate-fix.contract.json
terminalVerification: .ai/evidence/WI-317-post-close-reconciliation-quality-gate-fix.verification.json
terminalFinalization: .ai/decisions/WI-317-post-close-reconciliation-quality-gate-fix.finalize.ef51268d4b7db25d8f189d4bbd6b87faa306e48150888b884c32006a428f4f1d.json
terminalDecision: .ai/decisions/WI-317-post-close-reconciliation-quality-gate-fix.close.json
---

# WI-317 — post-close reconciliation quality gate fix

## Intent and boundary

W316 is an immutable archived delivery whose hosted quality run exposed three
bounded defects: parity rows did not follow recovery decisions, the Chinese
resource-finalization page lacked the explicit close ordering rule, and a
promotion regression still asserted an obsolete error message. This successor
preserves W316 bytes and redelivers only those corrections from the latest
`origin/main` base.

## Scope and acceptance

- W316 Contract, evidence, Outcome, Events, archive, recovery, and PR #280
  history remain byte-for-byte immutable.
- The three parity ledgers truthfully classify W312 as Implemented and W314/W315
  as Recovered, with exact recovery evidence paths.
- All three resource-finalization workflow pages state that `close` cannot occur
  before `finalize-verify` succeeds.
- The promotion regression matches the current helper error and all focused,
  full, and hosted quality gates pass without weakening any gate.
- The successor starts from the latest remote default base, is merged only after
  reviewed hosted checks, then finalized, closed, and exactly cleaned.

## Verification

Run the focused documentation/promotion/resource-finalization regressions,
documentation acceptance, the locked single-process workspace tests, and the
hosted CI checks for this exact reviewed branch using the installed Runtime.

## Related history

- W316: immutable delivery rejected by hosted quality checks; its bytes remain
  historical evidence.
- W317: bounded successor correcting only the findings from that run.

[简体中文](WI-317-post-close-reconciliation-quality-gate-fix.zh-CN.md) ·
[日本語](WI-317-post-close-reconciliation-quality-gate-fix.ja.md)
