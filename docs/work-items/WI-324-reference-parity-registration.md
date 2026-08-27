---
author: AI Cockpit maintainers
title: "WI-324 — reference parity registration recovery"
workItemId: WI-324-reference-parity-registration
description: "Repair the hosted documentation-governance omission discovered after the immutable WI-323 archive."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-324-reference-parity-registration
terminalArchive: .ai/work-items/archive/WI-324-reference-parity-registration.contract.json
terminalVerification: .ai/evidence/WI-324-reference-parity-registration.verification.json
terminalFinalization: .ai/decisions/WI-324-reference-parity-registration.finalize.json
terminalDecision: .ai/decisions/WI-324-reference-parity-registration.close.json
---

# WI-324 — reference parity registration recovery

## Intent and goal

Repair the missing tri-language `reference-parity` registrations discovered by
the hosted `docs_governance_integrity` gate for WI-323. Preserve WI-323's
immutable archive and failed-delivery history; make the recovered successor
auditable and independently reviewable.

## Scope and boundaries

Register WI-323 as a recovered immutable predecessor and WI-324 as its bounded
successor in the English, Simplified Chinese, and Japanese parity ledgers.
Carry forward the already-reviewed WI-323 inventory, comparison pages, Human
Benefit pages, conformance generator/test, and tri-language Work Item record.
Add a tri-language WI-324 record and run the same documentation/conformance
checks before creating a fresh PR.

Do not rewrite predecessor archive, evidence, or recovery bytes; add Runtime
features; alter CI policy; copy source Python/Make artifacts; or change global
Agent/MCP configuration.

## Acceptance and verification

1. All three parity ledgers register WI-323 and WI-324 with consistent links,
   status, and recovery explanation.
2. The carried-forward inventory and documentation tests pass from the clean
   `origin/main` baseline.
3. Hosted `docs_governance_integrity` and all other required PR checks pass.
4. The predecessor archive digest and recovery binding remain unchanged, and
   the successor Contract/evidence bind the explicit repository context.

## Recovery evidence

The predecessor archive and hosted failure are referenced by
`.ai/decisions/WI-323-reference-documentation-foundation.recovery.json`. This
successor exists only because the predecessor was already archived when the
omission was discovered; no new feature scope is introduced.

[简体中文](WI-324-reference-parity-registration.zh-CN.md) ·
[日本語](WI-324-reference-parity-registration.ja.md)
