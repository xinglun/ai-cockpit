---
author: AI Cockpit maintainers
title: "WI-381 — reference parity supersede decision link fix"
description: "Bind the versioned historical supersede decision in all parity projections without changing historical evidence."
workItemId: WI-381-reference-parity-decision-link-fix
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-381-reference-parity-decision-link-fix
terminalArchive: .ai/work-items/archive/WI-381-reference-parity-decision-link-fix.contract.json
terminalVerification: .ai/evidence/WI-381-reference-parity-decision-link-fix.verification.json
terminalFinalization: .ai/decisions/WI-381-reference-parity-decision-link-fix.finalize.json
terminalDecision: .ai/decisions/WI-381-reference-parity-decision-link-fix.close.json
capabilityClaims: [governance_integrity, reference_parity]
---

# WI-381 — reference parity supersede decision link fix

[简体中文](WI-381-reference-parity-decision-link-fix.zh-CN.md) · [日本語](WI-381-reference-parity-decision-link-fix.ja.md)

## Intent and boundary

The closed WI-379 predecessor has both a canonical successor decision and a
digest-versioned supersede decision. The three parity projections must expose
the exact terminal decision path so the governance gate can verify the
historical recovery chain. This Work Item changes only documentation links;
all archive, evidence, and decision bytes remain Runtime-owned and immutable.

## Scope

- Add the exact versioned WI-379 supersede decision path to all parity ledgers.
- Keep WI-379 archive, evidence, recovery, and close records unchanged.
- Keep English, Simplified Chinese, and Japanese projections semantically aligned.

## Out of scope

Runtime code, generated `.ai` records, release artifacts, and global Agent/MCP configuration.

## Acceptance

- Each parity row references WI-379 archive, verification, canonical recovery,
  versioned supersede recovery, and superseded close paths.
- The governance integrity gate reports no WI-379 `missing_parity_decision`.
- Historical archive/evidence digests are unchanged.

## Verification and terminal records

Use the installed Runtime with explicit `--repo`, the governance/documentation
checks, and `cargo test --locked --workspace`. After reviewed merge, record the
archive, verification, finalization, and close paths declared in this header.
