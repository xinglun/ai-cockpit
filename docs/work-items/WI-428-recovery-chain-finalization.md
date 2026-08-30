---
author: AI Cockpit maintainers
title: "WI-428 — Recovery-chain finalization"
description: Close the remaining recovery boundary and prevent competing successors.
audience: [contributor, maintainer]
status: in-progress
authority: governed
workItemId: WI-428-recovery-chain-finalization
predecessorWorkItemId: WI-426-recovery-successor-binding
lastVerifiedBy: WI-428-recovery-chain-finalization
---

# WI-428 — Recovery-chain finalization

## Intent and boundary

This Work Item resolves the remaining WI-426 recovery boundary through a
directly bound successor and makes recovery selection deterministic. It keeps
all predecessor Contract, Summary, Outcome, Events, evidence, and recovery
receipt bytes immutable.

In scope:

- reject a second `successor` decision that targets a different Work Item;
- retain append-only retry/supersede decisions and stable fail-closed errors;
- update the three reference-parity ledgers to the actual terminal receipts;
- document the one-successor lineage rule and verify it with Rust tests.

Out of scope: release artifacts, unrelated Work Items, global Agent/MCP
configuration, and Runtime architecture splitting.

## Acceptance and evidence

The predecessor must not accumulate an ambiguous competing successor chain.
WI-426 must be represented by a directly bound reviewed successor, WI-424 by
its immutable supersede receipt, and all three parity ledgers must point to
the actual receipts. No historical bytes may be rewritten. A competing
successor request must fail with
`recovery_decision_invalid:competing_successor`.

Verification evidence, archive manifest, finalization, close, and the merged
PR are recorded under `.ai/evidence/` and `.ai/decisions/`.

[中文](WI-428-recovery-chain-finalization.zh-CN.md) · [日本語](WI-428-recovery-chain-finalization.ja.md)
