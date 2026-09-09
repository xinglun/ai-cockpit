---
author: AI Cockpit maintainers
title: "WI-746 — WI-745 terminal recovery"
description: "Reconcile the merged WI-745 documentation successor without rewriting historical evidence."
audience: [maintainer, reviewer, adopter]
workItemId: WI-746-wi745-terminal-recovery
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-746-wi745-terminal-recovery
terminalArchive: .ai/work-items/archive/WI-746-wi745-terminal-recovery.contract.json
terminalVerification: .ai/evidence/WI-746-wi745-terminal-recovery.verification.json
terminalFinalization: .ai/decisions/WI-746-wi745-terminal-recovery.finalize.json
terminalDecision: .ai/decisions/WI-746-wi745-terminal-recovery.close.json
predecessorWorkItem: WI-745-wi743-doc-promotion
recoveryDecision: .ai/decisions/WI-745-wi743-doc-promotion.recovery.6ef6b1b0e804eaf246518a1839851b47b0de88cb9a18c73282f21fb0816d1e41.json
---

# WI-746 — WI-745 terminal recovery

WI-746 is the immutable recovery successor for the merged WI-745
documentation promotion. It preserves the WI-745 and WI-743 historical bytes,
repairs the tri-language projections, and records fresh verification through
Runtime archive. It makes no production or performance change. Runtime
finalization and the human-approved close are recorded as terminal governance
decisions.

[简体中文](WI-746-wi745-terminal-recovery.zh-CN.md) · [日本語](WI-746-wi745-terminal-recovery.ja.md)

## Recovery boundary

The Runtime recovery receipt binds WI-745, its Contract and Summary digests,
PR #723, and this successor. WI-743 remains represented by its immutable
closed evidence; the current-state prose now reflects that terminal fact.

## Evidence and lifecycle

- Fresh verification is `.ai/evidence/WI-746-wi745-terminal-recovery.verification.json`.
- Runtime generated the archive and Outcome records; finalization is recorded
  in `.ai/decisions/WI-746-wi745-terminal-recovery.finalize.json` and the
  human-approved close in `.ai/decisions/WI-746-wi745-terminal-recovery.close.json`.
- No user-visible performance benefit is claimed by this recovery.
