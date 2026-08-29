---
author: AI Cockpit maintainers
title: "WI-380 — WI-379 provider finalization recovery"
description: "Bind the reviewed successor delivery and close the documentation batch without rewriting WI-379 history."
workItemId: WI-380-reference-documentation-batch-18-finalization-recovery
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-380-reference-documentation-batch-18-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-380-reference-documentation-batch-18-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-380-reference-documentation-batch-18-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-380-reference-documentation-batch-18-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-380-reference-documentation-batch-18-finalization-recovery.close.json
capabilityClaims: [governance_integrity, resource_finalization]
---

# WI-380 — WI-379 provider finalization recovery

[简体中文](WI-380-reference-documentation-batch-18-finalization-recovery.zh-CN.md) · [日本語](WI-380-reference-documentation-batch-18-finalization-recovery.ja.md)

## Intent and boundary

WI-379 delivered the reference documentation in reviewed PR #343, but it was
archived before the provider PR identity was known. This explicit successor
preserves WI-379's immutable archive, evidence, Outcome, and recovery decision,
then records a real provider-bound lifecycle for the recovery itself.

## Scope

- Keep the exact WI-379 predecessor digests and recovery lineage visible.
- Mark WI-379 as recovered and register this successor in all three parity documents.
- Bind this Work Item's actual reviewed PR context before verification.
- Prove exact branch/worktree cleanup before close.

## Out of scope

Runtime code, release artifacts, global Agent/MCP configuration, and every
immutable WI-379 archive/evidence/Outcome/PR byte.

## Acceptance

- The recovery decision binds the predecessor Contract, Summary, Outcome,
  Events, repository, and Runtime identities.
- WI-379 bytes remain unchanged and are explicitly labeled historical/recovered.
- The successor PR context is bound before verification evidence is recorded.
- Hosted checks, installed Runtime verification, finalization, close, and the
  visible human Outcome all pass.

## Verification and terminal records

Use the installed Runtime with an explicit `--repo`, documentation/governance
checks, and `cargo test --locked --workspace`. After reviewed merge, record:

- Archive: `.ai/work-items/archive/WI-380-reference-documentation-batch-18-finalization-recovery.contract.json`
- Verification: `.ai/evidence/WI-380-reference-documentation-batch-18-finalization-recovery.verification.json`
- Finalization: `.ai/decisions/WI-380-reference-documentation-batch-18-finalization-recovery.finalize.json`
- Close: `.ai/decisions/WI-380-reference-documentation-batch-18-finalization-recovery.close.json`
