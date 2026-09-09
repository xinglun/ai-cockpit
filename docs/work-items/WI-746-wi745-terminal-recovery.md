---
author: AI Cockpit maintainers
title: "WI-746 — WI-745 terminal recovery"
description: "Reconcile the merged WI-745 documentation successor without rewriting historical evidence."
audience: [maintainer, reviewer, adopter]
workItemId: WI-746-wi745-terminal-recovery
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-746-wi745-terminal-recovery
predecessorWorkItem: WI-745-wi743-doc-promotion
recoveryDecision: .ai/decisions/WI-745-wi743-doc-promotion.recovery.json
---

# WI-746 — WI-745 terminal recovery

WI-746 is the current-base recovery successor for the merged WI-745
documentation promotion. It preserves the WI-745 and WI-743 historical bytes,
repairs the tri-language projections, and records fresh verification through
Runtime archive. It makes no production or performance change; provider
finalization and human close remain separate lifecycle steps.

[简体中文](WI-746-wi745-terminal-recovery.zh-CN.md) · [日本語](WI-746-wi745-terminal-recovery.ja.md)

## Recovery boundary

The Runtime recovery receipt binds WI-745, its Contract and Summary digests,
PR #718, and this successor. WI-743 remains represented by its immutable
closed evidence; the current-state prose now reflects that terminal fact.

## Evidence and lifecycle

- Fresh verification is `.ai/evidence/WI-746-wi745-terminal-recovery.verification.json`.
- Runtime generated the archive and Outcome records; finalization and close
  remain pending provider and human lifecycle steps.
- No user-visible performance benefit is claimed by this recovery.
